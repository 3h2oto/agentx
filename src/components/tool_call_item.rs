use gpui::{
    div, prelude::FluentBuilder as _, px, App, AppContext, ClickEvent, Context, ElementId, Entity,
    IntoElement, ParentElement, Render, RenderOnce, SharedString, Styled, Window,
};

use agent_client_protocol_schema::{ToolCall, ToolCallContent, ToolCallId, ToolCallStatus, ToolKind};
use gpui_component::{
    button::{Button, ButtonVariants},
    collapsible::Collapsible,
    h_flex, v_flex, ActiveTheme, Icon, IconName, Sizable,
};

/// Helper trait to get icon for ToolKind
pub trait ToolKindExt {
    fn icon(&self) -> IconName;
}

impl ToolKindExt for ToolKind {
    fn icon(&self) -> IconName {
        match self {
            ToolKind::Read => IconName::File,
            ToolKind::Edit => IconName::Replace,
            ToolKind::Delete => IconName::Delete,
            ToolKind::Move => IconName::ArrowRight,
            ToolKind::Search => IconName::Search,
            ToolKind::Execute => IconName::SquareTerminal,
            ToolKind::Think => IconName::Bot,
            ToolKind::Fetch => IconName::Globe,
            ToolKind::SwitchMode => IconName::ArrowRight,
            ToolKind::Other | _ => IconName::Ellipsis,
        }
    }
}

/// Helper trait to get icon for ToolCallStatus
pub trait ToolCallStatusExt {
    fn icon(&self) -> IconName;
}

impl ToolCallStatusExt for ToolCallStatus {
    fn icon(&self) -> IconName {
        match self {
            ToolCallStatus::Pending => IconName::Dash,
            ToolCallStatus::InProgress => IconName::LoaderCircle,
            ToolCallStatus::Completed => IconName::CircleCheck,
            ToolCallStatus::Failed => IconName::CircleX,
            _ => IconName::Dash,
        }
    }
}

/// Helper to extract text from ToolCallContent
fn extract_text_from_content(content: &ToolCallContent) -> Option<String> {
    match content {
        ToolCallContent::Content(c) => match &c.content {
            agent_client_protocol_schema::ContentBlock::Text(text) => Some(text.text.clone()),
            _ => None,
        },
        ToolCallContent::Diff(diff) => Some(format!(
            "Modified: {:?}\n{} -> {}",
            diff.path,
            diff.old_text.as_deref().unwrap_or("<new file>"),
            diff.new_text
        )),
        ToolCallContent::Terminal(terminal) => {
            Some(format!("Terminal: {}", terminal.terminal_id))
        }
        _ => None,
    }
}

