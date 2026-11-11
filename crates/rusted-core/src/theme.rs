use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ThemeName {
    #[default]
    ObsidianBlack,
    Midnight,
    Graphite,
    PureBlack,
    Snow,
    WarmSand,
}

impl ThemeName {
    pub const ALL: [Self; 6] =
        [Self::ObsidianBlack, Self::Midnight, Self::Graphite, Self::PureBlack, Self::Snow, Self::WarmSand];

    pub fn label(self) -> &'static str {
        match self {
            Self::ObsidianBlack => "Obsidian Black",
            Self::Midnight => "Midnight",
            Self::Graphite => "Graphite",
            Self::PureBlack => "Pure Black",
            Self::Snow => "Snow",
            Self::WarmSand => "Warm Sand",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::ObsidianBlack => "Default. Deep neutral charcoal with silver accents.",
            Self::Midnight => "Cool graphite with a faint blue undertone.",
            Self::Graphite => "Medium charcoal, softer contrast.",
            Self::PureBlack => "True black surfaces for OLED displays.",
            Self::Snow => "Bright, low-glare light theme.",
            Self::WarmSand => "Warm neutral paper tones.",
        }
    }

    pub fn is_dark(self) -> bool {
        !matches!(self, Self::Snow | Self::WarmSand)
    }

    /// Centralised palette. Every surface in the UI resolves through these tokens,
    /// so switching themes cannot leave stale colours behind.
    pub fn tokens(self) -> ThemeTokens {
        match self {
            Self::ObsidianBlack => ThemeTokens {
                background: 0x101010,
                sidebar: 0x141414,
                panel: 0x171717,
                elevated: 0x1d1d1d,
                hover: 0x252525,
                border: 0x2a2a2a,
                text: 0xf2f2f2,
                secondary: 0xa0a0a0,
                muted: 0x707070,
                accent: 0xd4d4d4,
                selection: 0x2f2f2f,
                danger: 0xd66b6b,
                success: 0x79a886,
                warning: 0xd3ac6b,
                overlay: 0x0a0a0a,
                is_dark: true,
            },
            Self::Midnight => ThemeTokens {
                background: 0x101216,
                sidebar: 0x14161a,
                panel: 0x181b20,
                elevated: 0x1e2228,
                hover: 0x282d34,
                border: 0x303640,
                text: 0xf0f2f4,
                secondary: 0xa5acb5,
                muted: 0x737c88,
                accent: 0xc6cdd5,
                selection: 0x2c3642,
                danger: 0xd9776f,
                success: 0x7fb0a0,
                warning: 0xd6b077,
                overlay: 0x0b0d10,
                is_dark: true,
            },
            Self::Graphite => ThemeTokens {
                background: 0x1c1c1c,
                sidebar: 0x202020,
                panel: 0x252525,
                elevated: 0x2b2b2b,
                hover: 0x343434,
                border: 0x3b3b3b,
                text: 0xf4f4f4,
                secondary: 0xb0b0b0,
                muted: 0x808080,
                accent: 0xdadada,
                selection: 0x3a3a3a,
                danger: 0xd8796f,
                success: 0x8bb59b,
                warning: 0xd8b674,
                overlay: 0x121212,
                is_dark: true,
            },
            Self::PureBlack => ThemeTokens {
                background: 0x000000,
                sidebar: 0x080808,
                panel: 0x0b0b0b,
                elevated: 0x121212,
                hover: 0x1d1d1d,
                border: 0x252525,
                text: 0xf5f5f5,
                secondary: 0xa0a0a0,
                muted: 0x686868,
                accent: 0xe0e0e0,
                selection: 0x242424,
                danger: 0xdb6f6f,
                success: 0x7fae8c,
                warning: 0xd2ab68,
                overlay: 0x000000,
                is_dark: true,
            },
            Self::Snow => ThemeTokens {
                background: 0xf5f5f4,
                sidebar: 0xeeeeec,
                panel: 0xffffff,
                elevated: 0xffffff,
                hover: 0xe5e5e3,
                border: 0xd8d8d4,
                text: 0x1d1d1c,
                secondary: 0x575754,
                muted: 0x82827d,
                accent: 0x353533,
                selection: 0xe0e6ee,
                danger: 0xb3453f,
                success: 0x2f6f4a,
                warning: 0x8a6416,
                overlay: 0xffffff,
                is_dark: false,
            },
            Self::WarmSand => ThemeTokens {
                background: 0xeee9df,
                sidebar: 0xe7e0d4,
                panel: 0xf5f0e7,
                elevated: 0xfaf6ef,
                hover: 0xddd4c6,
                border: 0xcec3b2,
                text: 0x29251f,
                secondary: 0x665e51,
                muted: 0x8a8071,
                accent: 0x554b3b,
                selection: 0xe2d8c6,
                danger: 0xa8463c,
                success: 0x4f6b46,
                warning: 0x7d5f22,
                overlay: 0xfaf6ef,
                is_dark: false,
            },
        }
    }

    /// Cycles for the Ctrl+Shift+F1 style quick theme shortcut.
    pub fn next(self) -> Self {
        let all = Self::ALL;
        let index = all.iter().position(|theme| *theme == self).unwrap_or(0);
        all[(index + 1) % all.len()]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeTokens {
    pub background: u32,
    pub sidebar: u32,
    pub panel: u32,
    pub elevated: u32,
    pub hover: u32,
    pub border: u32,
    pub text: u32,
    pub secondary: u32,
    pub muted: u32,
    pub accent: u32,
    pub selection: u32,
    pub danger: u32,
    pub success: u32,
    pub warning: u32,
    pub overlay: u32,
    pub is_dark: bool,
}

impl ThemeTokens {
    pub fn all_surfaces(self) -> [u32; 6] {
        [self.background, self.sidebar, self.panel, self.elevated, self.hover, self.selection]
    }

    /// Relative luminance in the sRGB space, used for contrast tests.
    pub fn luminance(color: u32) -> f32 {
        let channel = |shift: u32| {
            let value = ((color >> shift) & 0xff) as f32 / 255.0;
            if value <= 0.03928 { value / 12.92 } else { ((value + 0.055) / 1.055).powf(2.4) }
        };
        0.2126 * channel(16) + 0.7152 * channel(8) + 0.0722 * channel(0)
    }

    pub fn contrast_ratio(self, foreground: u32, background: u32) -> f32 {
        let (a, b) = (Self::luminance(foreground), Self::luminance(background));
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }
}

/// Spacing, radius and typography tokens shared by every component.
#[derive(Debug, Clone, Copy)]
pub struct Metrics;

impl Metrics {
    pub const RADIUS_BUTTON: f32 = 9.0;
    pub const RADIUS_INPUT: f32 = 10.0;
    pub const RADIUS_TAB: f32 = 11.0;
    pub const RADIUS_PANEL: f32 = 14.0;
    pub const RADIUS_DIALOG: f32 = 18.0;
    pub const SPACE_XS: f32 = 4.0;
    pub const SPACE_SM: f32 = 8.0;
    pub const SPACE_MD: f32 = 12.0;
    pub const SPACE_LG: f32 = 16.0;
    pub const SPLITTER_SIZE: f32 = 5.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_theme_defines_distinct_surfaces_and_readable_text() {
        for theme in ThemeName::ALL {
            let tokens = theme.tokens();
            let surfaces = tokens.all_surfaces();
            assert!(surfaces.iter().any(|value| *value != surfaces[0]), "{} lacks surface variation", theme.label());
            let primary = tokens.contrast_ratio(tokens.text, tokens.background);
            let secondary = tokens.contrast_ratio(tokens.secondary, tokens.panel);
            assert!(primary >= 7.0, "{} primary contrast {}", theme.label(), primary);
            assert!(secondary >= 3.5, "{} secondary contrast {}", theme.label(), secondary);
        }
    }

    #[test]
    fn themes_are_explicitly_light_or_dark() {
        assert!(ThemeName::ObsidianBlack.tokens().is_dark);
        assert!(!ThemeName::Snow.tokens().is_dark);
        assert!(!ThemeName::WarmSand.tokens().is_dark);
    }

    #[test]
    fn theme_cycle_visits_every_variant() {
        let mut theme = ThemeName::ObsidianBlack;
        let mut seen = 1;
        while theme.next() != ThemeName::ObsidianBlack {
            theme = theme.next();
            seen += 1;
        }
        assert_eq!(seen, ThemeName::ALL.len());
    }
}
