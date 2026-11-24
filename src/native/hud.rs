// SunnyF3 in the game (not in the Cargo library): the F3 screen in sections, drawn in place
// of the game's text while F3 is open, and the settings key.
use super::{Game, Screen};
use mod_sunnyf3::config::{self, Config, Place, Section, Style};
use mod_sunnyf3::format as f;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use sunrise_client::debug_overlay::{self, Displayer, PaintCache};
use sunrise_client::hud::HudBatch;
use sunrise_client::hud_layout::{self, GuiDraw, PoseOp};
use sunrise_client::options_ui::Page;
use sunrise_client::text::Text;

const LINE: i32 = 10;
const PAD: i32 = 3;
const GAP: i32 = 4;
/// Frame times kept for the 1% low: about five seconds at 200 fps.
const FRAMES: usize = 1000;

/// One coloured run of a line.
type Run = (String, u32);
/// A line: its runs, left to right.
type Line = Vec<Run>;

struct State {
    paint: PaintCache,
    /// The settings the paint was made with.
    painted_with: Option<Config>,
    last_frame: Option<Instant>,
    frames: VecDeque<f32>,
    refresh_hz: Option<(Instant, Option<f32>)>,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State {
        paint: PaintCache::default(),
        painted_with: None,
        last_frame: None,
        frames: VecDeque::new(),
        refresh_hz: None,
    });
}

static CPU: OnceLock<String> = OnceLock::new();

fn kv(key: &str, value: impl Into<String>, colour: u32) -> Line {
    vec![(format!("{key} "), f::KEY), (value.into(), colour)]
}

/// Once a frame: the settings key opens SunnyF3's page.
pub(super) fn frame(game: &mut Game<'_>) {
    if !mod_sunnyf3::take_open_settings() {
        return;
    }
    game.list_native_mods();
    let index = game
        .settings
        .mods
        .installed
        .iter()
        .position(|m| m.id == mod_sunnyf3::ID && m.native.is_some());
    let Some(index) = index else { return };
    let from_play = game.screen == Screen::Playing;
    game.open_options_at(false, Page::NativeMod(index));
    if from_play {
        game.mining = None;
        game.pause_menu = true;
        game.capture(false);
    }
}

/// Draws the overlay when SunnyF3 has it: F3 is open and the mod is on. False leaves it to
/// the game (F3 closed, with entries shown always, or SunnyF3 turned off).
pub(super) fn draw(game: &Game<'_>, batch: &mut HudBatch, size: [u32; 2]) -> bool {
    let cfg = config::config();
    if !cfg.enabled || !game.debug {
        STATE.with(|s| {
            if let Ok(mut s) = s.try_borrow_mut() {
                s.last_frame = None;
                s.frames.clear();
            }
        });
        return false;
    }
    // Only in a world with nothing over it: no pause menu, inventory, chat, container, book,
    // sign or mod screen. F3+F6's own screen keeps the game's preview.
    if game.screen == Screen::DebugOptions {
        return false;
    }
    if !in_play(game) {
        STATE.with(|s| {
            if let Ok(mut s) = s.try_borrow_mut() {
                s.last_frame = None;
            }
        });
        return true;
    }
    STATE.with(|cell| {
        let Ok(mut s) = cell.try_borrow_mut() else { return };
        // Frame times, for the 1% low.
        let now = Instant::now();
        if let Some(last) = s.last_frame {
            let ms = now.duration_since(last).as_secs_f32() * 1000.0;
            if ms < 1000.0 {
                s.frames.push_back(ms);
                while s.frames.len() > FRAMES {
                    s.frames.pop_front();
                }
            }
        }
        s.last_frame = Some(now);
        if game.ui.hide_gui {
            return;
        }
        // Whether the lightmap chart has a current sample is part of the cache key (as in the
        // game's own overlay), so a sample that stops being current is not painted again.
        let lightmap = lightmap_sample(game).is_some();
        let layout = (size, batch.scale.to_bits(), game.gui.atlas.font.epoch(), lightmap);
        let same = s.painted_with.as_ref() == Some(&cfg);
        if same && let Some(vertices) = fresh(&s.paint, now, layout, cfg.refresh) {
            batch.vertices.extend_from_slice(vertices);
            return;
        }
        let first = batch.vertices.len();
        paint(game, &mut s, &cfg, batch, size);
        s.paint.store(now, layout, &batch.vertices[first..]);
        s.painted_with = Some(cfg);
    });
    true
}

