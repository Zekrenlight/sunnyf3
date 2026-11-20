//! SunnyF3's settings: their keys, defaults and the copy the game half reads.

use std::sync::Mutex;

pub const ID: &str = "sunnyf3";

pub const KEY_ENABLED: &str = "enabled";
pub const KEY_STYLE: &str = "style";
pub const KEY_OPACITY: &str = "opacity";
pub const KEY_HEADERS: &str = "headers";
pub const KEY_SHADOW: &str = "shadow";
pub const KEY_COLOURS: &str = "colours";
pub const KEY_ACCENT: &str = "accent";
pub const KEY_REFRESH: &str = "refresh";

/// The overlay's sections, in their default order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Section {
    Performance,
    Position,
    World,
    Target,
    System,
    Sunrise,
    Entries,
}

impl Section {
    pub const ALL: [Section; 7] = [
        Section::Performance,
        Section::Position,
        Section::World,
        Section::Target,
        Section::System,
        Section::Sunrise,
        Section::Entries,
    ];
    pub fn key(self) -> &'static str {
        match self {
            Self::Performance => "performance",
            Self::Position => "position",
            Self::World => "world",
            Self::Target => "target",
            Self::System => "system",
            Self::Sunrise => "sunrise",
            Self::Entries => "entries",
        }
    }
    pub fn title(self) -> &'static str {
        match self {
            Self::Performance => "Performance",
            Self::Position => "Position",
            Self::World => "World",
            Self::Target => "Looking At",
            Self::System => "System",
            Self::Sunrise => "Sunrise",
            Self::Entries => "Debug Entries",
        }
    }
    /// The setting holding where it goes, and the one holding its place in its column.
    pub fn place_key(self) -> String {
        format!("{}_place", self.key())
    }
    pub fn order_key(self) -> String {
        format!("{}_order", self.key())
    }
    pub fn default_place(self) -> Place {
        match self {
            Self::Performance | Self::Position | Self::World => Place::Left,
            Self::Target | Self::System | Self::Sunrise => Place::Right,
            Self::Entries => Place::Hidden,
        }
    }
    pub fn default_order(self) -> u32 {
        match self {
            Self::Performance => 1,
            Self::Position => 2,
            Self::World => 3,
            Self::Target => 1,
            Self::System => 2,
            Self::Sunrise => 3,
            Self::Entries => 4,
        }
    }
}

/// Which column a section is in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    Left,
    Right,
    Hidden,
}

impl Place {
    pub const OPTIONS: [&'static str; 3] = ["sunnyf3.place.left", "sunnyf3.place.right", "sunnyf3.place.hidden"];
    pub fn from_index(i: u32) -> Self {
        match i {
            0 => Self::Left,
            1 => Self::Right,
            _ => Self::Hidden,
        }
    }
    pub fn index(self) -> usize {
        match self {
            Self::Left => 0,
            Self::Right => 1,
            Self::Hidden => 2,
        }
    }
}

/// How the sections are drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    /// Each section on a panel of its own, its title on top.
    Panels,
    /// Each line on its own strip, as the game draws F3.
    Strips,
    /// Text only, with a shadow.
    Bare,
}

impl Style {
    pub const OPTIONS: [&'static str; 3] = ["sunnyf3.style.panels", "sunnyf3.style.strips", "sunnyf3.style.bare"];
    pub fn from_index(i: u32) -> Self {
        match i {
            1 => Self::Strips,
            2 => Self::Bare,
            _ => Self::Panels,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    pub enabled: bool,
    pub style: Style,
    /// The backgrounds' opacity, 0 to 100.
    pub opacity: u32,
    pub headers: bool,
    pub shadow: bool,
    /// Values coloured by meaning (good, fair, poor; axes; true and false).
    pub colours: bool,
    /// `0xRRGGBB`.
    pub accent: u32,
    /// How often the text is made again, in milliseconds.
    pub refresh: u32,
    /// Each section's column and place in it.
    pub layout: Vec<(Section, Place, u32)>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: true,
            style: Style::Panels,
            opacity: 55,
            headers: true,
            shadow: true,
            colours: true,
            accent: 0xF9801D,
            refresh: 100,
            layout: Section::ALL
                .iter()
                .map(|&s| (s, s.default_place(), s.default_order()))
                .collect(),
        }
    }
}

impl Config {
    /// The sections of a column, in order (ties keep the default order).
    pub fn column(&self, place: Place) -> Vec<Section> {
        let mut v: Vec<(u32, usize, Section)> = self
            .layout
            .iter()
            .enumerate()
            .filter(|(_, (_, p, _))| *p == place)
            .map(|(i, (s, _, o))| (*o, i, *s))
            .collect();
        v.sort_by_key(|(order, i, _)| (*order, *i));
        v.into_iter().map(|(_, _, s)| s).collect()
    }
    pub fn shows(&self, section: Section) -> bool {
        self.layout.iter().any(|(s, p, _)| *s == section && *p != Place::Hidden)
    }
}

static CONFIG: Mutex<Option<Config>> = Mutex::new(None);

pub fn set_config(config: Config) {
    *CONFIG.lock().unwrap_or_else(|e| e.into_inner()) = Some(config);
}
pub fn config() -> Config {
    CONFIG.lock().unwrap_or_else(|e| e.into_inner()).clone().unwrap_or_default()
}
pub fn config_loaded() -> bool {
    CONFIG.lock().unwrap_or_else(|e| e.into_inner()).is_some()
}
