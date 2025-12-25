use gpui::{
    App, Bounds, Corner, InteractiveElement, IntoElement, ParentElement, Pixels, Point,
    RenderOnce, Styled, Window, anchored, deferred, div, prelude::FluentBuilder, px,
};

use gpui_component::{ActiveTheme, h_flex, scroll::ScrollableElement, v_flex};

use agent_client_protocol::AvailableCommand;

/// A popover component that displays command suggestions above an anchor element.
///
/// Features:
/// - Displays a list of available commands with names and descriptions
/// - Positioned above the anchor element
/// - Auto-adjusts to window boundaries
/// - Styled with theme colors
#[derive(IntoElement)]
pub struct CommandSuggestionsPopover {
    /// The bounds of the anchor element (typically the input box)
    anchor_bounds: Option<Bounds<Pixels>>,
    /// List of commands to display
    commands: Vec<AvailableCommand>,
    /// Whether the popover should be visible
    visible: bool,
    /// Optional click handler for command selection
    on_select: Option<Box<dyn Fn(&AvailableCommand, &mut Window, &mut App) + 'static>>,
}

impl CommandSuggestionsPopover {
    /// Create a new CommandSuggestionsPopover
    pub fn new(commands: Vec<AvailableCommand>) -> Self {
        Self {
            anchor_bounds: None,
            commands,
            visible: true,
            on_select: None,
        }
    }

    /// Set the anchor bounds for positioning the popover
    pub fn anchor_bounds(mut self, bounds: Option<Bounds<Pixels>>) -> Self {
        self.anchor_bounds = bounds;
        self
    }

    /// Set whether the popover is visible
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Set a callback for when a command is selected
    pub fn on_select<F>(mut self, callback: F) -> Self
    where
        F: Fn(&AvailableCommand, &mut Window, &mut App) + 'static,
    {
        self.on_select = Some(Box::new(callback));
        self
    }
}

impl RenderOnce for CommandSuggestionsPopover {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        // Early return if not visible or no commands
        if !self.visible || self.commands.is_empty() {
            return div().into_any_element();
        }

        // Get theme
        let theme = cx.theme();

        // Calculate position based on anchor bounds
        match self.anchor_bounds {
            Some(bounds) => {
                let position = bounds.corner(Corner::TopLeft)
                    + Point {
                        x: px(0.),
                        y: -px(8.),
                    };

                let command_count = self.commands.len();

                deferred(
                    anchored()
                        .snap_to_window_with_margin(px(8.))
                        .anchor(Corner::BottomLeft)
                        .position(position)
                        .child(
                            v_flex()
                                // .occlude()
                                .w(bounds.size.width)
                                .gap_2()
                                .p_3()
                                .rounded(px(12.))
                                .border_1()
                                .border_color(theme.border)
                                .bg(theme.popover)
                                .shadow_lg()
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(theme.muted_foreground)
                                        .child("Available Commands:"),
                                )
                                .max_h(px(140.0))
                                .overflow_y_hidden()
                                .overflow_y_scrollbar()
                                .children(
                                    self.commands
                                        .into_iter()
                                        .enumerate()
                                        .map(|(idx, command)| {
                                            let row = h_flex()
                                                .w_full()
                                                .gap_3()
                                                .items_center()
                                                .py_1()
                                                .child(
                                                    div()
                                                        .w(px(140.))
                                                        .text_sm()
                                                        .font_family(
                                                            "Monaco, 'Courier New', monospace",
                                                        )
                                                        .text_color(theme.popover_foreground)
                                                        .child(format!("/{}", command.name)),
                                                )
                                                .child(
                                                    div()
                                                        .flex_1()
                                                        .text_sm()
                                                        .text_color(theme.muted_foreground)
                                                        .overflow_x_hidden()
                                                        .text_ellipsis()
                                                        .child(command.description),
                                                );

                                            // Add border between items except for the last one
                                            row.when(idx + 1 < command_count, |row| {
                                                row.border_b_1().border_color(theme.border)
                                            })
                                        }),
                                ),
                        ),
                )
                .with_priority(1)
                .into_any_element()
            }
            None => div().into_any_element(),
        }
    }
}
