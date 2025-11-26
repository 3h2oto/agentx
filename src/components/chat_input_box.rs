use gpui::{
    div, prelude::FluentBuilder, px, App, ElementId, Entity, IntoElement, ParentElement,
    RenderOnce, Styled, Window,
};

use gpui_component::{
    button::{Button, ButtonCustomVariant, ButtonVariants},
    h_flex,
    input::{Input, InputState},
    v_flex, ActiveTheme, Disableable, Icon, IconName, Sizable,
};

/// A reusable chat input component with context controls and send button.
///
/// Features:
/// - Add context button at the top
/// - Multi-line textarea with auto-grow (2-8 rows)
/// - Action buttons (attach, auto, sources)
/// - Send button with icon
/// - Optional title displayed above the input box
#[derive(IntoElement)]
pub struct ChatInputBox {
    id: ElementId,
    input_state: Entity<InputState>,
    title: Option<String>,
    on_send: Option<Box<dyn Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl ChatInputBox {
    /// Create a new ChatInputBox with the given input state
    pub fn new(id: impl Into<ElementId>, input_state: Entity<InputState>) -> Self {
        Self {
            id: id.into(),
            input_state,
            title: None,
            on_send: None,
        }
    }

    /// Set an optional title to display above the input box
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set a callback for when the send button is clicked
    pub fn on_send<F>(mut self, callback: F) -> Self
    where
        F: Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
    {
        self.on_send = Some(Box::new(callback));
        self
    }
}

impl RenderOnce for ChatInputBox {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let on_send = self.on_send;
        let input_value = self.input_state.read(cx).value();
        let is_empty = input_value.trim().is_empty();

        v_flex()
            .w_full()
            .gap_3()
            .px(px(32.)) // Left and right padding for spacing
            .when_some(self.title, |this, title| {
                this.child(
                    h_flex().w_full().pb_2().child(
                        div()
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child(title),
                    ),
                )
            })
            .child(
                v_flex()
                    .w_full()
                    .gap_3()
                    .p_4()
                    .rounded(px(16.))
                    .border_1()
                    .border_color(theme.border)
                    .bg(theme.secondary)
                    .shadow_lg()
                    .child(
                        // Top row: Add context button
                        h_flex().w_full().child(
                            Button::new("add-context")
                                .label("Add context")
                                .icon(Icon::new(IconName::Asterisk))
                                .ghost()
                                .small(),
                        ),
                    )
                    .child(
                        // Textarea (multi-line input)
                        Input::new(&self.input_state).appearance(false),
                    )
                    .child(
                        // Bottom row: Action buttons
                        h_flex()
                            .w_full()
                            .items_center()
                            .justify_between()
                            .child(
                                h_flex()
                                    .gap_2()
                                    .items_center()
                                    .child(
                                        Button::new("attach")
                                            .icon(Icon::new(IconName::Asterisk))
                                            .ghost()
                                            .small(),
                                    )
                                    .child(Button::new("auto").label("Auto").ghost().small())
                                    .child(
                                        Button::new("sources")
                                            .label("All Sources")
                                            .icon(Icon::new(IconName::Globe))
                                            .ghost()
                                            .small(),
                                    ),
                            )
                            .child({
                                let mut btn = Button::new("send")
                                    .icon(Icon::new(IconName::ArrowUp))
                                    .rounded_full()
                                    .small()
                                    .disabled(is_empty);

                                // Set button colors based on empty state
                                if is_empty {
                                    // Disabled state: lighter/muted color
                                    btn = btn.custom(
                                        ButtonCustomVariant::new(cx)
                                            .color(theme.muted.opacity(0.5))
                                            .foreground(theme.muted_foreground.opacity(0.5)),
                                    );
                                } else {
                                    // Enabled state: primary color with hover effect
                                    btn = btn.custom(
                                        ButtonCustomVariant::new(cx)
                                            .color(theme.primary)
                                            .foreground(theme.background)
                                            .hover(theme.primary.opacity(0.85)),
                                    );
                                }

                                if let Some(handler) = on_send {
                                    btn = btn.on_click(move |ev, window, cx| {
                                        handler(ev, window, cx);
                                    });
                                }

                                btn
                            }),
                    ),
            )
    }
}