/// The lightmap chart's pixels, once the texture drawn now has been read back
/// (`Renderer::lightmap_sample`), while the chart is on.
fn lightmap_sample<'a>(game: &'a Game<'_>) -> Option<&'a [u8; 16 * 16 * 4]> {
    if !game.debug_charts.lightmap {
        return None;
    }
    game.renderer.as_ref().and_then(|r| r.lightmap_sample())
}

/// Playing, with no screen of any kind open over the world.
fn in_play(game: &Game<'_>) -> bool {
    game.playing() && game.container.is_none() && game.mod_screen.is_none()
}

/// The paint while it is younger than the refresh setting (the game's cache keeps it 50 ms;
/// asking as if it were that much earlier keeps it for the rest).
fn fresh<'a>(
    paint: &'a PaintCache,
    now: Instant,
    layout: debug_overlay::PaintLayout,
    refresh_ms: u32,
) -> Option<&'a [sunrise_client::hud::HudVertex]> {
    let extra = Duration::from_millis(u64::from(refresh_ms.saturating_sub(50)));
    paint.fresh(now.checked_sub(extra).unwrap_or(now), layout)
}

fn refresh_hz(game: &Game<'_>, s: &mut State) -> Option<f32> {
    if let Some((at, hz)) = s.refresh_hz
        && at.elapsed() < Duration::from_secs(1)
    {
        return hz;
    }
    let hz = game
        .window
        .as_ref()
        .and_then(|w| w.current_monitor())
        .and_then(|m| m.refresh_rate_millihertz())
        .map(|mhz| mhz as f32 / 1000.0);
    s.refresh_hz = Some((Instant::now(), hz));
    hz
}

fn performance(game: &Game<'_>, s: &mut State, cfg: &Config) -> Vec<Line> {
    let mut out = Vec::new();
    let hz = refresh_hz(game, s);
    let fps = game.fps;
    let tint = |c: u32| if cfg.colours { c } else { f::VALUE };
    let mut line = vec![
        ("FPS ".to_owned(), f::KEY),
        (format!("{}", fps.round() as i32), tint(f::fps_colour(fps, hz))),
    ];
    if fps > 0.0 {
        line.push((format!("  {} ms", f::fixed(f64::from(1000.0 / fps), 1)), f::SOFT));
    }
    let frames: Vec<f32> = s.frames.iter().copied().collect();
    if let Some(low) = f::one_percent_low(&frames) {
        line.push(("  1% low ".to_owned(), f::KEY));
        line.push((format!("{}", low.round() as i32), tint(f::fps_colour(low, hz))));
    }
    out.push(line);
    let limit = game.framerate_limit();
    let limit = if limit == 260 { "Unlimited".to_owned() } else { limit.to_string() };
    let sync = game.renderer.as_ref().map_or("", |r| match r.config.present_mode {
        wgpu::PresentMode::Immediate => "Off",
        wgpu::PresentMode::Mailbox => "Mailbox",
        wgpu::PresentMode::Fifo => "On",
        wgpu::PresentMode::FifoRelaxed => "Relaxed",
        _ => "",
    });
    let mut line = kv("Limit", limit, f::VALUE);
    if !sync.is_empty() {
        line.push(("  VSync ".to_owned(), f::KEY));
        line.push((sync.to_owned(), f::VALUE));
    }
    if let Some(hz) = hz {
        line.push((format!("  {}Hz", hz.round() as i32), f::SOFT));
    }
    out.push(line);
    if let Some(share) = game.renderer.as_ref().and_then(|r| r.frame_generation_share()) {
        out.push(kv("Frame Gen", format!("{} fps shown", (fps * (1.0 + share)).round() as i32), f::VALUE));
    }
    // The server: its time a tick against the budget, or a remote server's brand.
    if game.player.is_some() {
        let budget = 1000.0 / game.clock.rate();
        let state = if game.clock.sprinting() {
            " (sprinting)"
        } else if game.clock.stepping() {
            " (stepping)"
        } else if game.clock.frozen() {
            " (frozen)"
        } else {
            ""
        };
        if game.remote.is_none() || game.integrated.is_some() {
            let ms = game
                .integrated
                .as_ref()
                .map_or(game.debug_charts.smoothed_tick_ms, |server| server.smoothed_tick_ms());
            let tps = if ms > 0.0 { (1000.0 / ms.max(budget)).min(1000.0 / budget) } else { 1000.0 / budget };
            out.push(vec![
                ("Server ".to_owned(), f::KEY),
                (format!("{} ms", f::fixed(f64::from(ms), 1)), tint(f::tick_colour(ms, budget))),
                (format!(" / {} ms", f::fixed(f64::from(budget), 0)), f::SOFT),
                (format!("  {} TPS{state}", f::fixed(f64::from(tps), 1)), f::VALUE),
            ]);
        } else if let Some(remote) = &game.remote {
            out.push(kv("Server", format!("{}{state}", remote.brand), f::VALUE));
        }
        let [tx, rx] = game.debug_charts.packet_rates.map(|n| n.round() as i64);
        out.push(kv("Packets", format!("{tx} out  {rx} in /s"), f::VALUE));
    }
    out
}

