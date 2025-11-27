use gpui::{
    div, prelude::*, App, Context, ElementId, Entity, FocusHandle, Focusable, IntoElement,
    ParentElement, Render, Styled, Window, AnyElement,
};
use gpui_component::{
    v_flex,
    scroll::{Scrollbar, ScrollbarState},
    theme::ActiveTheme,
};

// Use the published ACP schema crate
use agent_client_protocol_schema::{
    ContentBlock, ContentChunk, EmbeddedResourceResource, Plan, PlanEntryPriority,
    PlanEntryStatus, SessionUpdate, ToolCall, ToolCallStatus,
};

use crate::{
    dock_panel::DockPanel, AgentMessage, AgentMessageData, AgentTodoList, ToolCallItem,
    UserMessage, UserMessageData,
};

/// Conversation panel that displays SessionUpdate messages from ACP
pub struct ConversationPanelAcp {
    focus_handle: FocusHandle,
    scrollbar_state: Entity<ScrollbarState>,
    /// List of session updates from the agent
    session_updates: Vec<SessionUpdate>,
}

impl ConversationPanelAcp {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    fn new(_window: &mut Window, cx: &mut App) -> Self {
        let focus_handle = cx.focus_handle();

        Self {
            focus_handle,
            scrollbar_state: cx.new(|_cx| ScrollbarState::default()),
            session_updates: Self::load_mock_data(),
        }
    }

    /// Load mock session updates from JSON file
    fn load_mock_data() -> Vec<SessionUpdate> {
        let json_str = include_str!("../mock_conversation_acp.json");
        match serde_json::from_str::<Vec<SessionUpdate>>(json_str) {
            Ok(updates) => updates,
            Err(e) => {
                eprintln!("Failed to load mock conversation data: {}", e);
                Vec::new()
            }
        }
    }

    /// Render a single session update
    fn render_session_update(
        &self,
        update: &SessionUpdate,
        index: usize,
        cx: &Context<Self>,
    ) -> AnyElement {
        match update {
            SessionUpdate::UserMessageChunk(chunk) => {
                self.render_user_message_chunk(chunk, index, cx)
            }
            SessionUpdate::AgentMessageChunk(chunk) => {
                self.render_agent_message_chunk(chunk, index, cx)
            }
            SessionUpdate::AgentThoughtChunk(chunk) => {
                self.render_agent_thought_chunk(chunk, cx)
            }
            SessionUpdate::ToolCall(tool_call) => self.render_tool_call(tool_call, cx),
            SessionUpdate::ToolCallUpdate(tool_call_update) => div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(format!(
                    "Tool Call Update: {}",
                    tool_call_update.tool_call_id
                ))
                .into_any_element(),
            SessionUpdate::Plan(plan) => self.render_plan(plan, cx),
            SessionUpdate::AvailableCommandsUpdate(commands_update) => div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(format!(
                    "Available Commands: {} commands",
                    commands_update.available_commands.len()
                ))
                .into_any_element(),
            SessionUpdate::CurrentModeUpdate(mode_update) => div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(format!("Mode Update: {}", mode_update.current_mode_id))
                .into_any_element(),
            // Handle any future variants
            _ => div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("Unknown session update")
                .into_any_element(),
        }
    }

    fn render_user_message_chunk(
        &self,
        chunk: &ContentChunk,
        index: usize,
        _cx: &Context<Self>,
    ) -> AnyElement {
        let content_vec = vec![chunk.content.clone()];
        let data = UserMessageData::new("default-session").with_contents(content_vec);

        UserMessage::new(ElementId::Name(format!("user-msg-{}", index).into()), data)
            .into_any_element()
    }

    fn render_agent_message_chunk(
        &self,
        chunk: &ContentChunk,
        index: usize,
        _cx: &Context<Self>,
    ) -> AnyElement {
        let data = AgentMessageData::new("default-session").add_chunk(chunk.clone());

        AgentMessage::new(
            ElementId::Name(format!("agent-msg-{}", index).into()),
            data,
        )
        .into_any_element()
    }

    fn render_agent_thought_chunk(
        &self,
        chunk: &ContentChunk,
        cx: &Context<Self>,
    ) -> AnyElement {
        let content_text = self.extract_text_from_content(&chunk.content);

        div()
            .p_3()
            .rounded_lg()
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().muted.opacity(0.3))
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("Thinking..."),
            )
            .child(
                div()
                    .mt_2()
                    .text_sm()
                    .italic()
                    .text_color(cx.theme().foreground.opacity(0.8))
                    .child(content_text),
            )
            .into_any_element()
    }

    fn render_tool_call(&self, tool_call: &ToolCall, cx: &Context<Self>) -> AnyElement {
        // Simplified rendering for tool calls
        div()
            .p_3()
            .rounded_lg()
            .border_1()
            .border_color(cx.theme().border)
            .child(
                div()
                    .text_sm()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .child(format!("Tool: {}", tool_call.title))
            )
            .child(
                div()
                    .mt_2()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!("Status: {:?}", tool_call.status))
            )
            .into_any_element()
    }

    fn render_plan(&self, plan: &Plan, _cx: &Context<Self>) -> AnyElement {
        AgentTodoList::from_plan(plan.clone()).into_any_element()
    }

    /// Extract text from ContentBlock
    fn extract_text_from_content(&self, content: &ContentBlock) -> String {
        match content {
            ContentBlock::Text(text_content) => text_content.text.clone(),
            ContentBlock::Image(img) => {
                format!("[Image: {}]", img.mime_type)
            }
            ContentBlock::Audio(audio) => {
                format!("[Audio: {}]", audio.mime_type)
            }
            ContentBlock::ResourceLink(link) => {
                format!("[Resource: {}]", link.name)
            }
            ContentBlock::Resource(resource) => match &resource.resource {
                EmbeddedResourceResource::TextResourceContents(text_res) => {
                    format!(
                        "[Resource: {}]\n{}",
                        text_res.uri,
                        &text_res.text[..text_res.text.len().min(200)]
                    )
                }
                EmbeddedResourceResource::BlobResourceContents(blob_res) => {
                    format!("[Binary Resource: {}]", blob_res.uri)
                }
                // Handle any future variants
                _ => "[Unknown Resource]".to_string(),
            },
            // Handle any future variants
            _ => "[Unknown Content]".to_string(),
        }
    }
}

impl DockPanel for ConversationPanelAcp {
    fn title() -> &'static str {
        "Conversation (ACP)"
    }

    fn description() -> &'static str {
        "Conversation panel using Agent Client Protocol schema"
    }

    fn closable() -> bool {
        true
    }

    fn zoomable() -> Option<gpui_component::dock::PanelControl> {
        Some(gpui_component::dock::PanelControl::default())
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render> {
        cx.new(|cx| Self::new(window, cx))
    }

    fn on_active_any(view: gpui::AnyView, active: bool, window: &mut Window, cx: &mut App) {
        // Default implementation
        let _ = (view, active, window, cx);
    }
}

impl Focusable for ConversationPanelAcp {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ConversationPanelAcp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let updates: Vec<_> = self
            .session_updates
            .iter()
            .enumerate()
            .map(|(index, update)| self.render_session_update(update, index, cx))
            .collect();

        div()
            .track_focus(&self.focus_handle)
            .flex_1()
            .size_full()
            .bg(cx.theme().background)
            .child(
                v_flex()
                    .gap_4()
                    .p_4()
                    .size_full()
                    .children(updates)
            )
    }
}
