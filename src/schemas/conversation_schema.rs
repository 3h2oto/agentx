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

/// Agent message data schema aligned with ACP's ContentChunk format
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AgentMessageDataSchema {
    pub session_id: String,
    /// Content chunks following ACP ContentChunk structure
    pub chunks: Vec<ContentChunkSchema>,
    /// Extended metadata (agent_name, is_complete stored in _meta)
    #[serde(rename = "_meta")]
    pub meta: Option<AgentMessageMetaSchema>,
}

/// Content chunk schema aligned with ACP's ContentChunk
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContentChunkSchema {
    /// Content block following ACP's ContentBlock structure
    pub content: ContentBlockSchema,
    /// Extension point for implementations
    #[serde(rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

/// Content block schema aligned with ACP's ContentBlock enum
#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlockSchema {
    Text(TextContentSchema),
    Image(ImageContentSchema),
    // Add other content types as needed
}

/// Text content schema
#[derive(Debug, Deserialize, Clone)]
pub struct TextContentSchema {
    pub text: String,
    #[serde(rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

/// Image content schema
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImageContentSchema {
    pub data: String,
    pub mime_type: String,
    #[serde(rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

/// Extended metadata for agent messages
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AgentMessageMetaSchema {
    #[serde(default)]
    pub agent_name: Option<String>,
    #[serde(default)]
    pub is_complete: bool,
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
