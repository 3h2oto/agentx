use gpui::{
    px, App, AppContext, Context, ElementId, Entity, FocusHandle, Focusable, IntoElement,
    ParentElement, Pixels, Render, Styled, Window,
};

use agent_client_protocol_schema::{
    BlobResourceContents, Content, ContentBlock, ContentChunk, EmbeddedResource,
    EmbeddedResourceResource, ImageContent, Plan, PlanEntry, PlanEntryPriority, PlanEntryStatus,
    ResourceLink, TextContent, TextResourceContents, ToolCall, ToolCallContent, ToolCallId,
    ToolCallStatus, ToolKind,
};
use gpui_component::{scroll::ScrollbarAxis, v_flex, ActiveTheme, StyledExt};

use crate::{
    conversation_schema::{
        AgentMessageDataSchema, ContentBlockSchema, ConversationItem, PlanEntrySchema, PlanSchema,
        ResourceContentsSchema, ToolCallContentItemSchema, ToolCallItemSchema, ToolCallSchema,
        UserMessageDataSchema,
    },
    AgentMessage, AgentMessageData, AgentMessageMeta, AgentTodoList, ToolCallItem, UserMessage,
    UserMessageData, UserMessageView,
};

pub struct ConversationPanel {
    focus_handle: FocusHandle,
    items: Vec<ConversationItem>,
}

impl crate::dock_panel::DockPanel for ConversationPanel {
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

    fn map_user_message(id: String, data: UserMessageDataSchema, cx: &mut Context<Self>) -> Entity<UserMessageView> {
        let mut user_data = UserMessageData::new(data.session_id);

        // Convert content blocks from schema to ACP types
        for content_schema in data.prompt {
            let content_block = Self::map_content_block(content_schema);
            user_data.contents.push(content_block);
        }

        // Create UserMessageView entity
        cx.new(|cx| {
            let data_entity = cx.new(|_| user_data.clone());

            // Create ResourceItem entities for each resource in the data
            let resource_items: Vec<Entity<crate::components::ResourceItem>> = user_data
                .contents
                .iter()
                .filter_map(|content| crate::components::get_resource_info(content))
                .map(|resource_info| cx.new(|_| crate::components::ResourceItem::new(resource_info)))
                .collect();

            UserMessageView {
                data: data_entity,
                resource_items,
            }
        })
    }

    /// Convert schema ContentBlock to ACP ContentBlock
    fn map_content_block(schema: ContentBlockSchema) -> ContentBlock {
        match schema {
            ContentBlockSchema::Text(text) => ContentBlock::Text(TextContent::new(text.text)),
            ContentBlockSchema::Image(image) => {
                ContentBlock::Image(ImageContent::new(image.data, image.mime_type))
            }
            ContentBlockSchema::ResourceLink(link) => {
                let mut resource_link = ResourceLink::new(link.name, link.uri);
                if let Some(mime) = link.mime_type {
                    resource_link = resource_link.mime_type(mime);
                }
                ContentBlock::ResourceLink(resource_link)
            }
            ContentBlockSchema::Resource(embedded) => {
                let resource = match embedded.resource {
                    ResourceContentsSchema::TextResourceContents(text_res) => {
                        let mut content = TextResourceContents::new(text_res.text, text_res.uri);
                        if let Some(mime) = text_res.mime_type {
                            content = content.mime_type(mime);
                        }
                        EmbeddedResourceResource::TextResourceContents(content)
                    }
                    ResourceContentsSchema::BlobResourceContents(blob_res) => {
                        let mut content = BlobResourceContents::new(blob_res.blob, blob_res.uri);
                        if let Some(mime) = blob_res.mime_type {
                            content = content.mime_type(mime);
                        }
                        EmbeddedResourceResource::BlobResourceContents(content)
                    }
                };
                ContentBlock::Resource(EmbeddedResource::new(resource))
            }
        }
    }

    fn map_agent_message(id: String, data: AgentMessageDataSchema) -> AgentMessage {
        let mut agent_data = AgentMessageData::new(data.session_id);

        // Set metadata from _meta field
        if let Some(meta) = data.meta {
            agent_data.meta = AgentMessageMeta {
                agent_name: meta.agent_name,
                is_complete: meta.is_complete,
            };
        }

        // Convert content chunks
        for chunk_schema in data.chunks {
            let content_block = Self::map_content_block(chunk_schema.content);
            let mut content_chunk = ContentChunk::new(content_block);
            if let Some(meta) = chunk_schema.meta {
                content_chunk = content_chunk.meta(meta);
            }
            agent_data.chunks.push(content_chunk);
        }

        AgentMessage::new(Self::get_id(&id), agent_data)
    }

