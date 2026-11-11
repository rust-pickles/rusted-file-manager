//! Bridges the palette tokens in `rusted-core` to GPUI colours, and derives the
//! density metrics that every surface measures itself against.

use gpui::{rgb, Hsla, Rgba, SharedString};
use rusted_core::{Density, Metrics, ThemeName, ThemeTokens};

pub fn rgba(hex: u32) -> Rgba {
    rgb(hex)
}

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub name: ThemeName,
    pub tokens: ThemeTokens,
}

impl Theme {
    pub fn new(name: ThemeName, animate: bool) -> Self {
        let _ = animate;
        Self { name, tokens: name.tokens() }
    }

    pub fn is_dark(&self) -> bool {
        self.tokens.is_dark
    }

    fn token(&self, value: u32) -> Hsla {
        let mut color: Hsla = rgba(value).into();
        // Palettes are authored at full opacity; a fractional alpha here would
        // make every surface translucent over the window backdrop.
        color.a = 1.0;
        color
    }

    // Surfaces
    pub fn background(&self) -> Hsla {
        self.token(self.tokens.background)
    }
    pub fn sidebar(&self) -> Hsla {
        self.token(self.tokens.sidebar)
    }
    pub fn panel(&self) -> Hsla {
        self.token(self.tokens.panel)
    }
    pub fn elevated(&self) -> Hsla {
        self.token(self.tokens.elevated)
    }
    pub fn hover(&self) -> Hsla {
        self.token(self.tokens.hover)
    }
    pub fn border(&self) -> Hsla {
        self.token(self.tokens.border)
    }
    pub fn selection(&self) -> Hsla {
        self.token(self.tokens.selection)
    }
    pub fn overlay(&self) -> Hsla {
        self.token(self.tokens.overlay)
    }

    // Text
    pub fn text(&self) -> Hsla {
        self.token(self.tokens.text)
    }
    pub fn secondary(&self) -> Hsla {
        self.token(self.tokens.secondary)
    }
    pub fn muted(&self) -> Hsla {
        self.token(self.tokens.muted)
    }
    pub fn accent(&self) -> Hsla {
        self.token(self.tokens.accent)
    }

    // Status
    pub fn danger(&self) -> Hsla {
        self.token(self.tokens.danger)
    }
    pub fn success(&self) -> Hsla {
        self.token(self.tokens.success)
    }
    pub fn warning(&self) -> Hsla {
        self.token(self.tokens.warning)
    }

    /// Selection fill, kept opaque so text stays legible on top of it.
    pub fn selection_fill(&self) -> Hsla {
        self.selection()
    }

    pub fn hover_fill(&self) -> Hsla {
        self.hover()
    }

    pub fn translucent(&self, color: Hsla, alpha: f32) -> Hsla {
        let mut value = color;
        value.a = alpha;
        value
    }

    /// A dimmed backdrop used behind modal dialogs.
    pub fn scrim(&self) -> Hsla {
        let mut color = self.overlay();
        color.a = if self.is_dark() { 0.72 } else { 0.45 };
        color
    }

    pub fn radii() -> Radii {
        Radii {
            button: Metrics::RADIUS_BUTTON,
            input: Metrics::RADIUS_INPUT,
            tab: Metrics::RADIUS_TAB,
            panel: Metrics::RADIUS_PANEL,
            dialog: Metrics::RADIUS_DIALOG,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Radii {
    pub button: f32,
    pub input: f32,
    pub tab: f32,
    pub panel: f32,
    pub dialog: f32,
}

/// Row height and paddings derived from the density setting.
#[derive(Debug, Clone, Copy)]
pub struct DensityMetrics {
    pub row_height: f32,
    pub row_padding: f32,
    pub toolbar_height: f32,
    pub tab_height: f32,
    pub font_size: f32,
}

impl DensityMetrics {
    pub fn new(density: Density, root_font_size: u16) -> Self {
        let base = root_font_size.max(11) as f32;
        match density {
            Density::Compact => Self {
                row_height: 22.0,
                row_padding: 4.0,
                toolbar_height: 34.0,
                tab_height: 26.0,
                font_size: base - 1.0,
            },
            Density::Comfortable => Self {
                row_height: 28.0,
                row_padding: 8.0,
                toolbar_height: 40.0,
                tab_height: 30.0,
                font_size: base,
            },
        }
    }
}

/// Icon file names, resolved through the asset source in `shell::Assets`.
pub fn folder_icon() -> SharedString {
    SharedString::from("folder.png")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_theme_resolves_to_opaque_gpui_colours() {
        for name in ThemeName::ALL {
            let theme = Theme::new(name, true);
            assert_eq!(theme.background().a, 1.0);
            assert_eq!(theme.text().a, 1.0);
            assert_ne!(theme.background(), theme.text());
            assert!(theme.scrim().a < 1.0);
            assert_eq!(theme.is_dark(), name.tokens().is_dark);
        }
    }

    #[test]
    fn density_metrics_scale_with_the_setting() {
        let compact = DensityMetrics::new(Density::Compact, 13);
        let comfortable = DensityMetrics::new(Density::Comfortable, 13);
        assert!(compact.row_height < comfortable.row_height);
        assert!(compact.font_size < comfortable.font_size);
        assert!(compact.toolbar_height > compact.row_height);
    }

    #[test]
    fn radii_come_from_the_shared_metrics() {
        let radii = Theme::radii();
        assert_eq!(radii.panel, Metrics::RADIUS_PANEL);
        assert!(radii.dialog > radii.button);
    }
}
