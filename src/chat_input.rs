use gpui::{
    px, App, AppContext, Context, Entity, FocusHandle, Focusable, IntoElement, ParentElement,
    Pixels, Render, Styled, Window,
};

use gpui_component::{input::InputState, v_flex, ActiveTheme};

use crate::components::ChatInputBox;

pub struct ChatInputPanel {
    focus_handle: FocusHandle,
    input_state: Entity<InputState>,
}

impl super::DockPanel for ChatInputPanel {
    fn title() -> &'static str {
        "Chat Input"
    }

    fn description() -> &'static str {
        "A chat input box for sending messages."
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render> {
        Self::view(window, cx)
    }
    fn paddings() -> Pixels {
        px(0.)
    }
}

impl ChatInputPanel {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    fn new(window: &mut Window, cx: &mut App) -> Self {
        let input_state = cx.new(|cx| {
            InputState::new(window, cx)
                .auto_grow(2, 8) // Auto-grow from 2 to 8 rows
                .soft_wrap(true) // Enable word wrapping
                .placeholder("Ask, search, or make anything...")
        });

        Self {
            focus_handle: cx.focus_handle(),
            input_state,
        }
    }
}

impl Focusable for ChatInputPanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ChatInputPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .justify_end()
            .bg(cx.theme().background)
            .child(
                ChatInputBox::new("chat-input-box", self.input_state.clone())
                    .title("Send a message"),
            )
    }
}
