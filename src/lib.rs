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
    ClientHost, ClientTickPostEvent, Hooks, KeyRegistration, Registrar, RegistrationContext,
    SettingChangedEvent, SettingSpec,
};
use std::sync::atomic::{AtomicBool, Ordering};

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
fn toggle(r: &mut dyn Registrar, key: &str, default: bool) {
    let l = label(key);
    let _ = r.register_setting(SettingSpec::toggle(key, l.clone(), default).tooltip(format!("{l}.tooltip")));
}
fn range(r: &mut dyn Registrar, key: &str, min: f64, max: f64, step: f64, default: f64) {
    let l = label(key);
    let _ = r.register_setting(SettingSpec::range(key, l.clone(), min, max, step, default).tooltip(format!("{l}.tooltip")));
}
fn choice(r: &mut dyn Registrar, key: &str, options: &[&str], default: usize) {
    let l = label(key);
    let _ = r.register_setting(
        SettingSpec::choice(key, l.clone(), options.iter().copied(), default).tooltip(format!("{l}.tooltip")),
    );
}
fn color(r: &mut dyn Registrar, key: &str, default: u32) {
    let l = label(key);
    let _ = r.register_setting(SettingSpec::color(key, l.clone(), default).tooltip(format!("{l}.tooltip")));
}

/// Reads every setting into the copy the game half uses.
fn sync(host: &dyn ClientHost) {
    let d = Config::default();
    let b = |key: &str, default: bool| host.setting_bool(key).unwrap_or(default);
    let n = |key: &str, default: f64| host.setting_f64(key).filter(|v| v.is_finite()).unwrap_or(default);
    let c = |key: &str| host.setting_choice(key);
    set_config(Config {
        enabled: b(KEY_ENABLED, d.enabled),
        style: Style::from_index(c(KEY_STYLE).unwrap_or(0)),
        opacity: n(KEY_OPACITY, f64::from(d.opacity)).clamp(0.0, 100.0) as u32,
        headers: b(KEY_HEADERS, d.headers),
        shadow: b(KEY_SHADOW, d.shadow),
        colours: b(KEY_COLOURS, d.colours),
        accent: host.setting_color(KEY_ACCENT).unwrap_or(d.accent) & 0xFF_FFFF,
        refresh: n(KEY_REFRESH, f64::from(d.refresh)).clamp(50.0, 1000.0) as u32,
        layout: Section::ALL
            .iter()
            .map(|&s| {
                let place = c(&s.place_key()).map_or(s.default_place(), Place::from_index);
                let order = n(&s.order_key(), f64::from(s.default_order())).clamp(1.0, 9.0) as u32;
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
        toggle(r, KEY_ENABLED, d.enabled);
        choice(r, KEY_STYLE, &Style::OPTIONS, 0);
        range(r, KEY_OPACITY, 0.0, 100.0, 5.0, f64::from(d.opacity));
        toggle(r, KEY_HEADERS, d.headers);
        toggle(r, KEY_SHADOW, d.shadow);
        toggle(r, KEY_COLOURS, d.colours);
        color(r, KEY_ACCENT, d.accent);
        range(r, KEY_REFRESH, 50.0, 1000.0, 50.0, f64::from(d.refresh));
        for s in Section::ALL {
            choice(r, &s.place_key(), &Place::OPTIONS, s.default_place().index());
            range(r, &s.order_key(), 1.0, 9.0, 1.0, f64::from(s.default_order()));
        }
    }

    fn setting_changed(&self, host: &mut dyn ClientHost, event: &mut SettingChangedEvent<'_>) {
        if event.mod_id == ID {
            sync(host);
        }
    }

    fn client_tick_post(&self, host: &mut dyn ClientHost, _event: &mut ClientTickPostEvent) {
        if !config_loaded() {
            sync(host);
        }
        if host.consume_click(ID, "settings") {
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
