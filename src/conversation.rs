use gpui::{
    px, App, AppContext, Context, ElementId, Entity, FocusHandle, Focusable, IntoElement,
    ParentElement, Pixels, Render, Styled, Window,
};

use gpui_component::{scroll::ScrollbarAxis, v_flex, ActiveTheme, StyledExt};

use crate::{
    conversation_schema::{
        AgentMessageDataSchema, ConversationItem, MessageContentSchema, PlanEntrySchema,
        ToolCallItemSchema, UserMessageDataSchema,
    },
    AgentMessage, AgentMessageContent, AgentMessageData, AgentTodoList, MessageContent, PlanEntry,
    PlanEntryPriority, PlanEntryStatus, ResourceContent, ToolCallContent, ToolCallData,
    ToolCallItem, ToolCallKind, ToolCallStatus, UserMessage, UserMessageData,
};

pub struct ConversationPanel {
    focus_handle: FocusHandle,
    items: Vec<ConversationItem>,
}

impl super::DockPanel for ConversationPanel {
    fn title() -> &'static str {
        "Conversation"
    }

    fn description() -> &'static str {
        "A conversation view with agent messages, user messages, tool calls, and todos."
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render> {
        Self::view(window, cx)
    }

    fn paddings() -> Pixels {
        px(0.)
    }
}

impl ConversationPanel {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    fn new(_: &mut Window, cx: &mut App) -> Self {
        let json_content = include_str!("fixtures/mock_conversation.json");
        let items: Vec<ConversationItem> =
            serde_json::from_str(json_content).expect("Failed to parse mock conversation");

        Self {
            focus_handle: cx.focus_handle(),
            items,
        }
    }

    fn get_id(id: &str) -> ElementId {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        id.hash(&mut hasher);
        ElementId::from(("item", hasher.finish()))
    }

    fn map_user_message(id: String, data: UserMessageDataSchema) -> UserMessage {
        let mut user_data = UserMessageData::new(data.session_id);
        for content in data.contents {
            match content {
                MessageContentSchema::Text { text } => {
                    user_data = user_data.add_content(MessageContent::text(text));
                }
                MessageContentSchema::Resource { resource } => {
                    user_data = user_data.add_content(MessageContent::resource(
                        ResourceContent::new(resource.uri, resource.mime_type, resource.text),
                    ));
                }
            }
        }
        UserMessage::new(Self::get_id(&id), user_data)
    }

    fn map_agent_message(id: String, data: AgentMessageDataSchema) -> AgentMessage {
        let mut agent_data = AgentMessageData::new(data.session_id);
        if let Some(name) = data.agent_name {
            agent_data = agent_data.with_agent_name(name);
        }
        if data.is_complete {
            agent_data = agent_data.complete();
        }
        for chunk in data.chunks {
            // Assuming all chunks are text for now as per schema
            agent_data = agent_data.add_chunk(AgentMessageContent::text(chunk.text));
        }
        AgentMessage::new(Self::get_id(&id), agent_data)
    }

    fn map_todo_list(title: String, entries: Vec<PlanEntrySchema>) -> AgentTodoList {
        let plan_entries = entries
            .into_iter()
            .map(|e| {
                let priority = match e.priority.as_str() {
                    "High" => PlanEntryPriority::High,
                    "Medium" => PlanEntryPriority::Medium,
                    "Low" => PlanEntryPriority::Low,
                    _ => PlanEntryPriority::Medium,
                };
                let status = match e.status.as_str() {
                    "Pending" => PlanEntryStatus::Pending,
                    "InProgress" => PlanEntryStatus::InProgress,
                    "Completed" => PlanEntryStatus::Completed,
                    _ => PlanEntryStatus::Pending,
                };
                PlanEntry::new(e.content)
                    .with_priority(priority)
                    .with_status(status)
            })
            .collect();

        AgentTodoList::new().title(title).entries(plan_entries)
    }

    fn map_tool_call(item: ToolCallItemSchema) -> ToolCallItem {
        let kind = ToolCallKind::from_str(&item.data.kind.to_lowercase());
        let status = match item.data.status.as_str() {
            "Pending" => ToolCallStatus::Pending,
            "InProgress" => ToolCallStatus::InProgress,
            "Completed" => ToolCallStatus::Completed,
            "Failed" => ToolCallStatus::Failed,
            _ => ToolCallStatus::Pending,
        };

        let content = item
            .data
            .content
            .into_iter()
            .map(|c| ToolCallContent::new(c.text))
            .collect();

        let data = ToolCallData::new(item.data.tool_call_id, item.data.title)
            .with_kind(kind)
            .with_status(status)
            .with_content(content);

        ToolCallItem::new(Self::get_id(&item.id), data).open(item.open)
    }
}

impl Focusable for ConversationPanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ConversationPanel {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut children = v_flex().p_4().gap_6().bg(cx.theme().background);

        for item in &self.items {
            match item {
                ConversationItem::UserMessage { id, data } => {
                    let user_msg = Self::map_user_message(id.clone(), data.clone());
                    children = children.child(user_msg);
                }
                ConversationItem::AgentMessage { id, data } => {
                    let agent_msg = Self::map_agent_message(id.clone(), data.clone());
                    children = children.child(agent_msg);
                }
                ConversationItem::AgentTodoList { title, entries } => {
                    let todo_list = Self::map_todo_list(title.clone(), entries.clone());
                    // Apply indentation for todo list
                    children = children.child(v_flex().pl_6().child(todo_list));
                }
                ConversationItem::ToolCallGroup { items } => {
                    let mut group = v_flex().pl_6().gap_2();
                    for tool_item in items {
                        let tool_call = Self::map_tool_call(tool_item.clone());
                        group = group.child(tool_call);
                    }
                    children = children.child(group);
                }
            }
        }

        children.scrollable(ScrollbarAxis::Vertical).size_full()
    }
}