fn position(game: &Game<'_>, cfg: &Config) -> Vec<Line> {
    let Some(p) = game.player.as_ref() else { return Vec::new() };
    let tint = |c: u32| if cfg.colours { c } else { f::VALUE };
    let mut out = Vec::new();
    let reduced = game.reduced_debug_info();
    let pos = [p.position.x, p.position.y, p.position.z];
    if !reduced {
        out.push(vec![
            ("X ".to_owned(), tint(f::AXIS_X)),
            (f::fixed(pos[0], 3), f::VALUE),
            ("  Y ".to_owned(), tint(f::AXIS_Y)),
            (f::fixed(pos[1], 3), f::VALUE),
            ("  Z ".to_owned(), tint(f::AXIS_Z)),
            (f::fixed(pos[2], 3), f::VALUE),
        ]);
        let b = pos.map(|v| v.floor() as i32);
        out.push(kv("Block", format!("{} {} {}", b[0], b[1], b[2]), f::VALUE));
        let mut chunk = kv("Chunk", format!("{} {} {}", b[0] >> 4, b[1] >> 4, b[2] >> 4), f::VALUE);
        chunk.push((format!("  at {} {} {}", b[0] & 15, b[1] & 15, b[2] & 15), f::SOFT));
        out.push(chunk);
    }
    let (name, axis) = f::facing(game.yaw);
    let mut facing = kv("Facing", name, f::VALUE);
    facing.push((format!(" {axis}"), f::SOFT));
    facing.push((
        format!("  {}° / {}°", f::fixed(f64::from(f::wrap(game.yaw)), 1), f::fixed(f64::from(f::wrap(game.pitch)), 1)),
        f::SOFT,
    ));
    out.push(facing);
    let speed = game.known_player_movement().length() * 20.0;
    out.push(kv("Speed", format!("{} blocks/s", f::fixed(speed, 2)), f::VALUE));
    out.push(kv("Dimension", f::pretty_id(game.dim.id()), f::VALUE));
    if reduced {
        out.push(vec![("Reduced debug info".to_owned(), f::SOFT)]);
    }
    out
}

fn world(game: &Game<'_>, cfg: &Config) -> Vec<Line> {
    let Some(p) = game.player.as_ref() else { return Vec::new() };
    let tint = |c: u32| if cfg.colours { c } else { f::VALUE };
    let mut out = Vec::new();
    let [x, y, z] = [p.position.x, p.position.y, p.position.z].map(|v| v.floor() as i32);
    if y >= game.world.bottom_y()
        && y < game.world.top_y()
        && let Some(chunk) = game.world.chunks.get(&(x >> 4, z >> 4))
        && let Some(biome) = game.res.biomes().list.get(chunk.biome(x, y, z) as usize)
    {
        out.push(kv("Biome", f::pretty_id(&biome.name), f::VALUE));
    }
    let light = game.world.light(x, y, z);
    let (sky, block) = (light >> 4, light & 15);
    out.push(vec![
        ("Light ".to_owned(), f::KEY),
        (format!("{}", sky.max(block)), tint(f::light_colour(sky.max(block)))),
        (format!("  sky {sky}  block "), f::SOFT),
        (format!("{block}"), tint(f::light_colour(block))),
    ]);
    let (day, clock) = f::day_and_clock(game.world.day_time);
    let mut time = kv("Day", day.to_string(), f::VALUE);
    time.push(("  ".to_owned(), f::SOFT));
    time.push((clock, f::VALUE));
    out.push(time);
    out
}