/// Tool call item component based on ACP's ToolCall
#[derive(IntoElement)]
pub struct ToolCallItem {
    id: ElementId,
    tool_call: ToolCall,
    open: bool,
    on_toggle: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl ToolCallItem {
    pub fn new(id: impl Into<ElementId>, tool_call: ToolCall) -> Self {
        Self {
            id: id.into(),
            tool_call,
            open: false,
            on_toggle: None,
        }
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn on_toggle(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle = Some(Box::new(handler));
        self
    }

    fn has_content(&self) -> bool {
        !self.tool_call.content.is_empty()
    }
}

impl RenderOnce for ToolCallItem {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let has_content = self.has_content();
        let status_color = match self.tool_call.status {
            ToolCallStatus::Completed => cx.theme().green,
            ToolCallStatus::Failed => cx.theme().red,
            ToolCallStatus::InProgress => cx.theme().accent,
            ToolCallStatus::Pending | _ => cx.theme().muted_foreground,
        };

        let on_toggle = self.on_toggle;
        let id = self.id;
        let open = self.open;

        Collapsible::new()
            .open(open)
            .w_full()
            .gap_2()
            // Header - always visible
            .child(
                h_flex()
                    .items_center()
                    .gap_3()
                    .p_2()
                    .rounded(cx.theme().radius)
                    .bg(cx.theme().secondary)
                    .child(
                        // Kind icon
                        Icon::new(self.tool_call.kind.icon())
                            .size(px(16.))
                            .text_color(cx.theme().muted_foreground),
                    )
                    .child(
                        // Title
                        div()
                            .flex_1()
                            .text_size(px(13.))
                            .text_color(cx.theme().foreground)
                            .child(self.tool_call.title.clone()),
                    )
                    .child(
                        // Status icon
                        Icon::new(self.tool_call.status.icon())
                            .size(px(14.))
                            .text_color(status_color),
                    )
                    .when(has_content, |this| {
                        // Add expand/collapse button only if there's content
                        let btn = Button::new(SharedString::from(format!("{}-toggle", id)))
                            .icon(if open {
                                IconName::ChevronUp
                            } else {
                                IconName::ChevronDown
                            })
                            .ghost()
                            .xsmall();

                        let btn = if let Some(handler) = on_toggle {
                            btn.on_click(move |ev, window, cx| {
                                handler(ev, window, cx);
                            })
                        } else {
                            btn
                        };

                        this.child(btn)
                    }),
            )
            // Content - only visible when open and has content
            .when(has_content, |this| {
                this.content(
                    v_flex()
                        .gap_1()
                        .p_3()
                        .pl_8()
                        .children(self.tool_call.content.iter().filter_map(|content| {
                            extract_text_from_content(content).map(|text| {
                                div()
                                    .text_size(px(12.))
                                    .text_color(cx.theme().muted_foreground)
                                    .line_height(px(18.))
                                    .child(text)
                            })
                        })),
                )
            })
    }
}

/// A stateful wrapper for ToolCallItem that can be used as a GPUI view
pub struct ToolCallItemView {
    tool_call: Entity<ToolCall>,
    open: bool,
}

impl ToolCallItemView {
    pub fn new(tool_call: ToolCall, _window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| {
            let tool_call_entity = cx.new(|_| tool_call);
            Self {
                tool_call: tool_call_entity,
                open: false,
            }
        })
    }

    /// Update the tool call data
    pub fn update_tool_call(&mut self, tool_call: ToolCall, cx: &mut App) {
        self.tool_call.update(cx, |tc, cx| {
            *tc = tool_call;
            cx.notify();
        });
    }

    /// Update the status
    pub fn update_status(&mut self, status: ToolCallStatus, cx: &mut Context<Self>) {
        self.tool_call.update(cx, |tc, cx| {
            tc.status = status;
            cx.notify();
        });
        cx.notify();
    }

    /// Add content to the tool call
    pub fn add_content(&mut self, content: ToolCallContent, cx: &mut Context<Self>) {
        self.tool_call.update(cx, |tc, cx| {
            tc.content.push(content);
            cx.notify();
        });
        cx.notify();
    }

    /// Set content for the tool call
    pub fn set_content(&mut self, content: Vec<ToolCallContent>, cx: &mut Context<Self>) {
        self.tool_call.update(cx, |tc, cx| {
            tc.content = content;
            cx.notify();
        });
        cx.notify();
    }

    /// Toggle the open state
    pub fn toggle(&mut self, cx: &mut Context<Self>) {
        self.open = !self.open;
        cx.notify();
    }

    /// Set the open state
    pub fn set_open(&mut self, open: bool, cx: &mut Context<Self>) {
        self.open = open;
        cx.notify();
    }
}

impl Render for ToolCallItemView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tool_call = self.tool_call.read(cx).clone();
        let id = SharedString::from(format!("tool-call-{}", tool_call.tool_call_id));
        let open = self.open;

        ToolCallItem::new(id, tool_call)
            .open(open)
            .on_toggle(cx.listener(|this, _ev, _window, cx| {
                this.toggle(cx);
            }))
    }
}
