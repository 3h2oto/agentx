use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum ConversationItem {
    UserMessage {
        id: String,
        data: UserMessageDataSchema,
    },
    AgentMessage {
        id: String,
        data: AgentMessageDataSchema,
    },
    AgentTodoList {
        title: String,
        entries: Vec<PlanEntrySchema>,
    },
    ToolCallGroup {
        items: Vec<ToolCallItemSchema>,
    },
}

#[derive(Debug, Deserialize, Clone)]
pub struct UserMessageDataSchema {
    pub session_id: String,
    pub contents: Vec<MessageContentSchema>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum MessageContentSchema {
    Text { text: String },
    Resource { resource: ResourceContentSchema },
}

#[derive(Debug, Deserialize, Clone)]
pub struct ResourceContentSchema {
    pub uri: String,
    pub mime_type: String,
    pub text: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AgentMessageDataSchema {
    pub session_id: String,
    pub agent_name: Option<String>,
    pub chunks: Vec<AgentMessageContentSchema>,
    pub is_complete: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AgentMessageContentSchema {
    pub content_type: String, // "Text", etc.
    pub text: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PlanEntrySchema {
    pub content: String,
    pub priority: String,
    pub status: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ToolCallItemSchema {
    pub id: String,
    pub data: ToolCallDataSchema,
    pub open: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ToolCallDataSchema {
    pub tool_call_id: String,
    pub title: String,
    pub kind: String,
    pub status: String,
    pub content: Vec<ToolCallContentSchema>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ToolCallContentSchema {
    pub text: String,
}