fn target(game: &Game<'_>, cfg: &Config) -> Vec<Line> {
    if game.player.is_none() || game.reduced_debug_info() {
        return Vec::new();
    }
    let mut out = Vec::new();
    if let Some((origin, dir)) = game.aim()
        && let Some(hit) = game.clip_ray(origin, dir, 20.0)
    {
        let pos = hit.pos;
        let id = game.world.block(pos[0], pos[1], pos[2]);
        if let Some(state) = game.pack.assets.registry.state(id) {
            out.push(kv("Block", f::pretty_id(&state.name), f::VALUE));
            out.push(vec![(format!("{}  at {} {} {}", state.name, pos[0], pos[1], pos[2]), f::SOFT)]);
            let mut props: Line = Vec::new();
            for (i, (k, v)) in state.properties.iter().enumerate() {
                if i > 0 && i % 3 == 0 {
                    out.push(std::mem::take(&mut props));
                }
                let colour = match v.as_str() {
                    "true" if cfg.colours => f::TRUE,
                    "false" if cfg.colours => f::FALSE,
                    _ => f::VALUE,
                };
                props.push((format!("{k}="), f::KEY));
                props.push((format!("{v}  "), colour));
            }
            if !props.is_empty() {
                out.push(props);
            }
        }
    }
    if let Some((i, distance, _)) = game.mob_hit()
        && game.before_block(distance)
        && let Some(mob) = game.mobs.get(i)
    {
        let mut line = kv("Entity", f::pretty_id(&mob.kind.id), f::VALUE);
        line.push((format!("  {} m", f::fixed(distance, 1)), f::SOFT));
        out.push(line);
    }
    out
}

fn system(game: &Game<'_>, cfg: &Config, size: [u32; 2]) -> Vec<Line> {
    let tint = |c: u32| if cfg.colours { c } else { f::VALUE };
    let mut out = Vec::new();
    out.push(kv("CPU", CPU.get_or_init(sunrise_client::crash::cpu).clone(), f::VALUE));
    if let Some(r) = game.renderer.as_ref() {
        let info = &r.info;
        let kind = match info.device_type {
            wgpu::DeviceType::IntegratedGpu => " (integrated)",
            wgpu::DeviceType::DiscreteGpu => "",
            wgpu::DeviceType::VirtualGpu => " (virtual)",
            wgpu::DeviceType::Cpu => " (software)",
            wgpu::DeviceType::Other => "",
        };
        out.push(kv("GPU", format!("{}{kind}", info.name), f::VALUE));
        let driver = format!("{} {}", info.driver, info.driver_info);
        let driver = driver.trim().lines().next().unwrap_or("").to_owned();
        let mut api = kv("API", format!("{:?}", info.backend), f::VALUE);
        if !driver.is_empty() {
            api.push((format!("  {driver}"), f::SOFT));
        }
        out.push(api);
    }
    let gpu = game.gpu_utilization;
    if gpu > 0.0 {
        out.push(vec![("GPU Load ".to_owned(), f::KEY), (format!("{}%", gpu.round().min(100.0) as i64), tint(f::load_colour(gpu)))]);
    }
    if let Some(lines) = game.memory_meter.borrow_mut().lines() {
        // "Mem: 45% 1234/4096MiB": the share, coloured, then the amounts.
        let mem = lines[0].trim_start_matches("Mem:").trim().to_owned();
        let (share, rest) = mem.split_once(' ').unwrap_or((mem.as_str(), ""));
        let percent: f64 = share.trim_end_matches('%').trim().parse().unwrap_or(0.0);
        out.push(vec![
            ("Memory ".to_owned(), f::KEY),
            (share.to_owned(), tint(f::load_colour(percent))),
            (format!("  {}", rest.trim()), f::SOFT),
        ]);
    }
    let mut display = kv("Display", format!("{}x{}", size[0], size[1]), f::VALUE);
    if let Some(w) = game.window.as_ref() {
        let density = w.scale_factor();
        if (density - 1.0).abs() > 0.01 {
            display.push((format!("  {density:.2}x"), f::SOFT));
        }
    }
    out.push(display);
    out
}

