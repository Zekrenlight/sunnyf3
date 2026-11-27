//! SunnyF3 1.0.0 for Sunrise, by Zekrenlight (MIT).
//!
//! Draws the F3 screen in sections: Performance, Position, World, Looking At, System and
//! Sunrise (and, if you want them, Sunrise's own debug entries as chosen with F3+F6). Each
//! section goes in the left or right column, at any place, or is hidden; values are coloured
//! by meaning; the panels, strips or bare text are styled from the settings page. Sunrise's
//! charts (F3+1 to F3+4) and every F3 key combination keep working as they are.
//!
//! This crate holds what needs no game types: the settings and the wording of values. The
//! overlay itself is `src/native/hud.rs`.

pub mod config;
pub mod format;

pub use config::*;

use mod_api::{
    ClientHost, ClientTickPostEvent, Host, Hooks, KeyRegistration, Registrar, RegistrationContext,
    SettingChangedEvent, SettingKey, SettingSpec,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

static OPEN_SETTINGS: AtomicBool = AtomicBool::new(false);

/// Whether the settings page was asked for (its key) since the last call.
pub fn take_open_settings() -> bool {
    OPEN_SETTINGS.swap(false, Ordering::Relaxed)
}

#[derive(Default)]
struct SunnyF3;

fn label(key: &str) -> String {
    format!("sunnyf3.setting.{key}")
}
fn toggle(r: &mut dyn Registrar, key: &str, default: bool) -> Option<SettingKey> {
    let l = label(key);
    r.register_setting(SettingSpec::toggle(key, l.clone(), default).tooltip(format!("{l}.tooltip"))).ok()
}
fn range(r: &mut dyn Registrar, key: &str, min: f64, max: f64, step: f64, default: f64) -> Option<SettingKey> {
    let l = label(key);
    r.register_setting(SettingSpec::range(key, l.clone(), min, max, step, default).tooltip(format!("{l}.tooltip"))).ok()
}
fn choice(r: &mut dyn Registrar, key: &str, options: &[&str], default: usize) -> Option<SettingKey> {
    let l = label(key);
    r.register_setting(
        SettingSpec::choice(key, l.clone(), options.iter().copied(), default).tooltip(format!("{l}.tooltip")),
    ).ok()
}
fn color(r: &mut dyn Registrar, key: &str, default: u32) -> Option<SettingKey> {
    let l = label(key);
    r.register_setting(SettingSpec::color(key, l.clone(), default).tooltip(format!("{l}.tooltip"))).ok()
}

struct SettingKeys {
    enabled: Option<SettingKey>,
    style: Option<SettingKey>,
    opacity: Option<SettingKey>,
    headers: Option<SettingKey>,
    shadow: Option<SettingKey>,
    colours: Option<SettingKey>,
    accent: Option<SettingKey>,
    refresh: Option<SettingKey>,
    places: [Option<SettingKey>; 7],
    orders: [Option<SettingKey>; 7],
}

static SETTING_KEYS: OnceLock<SettingKeys> = OnceLock::new();

fn bool_of(host: &dyn Host, key: Option<SettingKey>, default: bool) -> bool {
    key.and_then(|k| host.setting_bool(k).ok()).unwrap_or(default)
}
fn f64_of(host: &dyn Host, key: Option<SettingKey>, default: f64) -> f64 {
    key.and_then(|k| host.setting_f64(k).ok()).filter(|v| v.is_finite()).unwrap_or(default)
}
fn choice_of(host: &dyn Host, key: Option<SettingKey>, default: u32) -> u32 {
    key.and_then(|k| host.setting_choice(k).ok()).unwrap_or(default)
}

/// Reads every setting into the copy the game half uses.
fn sync(host: &dyn Host) {
    let d = Config::default();
    let Some(k) = SETTING_KEYS.get() else { return };
    set_config(Config {
        enabled: bool_of(host, k.enabled, d.enabled),
        style: Style::from_index(choice_of(host, k.style, 0)),
        opacity: f64_of(host, k.opacity, f64::from(d.opacity)).clamp(0.0, 100.0) as u32,
        headers: bool_of(host, k.headers, d.headers),
        shadow: bool_of(host, k.shadow, d.shadow),
        colours: bool_of(host, k.colours, d.colours),
        accent: k.accent.and_then(|a| host.setting_color(a).ok()).unwrap_or(d.accent) & 0xFF_FFFF,
        refresh: f64_of(host, k.refresh, f64::from(d.refresh)).clamp(50.0, 1000.0) as u32,
        layout: Section::ALL
            .iter()
            .enumerate()
            .map(|(i, &s)| {
                let place = Place::from_index(choice_of(host, k.places[i], s.default_place().index() as u32));
                let order = f64_of(host, k.orders[i], f64::from(s.default_order())).clamp(1.0, 9.0) as u32;
                (s, place, order)
            })
            .collect(),
    });
}

#[mod_api::sunrise_mod]
impl Hooks for SunnyF3 {
    fn register(&self, r: &mut dyn Registrar, _cx: &RegistrationContext) {
        r.register_key_mapping(
            KeyRegistration::new("settings", "key.keyboard.unknown")
                .label("sunnyf3.key.settings")
                .category("sunnyf3"),
        );
        let d = Config::default();
        let mut places = [None; 7];
        let mut orders = [None; 7];
        for (i, s) in Section::ALL.iter().enumerate() {
            places[i] = choice(r, &s.place_key(), &Place::OPTIONS, s.default_place().index());
            orders[i] = range(r, &s.order_key(), 1.0, 9.0, 1.0, f64::from(s.default_order()));
        }
        let keys = SettingKeys {
            enabled: toggle(r, KEY_ENABLED, d.enabled),
            style: choice(r, KEY_STYLE, &Style::OPTIONS, 0),
            opacity: range(r, KEY_OPACITY, 0.0, 100.0, 5.0, f64::from(d.opacity)),
            headers: toggle(r, KEY_HEADERS, d.headers),
            shadow: toggle(r, KEY_SHADOW, d.shadow),
            colours: toggle(r, KEY_COLOURS, d.colours),
            accent: color(r, KEY_ACCENT, d.accent),
            refresh: range(r, KEY_REFRESH, 50.0, 1000.0, 50.0, f64::from(d.refresh)),
            places,
            orders,
        };
        let _ = SETTING_KEYS.set(keys);
    }

    fn setting_changed(&self, host: &dyn Host, event: &mut SettingChangedEvent<'_>) {
        if event.mod_id == ID {
            sync(host);
        }
    }

    fn client_tick_post(&self, host: &mut dyn ClientHost, _event: &mut ClientTickPostEvent) {
        if !config_loaded() {
            sync(host);
        }
        if host.mod_key_mapping(ID, "settings").and_then(|m| m.consume_click(host)).unwrap_or(false) {
            OPEN_SETTINGS.store(true, Ordering::Relaxed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_columns() {
        let c = Config::default();
        assert_eq!(c.column(Place::Left), vec![Section::Performance, Section::Position, Section::World]);
        assert_eq!(c.column(Place::Right), vec![Section::Target, Section::System, Section::Sunrise]);
        assert!(!c.shows(Section::Entries));
        let mut c = c;
        c.layout[0].2 = 9;
        assert_eq!(c.column(Place::Left)[2], Section::Performance);
    }
}