    fn map_plan(plan_schema: PlanSchema) -> AgentTodoList {
        // Convert PlanEntrySchema to ACP PlanEntry
        let plan_entries: Vec<PlanEntry> = plan_schema
            .entries
            .into_iter()
            .map(|e| Self::map_plan_entry(e))
            .collect();

        // Create ACP Plan
        let mut plan = Plan::new(plan_entries);

        // Copy meta from schema if present
        if let Some(meta) = plan_schema.meta {
            plan.meta = Some(meta);
        }

        AgentTodoList::from_plan(plan)
    }

    fn map_plan_entry(entry: PlanEntrySchema) -> PlanEntry {
        let priority = match entry.priority.to_lowercase().as_str() {
            "high" => PlanEntryPriority::High,
            "medium" => PlanEntryPriority::Medium,
            "low" => PlanEntryPriority::Low,
            _ => PlanEntryPriority::Medium,
        };
        let status = match entry.status.to_lowercase().as_str() {
            "pending" => PlanEntryStatus::Pending,
            "in_progress" => PlanEntryStatus::InProgress,
            "completed" => PlanEntryStatus::Completed,
            _ => PlanEntryStatus::Pending,
        };

        let mut plan_entry = PlanEntry::new(entry.content, priority, status);
        if let Some(meta) = entry.meta {
            plan_entry = plan_entry.meta(meta);
        }
        plan_entry
    }

    fn map_tool_call(item: ToolCallItemSchema, cx: &mut Context<Self>) -> Entity<ToolCallItem> {
        let kind = item
            .data
            .kind
            .as_deref()
            .and_then(|k| match k.to_lowercase().as_str() {
                "read" => Some(ToolKind::Read),
                "edit" => Some(ToolKind::Edit),
                "delete" => Some(ToolKind::Delete),
                "move" => Some(ToolKind::Move),
                "search" => Some(ToolKind::Search),
                "execute" => Some(ToolKind::Execute),
                "think" => Some(ToolKind::Think),
                "fetch" => Some(ToolKind::Fetch),
                "switch_mode" => Some(ToolKind::SwitchMode),
                _ => Some(ToolKind::Other),
            })
            .unwrap_or(ToolKind::Other);

        let status = item
            .data
            .status
            .as_deref()
            .and_then(|s| match s.to_lowercase().as_str() {
                "pending" => Some(ToolCallStatus::Pending),
                "in_progress" | "inprogress" => Some(ToolCallStatus::InProgress),
                "completed" => Some(ToolCallStatus::Completed),
                "failed" => Some(ToolCallStatus::Failed),
                _ => Some(ToolCallStatus::Pending),
            })
            .unwrap_or(ToolCallStatus::Pending);

        // Convert simple text content to ACP ToolCallContent
        let content: Vec<ToolCallContent> = item
            .data
            .content
            .into_iter()
            .map(|c| ToolCallContent::Content(Content::new(ContentBlock::from(c.text))))
            .collect();

        let tool_call = ToolCall::new(
            ToolCallId::new(item.data.tool_call_id),
            item.data.title,
        )
        .kind(kind)
        .status(status)
        .content(content);

        let is_open = item.open;
        cx.new(|cx| {
            let mut tool_item = ToolCallItem::new(tool_call);
            tool_item.set_open(is_open, cx);
            tool_item
        })
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
                    let user_msg = Self::map_user_message(id.clone(), data.clone(), cx);
                    children = children.child(user_msg);
                }
                ConversationItem::AgentMessage { id, data } => {
                    let agent_msg = Self::map_agent_message(id.clone(), data.clone());
                    children = children.child(agent_msg);
                }
                ConversationItem::Plan(plan_schema) => {
                    let todo_list = Self::map_plan(plan_schema.clone());
                    // Apply indentation for todo list
                    children = children.child(v_flex().pl_6().child(todo_list));
                }
                ConversationItem::ToolCallGroup { items } => {
                    let mut group = v_flex().pl_6().gap_2();
                    for tool_item in items {
                        let tool_call = Self::map_tool_call(tool_item.clone(), cx);
                        group = group.child(tool_call);
                    }
                    children = children.child(group);
                }
            }
        }

        children.scrollable(ScrollbarAxis::Vertical).size_full()
    }
}