fn sunrise(game: &Game<'_>) -> Vec<Line> {
    let mut out = Vec::new();
    let mut version = kv("Sunrise", sunrise_core::release::VERSION, f::VALUE);
    version.push((format!("  Minecraft {}", sunrise_core::VANILLA_VERSION), f::SOFT));
    out.push(version);
    let mods = &game.settings.mods;
    let running = (0..mods.installed.len()).filter(|&i| mods.is_on(i) && mods.installed[i].problem.is_none()).count();
    out.push(kv("Mods", format!("{running} running of {}", mods.installed.len()), f::VALUE));
    if let Some(r) = game.renderer.as_ref() {
        let distance = game.settings.render_distance;
        let mut chunks = kv("Sections", format!("{} drawn", r.drawn_sections), f::VALUE);
        chunks.push((format!("  {} queued  view {distance}", game.streaming.pending()), f::SOFT));
        out.push(chunks);
        let mut ents = kv("Entities", format!("{} drawn", game.rendered_mobs.get()), f::VALUE);
        ents.push((format!("  {} particles", game.particles.len()), f::SOFT));
        out.push(ents);
        let pack: f32 = r.shader_pass_times().iter().map(|p| p.1).sum();
        if pack > 0.0 {
            out.push(kv("Shader Pack", format!("{} ms GPU", f::fixed(f64::from(pack), 2)), f::VALUE));
        }
        let own: f32 = r.own_pass_times().iter().map(|p| p.1).sum();
        if own > 0.0 {
            out.push(kv("Sunrise Passes", format!("{} ms GPU", f::fixed(f64::from(own), 2)), f::VALUE));
        }
        if game.settings.sunrise.dynamic_resolution {
            let scale = super::frame_autotune::world_scale(&game.settings.sunrise, &game.dynamic_resolution);
            out.push(kv("Resolution", format!("{scale}% (dynamic)"), f::VALUE));
        }
        if r.output().requests_hdr(game.settings.sunrise.hdr) {
            out.push(vec![(r.hdr_line(), f::VALUE)]);
        }
    }
    let effects = game.requested_post_effects();
    if !effects.is_empty() {
        out.push(kv("Post", effects.join(", "), f::VALUE));
    }
    out
}

/// Sunrise's own entries as F3+F6 has them, a line each (the native mods' entries too).
fn entries(game: &Game<'_>, size: [u32; 2]) -> Vec<Line> {
    let mut d = Displayer::default();
    game.debug_entry_lines(&mut d, size.map(|v| v as i32));
    let (left, right) = d.columns(&[]);
    left.into_iter()
        .chain(right)
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let plain = Text::legacy(&l).plain();
            match plain.split_once(": ") {
                Some((k, v)) if k.len() < 28 => vec![(format!("{k}: "), f::KEY), (v.to_owned(), f::VALUE)],
                _ => vec![(plain, f::VALUE)],
            }
        })
        .collect()
}

fn lines_of(game: &Game<'_>, s: &mut State, cfg: &Config, section: Section, size: [u32; 2]) -> Vec<Line> {
    match section {
        Section::Performance => performance(game, s, cfg),
        Section::Position => position(game, cfg),
        Section::World => world(game, cfg),
        Section::Target => target(game, cfg),
        Section::System => system(game, cfg, size),
        Section::Sunrise => sunrise(game),
        Section::Entries => entries(game, size),
    }
}

fn line_text(line: &Line) -> Text {
    let mut t = Text::empty();
    for (s, c) in line {
        t.push(Text::literal(s.clone()).with_color(c & 0xFF_FFFF));
    }
    t
}

