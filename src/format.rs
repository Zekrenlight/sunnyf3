//! How SunnyF3 words and colours its values. Nothing here needs the game.

/// ARGB colours for values by meaning.
pub const GOOD: u32 = 0xFF6E_E06E;
pub const FAIR: u32 = 0xFFFF_D45A;
pub const POOR: u32 = 0xFFFF_6060;
pub const AXIS_X: u32 = 0xFFFF_7A7A;
pub const AXIS_Y: u32 = 0xFF7A_F07A;
pub const AXIS_Z: u32 = 0xFF7A_B4FF;
pub const KEY: u32 = 0xFFA8_ADB8;
pub const VALUE: u32 = 0xFFF2_F2F2;
pub const SOFT: u32 = 0xFF8A_8F9A;
pub const TRUE: u32 = 0xFF6E_E06E;
pub const FALSE: u32 = 0xFFFF_7070;

/// Frames a second against what the screen can show: green from 90% of the refresh rate
/// (or 60), yellow from half of that, red below.
pub fn fps_colour(fps: f32, refresh_hz: Option<f32>) -> u32 {
    let target = refresh_hz.filter(|r| *r > 1.0).map_or(60.0, |r| (r * 0.9).min(60.0).max(30.0));
    if fps >= target {
        GOOD
    } else if fps >= target / 2.0 {
        FAIR
    } else {
        POOR
    }
}

/// A server's milliseconds a tick against its budget.
pub fn tick_colour(ms: f32, budget: f32) -> u32 {
    if ms <= budget * 0.6 {
        GOOD
    } else if ms <= budget {
        FAIR
    } else {
        POOR
    }
}

/// A share of something full (memory, GPU), 0 to 100.
pub fn load_colour(percent: f64) -> u32 {
    if percent < 60.0 {
        GOOD
    } else if percent < 85.0 {
        FAIR
    } else {
        POOR
    }
}

/// A light level, 0 to 15: where mobs can spawn (0) is red.
pub fn light_colour(level: u8) -> u32 {
    match level {
        0 => POOR,
        1..=7 => FAIR,
        _ => GOOD,
    }
}

/// The way the player faces from its yaw: the name and the axis it points along.
pub fn facing(yaw: f32) -> (&'static str, &'static str) {
    match ((f64::from(yaw) / 90.0 + 0.5).floor() as i32) & 3 {
        0 => ("South", "+Z"),
        1 => ("West", "-X"),
        2 => ("North", "-Z"),
        _ => ("East", "+X"),
    }
}

/// Degrees folded into -180 to 180.
pub fn wrap(deg: f32) -> f32 {
    let d = deg.rem_euclid(360.0);
    if d >= 180.0 { d - 360.0 } else { d }
}

/// The day's number and the time on a 24-hour clock (day starts at 06:00 in-game).
pub fn day_and_clock(day_time: i64) -> (i64, String) {
    let day = day_time.div_euclid(24000);
    let t = (day_time.rem_euclid(24000) + 6000) % 24000;
    let hours = t / 1000;
    let minutes = (t % 1000) * 60 / 1000;
    (day, format!("{hours:02}:{minutes:02}"))
}

/// `minecraft:dark_oak_forest` as `Dark Oak Forest`; another namespace stays in front.
pub fn pretty_id(id: &str) -> String {
    let (ns, path) = id.split_once(':').unwrap_or(("minecraft", id));
    let words: Vec<String> = path
        .split(['_', '/'])
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(f) => f.to_uppercase().chain(c).collect(),
                None => String::new(),
            }
        })
        .collect();
    let name = words.join(" ");
    if ns == "minecraft" { name } else { format!("{name} ({ns})") }
}

/// A number with a fixed number of decimals, `-0.000` shown as `0.000`.
pub fn fixed(v: f64, decimals: usize) -> String {
    let s = format!("{v:.decimals$}");
    if s.starts_with('-') && s[1..].chars().all(|c| c == '0' || c == '.') {
        s[1..].to_owned()
    } else {
        s
    }
}

/// The slowest 1% of frames, as frames a second, from frame times in milliseconds.
pub fn one_percent_low(frame_ms: &[f32]) -> Option<f32> {
    if frame_ms.len() < 20 {
        return None;
    }
    let mut v: Vec<f32> = frame_ms.iter().copied().filter(|m| m.is_finite() && *m > 0.0).collect();
    v.sort_by(|a, b| b.total_cmp(a));
    let n = (v.len() / 100).max(1);
    let avg = v[..n].iter().sum::<f32>() / n as f32;
    (avg > 0.0).then(|| 1000.0 / avg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words_and_numbers() {
        assert_eq!(facing(0.0), ("South", "+Z"));
        assert_eq!(facing(90.0), ("West", "-X"));
        assert_eq!(facing(180.0), ("North", "-Z"));
        assert_eq!(facing(-90.0), ("East", "+X"));
        assert_eq!(day_and_clock(0), (0, "06:00".into()));
        assert_eq!(day_and_clock(18000 + 24000 * 3), (3, "00:00".into()));
        assert_eq!(day_and_clock(6500), (0, "12:30".into()));
        assert_eq!(pretty_id("minecraft:dark_oak_forest"), "Dark Oak Forest");
        assert_eq!(pretty_id("terra:ash_flats"), "Ash Flats (terra)");
        assert_eq!(fixed(-0.0001, 3), "0.000");
        assert_eq!(fixed(-1.5, 1), "-1.5");
        assert_eq!(wrap(270.0), -90.0);
        assert_eq!(fps_colour(144.0, Some(144.0)), GOOD);
        assert_eq!(fps_colour(40.0, Some(144.0)), FAIR);
        assert_eq!(fps_colour(10.0, None), POOR);
        let mut frames = vec![10.0; 99];
        frames.push(50.0);
        assert_eq!(one_percent_low(&frames), Some(20.0));
    }
}
