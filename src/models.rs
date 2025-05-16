use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Visuals {
    #[serde(rename = "type")]
    #[serde(default = "default_null_string")]
    pub visual_type: String,
    #[serde(default = "default_null_string")]
    pub svg_content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Question {
    #[serde(default = "default_null_string")]
    pub paragraph: String,
    pub question: String,
    pub choices: Choices,
    pub correct_answer: String,
    pub explanation: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Choices {
    #[serde(rename = "A")]
    pub a: String,
    #[serde(rename = "B")]
    pub b: String,
    #[serde(rename = "C")]
    pub c: String,
    #[serde(rename = "D")]
    pub d: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SATQuestion {
    pub id: String,
    pub domain: String,
    #[serde(default)]
    pub visuals: Visuals,
    pub question: Question,
    pub difficulty: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MathResponse {
    pub math: Vec<SATQuestion>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SlackBlock {
    #[serde(rename = "type")]
    pub block_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<SlackText>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accessory: Option<SlackElement>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elements: Option<Vec<SlackElement>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SlackText {
    #[serde(rename = "type")]
    pub text_type: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emoji: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SlackElement {
    #[serde(rename = "type")]
    pub element_type: String,
    pub text: SlackText,
    pub action_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SlackMessage {
    pub ts: String,
    pub blocks: Vec<SlackMessageBlock>,
}

#[derive(Debug, Deserialize)]
pub struct SlackSlashCommand {
    pub channel_id: String,
    pub response_url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SlackInteraction {
    #[serde(rename = "type")]
    pub interaction_type: String,
    pub user: SlackUser,
    pub actions: Option<Vec<SlackAction>>,
    pub response_url: String,
    pub message: Option<SlackMessage>,
    pub channel: SlackChannel,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SlackUser {
    pub id: String,
    pub username: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SlackAction {
    #[serde(rename = "type")]
    pub action_type: String,
    pub action_id: String,
    pub value: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SlackMessageBlock {
    #[serde(rename = "type")]
    pub block_type: String,
    pub text: Option<SlackMessageText>,
    pub elements: Option<Vec<SlackElement>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SlackMessageText {
    pub text: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SlackChannel {
    pub id: String,
}

#[derive(Debug, Serialize)]
pub struct SlackMessageRequest {
    pub channel: String,
    pub blocks: Vec<SlackBlock>,
}

fn default_null_string() -> String {
    "null".to_string()
} 