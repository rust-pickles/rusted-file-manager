//! Small reusable GPUI pieces shared by every panel: buttons, chips, labels,
//! cards, empty states and progress bars.

use crate::theme::{DensityMetrics, Radii, Theme};
use gpui::prelude::*;
use gpui::{div, px, AnyElement, DefiniteLength, Div, ElementId, FontWeight, Hsla, IntoElement, SharedString, Stateful};

/// Immutable render context handed down to every panel builder.
#[derive(Clone, Copy)]
pub struct Ui {
    pub theme: Theme,
    pub density: DensityMetrics,
    pub animations: bool,
}

impl Ui {
    pub fn new(theme: Theme, density: DensityMetrics, animations: bool) -> Self {
        Self { theme, density, animations }
    }

    pub fn panel(&self) -> Div {
        div()
            .bg(self.theme.panel())
            .border_1()
            .border_color(self.theme.border())
            .rounded(px(Radii::panel))
    }

    pub fn card(&self) -> Div {
        div()
            .bg(self.theme.elevated())
            .border_1()
            .border_color(self.theme.border())
            .rounded(px(Radii::panel))
    }

    pub fn label(&self, text: impl Into<SharedString>) -> Div {
        div()
            .text_size(px(self.density.font_size))
            .text_color(self.theme.text())
            .child(text.into())
    }

    pub fn secondary(&self, text: impl Into<SharedString>) -> Div {
        div()
            .text_size(px(self.density.font_size - 1.0))
            .text_color(self.theme.secondary())
            .child(text.into())
    }

    pub fn muted(&self, text: impl Into<SharedString>) -> Div {
        div()
            .text_size(px(self.density.font_size - 2.0))
            .text_color(self.theme.muted())
            .child(text.into())
    }

    /// Section caption inside the sidebar and inspector.
    pub fn caption(&self, text: impl Into<SharedString>) -> Div {
        div()
            .text_size(px(self.density.font_size - 3.0))
            .text_color(self.theme.muted())
            .child(text.into())
    }

    /// Toolbar / icon button. `disabled` only dims and removes the hover; the
    /// caller is responsible for not attaching a handler.
    pub fn icon_button(
        &self,
        id: impl Into<ElementId>,
        symbol: impl Into<SharedString>,
        active: bool,
        disabled: bool,
    ) -> Stateful<Div> {
        let theme = self.theme;
        let text = if disabled {
            theme.muted()
        } else if active {
            theme.text()
        } else {
            theme.secondary()
        };
        let mut button = div()
            .id(id)
            .flex()
            .items_center()
            .justify_center()
            .h(px(self.density.toolbar_height - 14.0))
            .min_w(px(self.density.toolbar_height - 8.0))
            .px_2()
            .rounded(px(Radii::button))
            .text_size(px(self.density.font_size))
            .text_color(text)
            .child(symbol.into());
        if active {
            button = button.bg(theme.selection());
        }
        if !disabled {
            button = button.cursor_pointer().hover(move |style| style.bg(theme.hover()));
        } else {
            button = button.opacity(0.45);
        }
        button
    }

    pub fn text_button(&self, id: impl Into<ElementId>, label: impl Into<SharedString>, primary: bool) -> Stateful<Div> {
        let theme = self.theme;
        let background = if primary { theme.accent() } else { theme.elevated() };
        let text = if primary { theme.background() } else { theme.text() };
        let border = if primary { theme.accent() } else { theme.border() };
        div()
            .id(id)
            .flex()
            .items_center()
            .justify_center()
            .h(px(self.density.toolbar_height - 10.0))
            .px_3()
            .rounded(px(Radii::button))
            .bg(background)
            .border_1()
            .border_color(border)
            .text_size(px(self.density.font_size))
            .text_color(text)
            .cursor_pointer()
            .hover(move |style| style.opacity(0.88))
            .child(label.into())
    }

    /// Button that is visibly unavailable and carries no handler.
    pub fn disabled_button(&self, id: impl Into<ElementId>, label: impl Into<SharedString>) -> Stateful<Div> {
        div()
            .id(id)
            .flex()
            .items_center()
            .justify_center()
            .h(px(self.density.toolbar_height - 10.0))
            .px_3()
            .rounded(px(Radii::button))
            .bg(self.theme.elevated())
            .border_1()
            .border_color(self.theme.border())
            .opacity(0.4)
            .text_size(px(self.density.font_size))
            .text_color(self.theme.muted())
            .child(label.into())
    }

    pub fn chip(&self, label: impl Into<SharedString>, active: bool) -> Div {
        let theme = self.theme;
        div()
            .px_2()
            .py_1()
            .rounded(px(Radii::input))
            .text_size(px(self.density.font_size - 1.0))
            .when(active, |this| this.bg(theme.selection()).text_color(theme.text()))
            .when(!active, |this| this.bg(theme.elevated()).text_color(theme.secondary()))
            .child(label.into())
    }

    pub fn separator(&self) -> Div {
        div().w(px(1.0)).h(px(self.density.toolbar_height - 18.0)).bg(self.theme.border()).mx_1()
    }

    pub fn rule(&self) -> Div {
        div().h(px(1.0)).w_full().bg(self.theme.border()).my_1()
    }

    pub fn empty_state(&self, title: impl Into<SharedString>, detail: impl Into<SharedString>) -> AnyElement {
        div()
            .flex()
            .flex_col()
            .gap_2()
            .items_center()
            .justify_center()
            .size_full()
            .child(self.secondary(title))
            .child(self.muted(detail))
            .into_any_element()
    }

    /// Progress bar that only ever reflects measured values.
    pub fn progress_bar(&self, fraction: f64, color: Hsla) -> Div {
        let width: DefiniteLength = gpui::relative(fraction.clamp(0.0, 1.0) as f32).into();
        div()
            .h(px(4.0))
            .w_full()
            .rounded_full()
            .bg(self.theme.border())
            .child(div().h_full().rounded_full().bg(color).w(width))
    }

    /// Key/value row used by the inspector and details panel.
    pub fn field(&self, key: &'static str, value: impl Into<SharedString>) -> Div {
        div()
            .flex()
            .flex_col()
            .gap_1()
            .child(self.caption(key.to_uppercase()))
            .child(
                div()
                    .text_size(px(self.density.font_size - 1.0))
                    .text_color(self.theme.secondary())
                    .child(value.into()),
            )
    }

    pub fn badge(&self, label: impl Into<SharedString>, color: Hsla) -> Div {
        div()
            .px_2()
            .rounded_full()
            .bg(self.theme.translucent(color, 0.16))
            .text_size(px(self.density.font_size - 3.0))
            .text_color(color)
            .child(label.into())
    }

    pub fn strong(&self, text: impl Into<SharedString>) -> Div {
        self.label(text).font_weight(FontWeight::SEMIBOLD)
    }
}

/// Full-window dimmer used behind dialogs and menus.
pub fn scrim(theme: Theme) -> Div {
    div().absolute().inset_0().bg(theme.scrim())
}