fn paint(game: &Game<'_>, s: &mut State, cfg: &Config, batch: &mut HudBatch, size: [u32; 2]) {
    let framebuffer = size.map(|v| v as i32);
    let (scale, [width, height]) = debug_overlay::scale(
        framebuffer,
        batch.scale as i32,
        game.settings.file.int("debugGuiScale"),
        game.settings.force_unicode,
        [batch.width as i32, batch.height as i32],
    );
    let font = &game.gui.atlas.font;
    // The server's half of Sunrise's entries is only asked for while they show.
    if !cfg.shows(Section::Entries)
        && let Some(server) = &game.integrated
    {
        server.request_debug(None);
    }
    let accent = 0xFF00_0000 | cfg.accent;
    let alpha = (cfg.opacity.min(100) * 255 / 100) << 24;
    let panel = alpha | 0x0F_1118;
    let header_bg = ((cfg.opacity.min(100) * 255 / 100).saturating_add(30).min(255) << 24) | 0x1A_1D28;
    let shadow = cfg.shadow || cfg.style == Style::Bare;
    let mut draws = vec![GuiDraw::Push(vec![PoseOp::Scale(scale, scale)])];
    for (place, left) in [(Place::Left, true), (Place::Right, false)] {
        let mut y = 2;
        for section in cfg.column(place) {
            let lines = lines_of(game, s, cfg, section, size);
            if lines.is_empty() {
                continue;
            }
            let texts: Vec<Text> = lines.iter().map(line_text).collect();
            let title = Text::literal(section.title().to_owned()).bold().with_color(cfg.accent);
            let mut w = texts.iter().map(|t| hud_layout::width(font, t)).max().unwrap_or(0);
            let headers = cfg.headers;
            if headers {
                w = w.max(hud_layout::width(font, &title));
            }
            let x = if left { 2 } else { width - 2 - w - 2 * PAD };
            let rows = texts.len() as i32 + i32::from(headers);
            let h = rows * LINE + 2 * PAD - 1;
            if y + h > height && y > 2 {
                break;
            }
            match cfg.style {
                Style::Panels => {
                    if alpha != 0 {
                        draws.push(hud_layout::fill(x, y, x + w + 2 * PAD, y + h, panel));
                        if headers {
                            draws.push(hud_layout::fill(x, y, x + w + 2 * PAD, y + LINE + PAD - 1, header_bg));
                        }
                    }
                    let bar_x = if left { x } else { x + w + 2 * PAD - 1 };
                    draws.push(hud_layout::fill(bar_x, y, bar_x + 1, y + h, accent));
                }
                Style::Strips => {
                    if alpha != 0 {
                        let mut ly = y + PAD;
                        let all: Vec<&Text> = headers.then_some(&title).into_iter().chain(texts.iter()).collect();
                        for t in all {
                            let tw = hud_layout::width(font, t);
                            let lx = if left { x + PAD } else { x + PAD + w - tw };
                            draws.push(hud_layout::fill(lx - 1, ly - 1, lx + tw + 1, ly + LINE - 1, panel));
                            ly += LINE;
                        }
                    }
                }
                Style::Bare => {}
            }
            let mut ly = y + PAD;
            if headers {
                let tw = hud_layout::width(font, &title);
                let tx = if left { x + PAD } else { x + PAD + w - tw };
                draws.push(GuiDraw::Text { text: title.clone(), x: tx, y: ly, color: 0xFFFF_FFFF, shadow });
                ly += LINE;
            }
            for t in texts {
                let tw = hud_layout::width(font, &t);
                let tx = if left { x + PAD } else { x + PAD + w - tw };
                draws.push(GuiDraw::Text { text: t, x: tx, y: ly, color: 0xFFFF_FFFF, shadow });
                ly += LINE;
            }
            y += h + GAP;
        }
    }
    draws.push(GuiDraw::Pop);
    // Sunrise's charts, as the game draws them while F3 is open.
    let charts = game.debug_charts.draw(
        font,
        width,
        height,
        game.settings.max_fps,
        &game.clock,
        !game.has_singleplayer_server(),
    );
    draws.push(GuiDraw::Push(vec![PoseOp::Scale(scale, scale)]));
    draws.extend(charts);
    if let Some(pixels) = lightmap_sample(game) {
        draws.extend(sunrise_client::debug_chart::lightmap(pixels, width, height));
    }
    draws.push(GuiDraw::Pop);
    hud_layout::paint(batch, &game.gui.atlas, &draws);
}
