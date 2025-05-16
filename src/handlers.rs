use axum::{
    extract::Form,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use reqwest;
use serde_json::json;
use std::{collections::HashMap, env};
use crate::{
    models::*,
    slack::{fetch_question, create_question_blocks, post_message},
    utils::verify_slack_signature,
};

pub async fn handle_slash_command(
    headers: HeaderMap,
    Form(payload): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    tracing::debug!("Received headers: {:?}", headers);
    tracing::debug!("Received payload: {:?}", payload);
    
    let body_str = serde_urlencoded::to_string(&payload).unwrap();
    tracing::debug!("Request body string: {}", body_str);
    
    tracing::info!("Signature verification disabled - proceeding with request");

    let command = SlackSlashCommand {
        channel_id: payload.get("channel_id").cloned().unwrap_or_default(),
        response_url: payload.get("response_url").cloned().unwrap_or_default(),
    };

    tokio::spawn(async move {
        match fetch_question().await {
            Ok(question) => {
                tracing::info!("Successfully fetched question: {:?}", question);
                let blocks = create_question_blocks(&question);
                let token = env::var("SLACK_BOT_TOKEN").expect("SLACK_BOT_TOKEN must be set");
                
                if let Err(e) = post_message(&token, &command.channel_id, blocks).await {
                    tracing::error!("Error posting message: {}", e);
                } else {
                    tracing::info!("Successfully posted message to Slack");
                    
                    let client = reqwest::Client::new();
                    if let Err(e) = client
                        .post(&command.response_url)
                        .json(&json!({
                            "delete_original": true
                        }))
                        .send()
                        .await
                    {
                        tracing::error!("Failed to delete loading message: {}", e);
                    }
                }
            }
            Err(e) => {
                tracing::error!("Error fetching question: {}", e);
            }
        }
    });

    (StatusCode::OK, "Loading your SAT question...").into_response()
}

pub async fn handle_interaction(
    headers: HeaderMap,
    Form(payload): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    tracing::debug!("Received interaction payload: {:?}", payload);

    let payload_str = match payload.get("payload") {
        Some(p) => p,
        None => {
            tracing::error!("No payload in interaction");
            return StatusCode::BAD_REQUEST.into_response();
        }
    };

    let interaction: SlackInteraction = match serde_json::from_str(payload_str) {
        Ok(i) => i,
        Err(e) => {
            tracing::error!("Failed to parse interaction: {}", e);
            return StatusCode::BAD_REQUEST.into_response();
        }
    };

    if interaction.interaction_type != "block_actions" {
        return StatusCode::OK.into_response();
    }

    let Some(actions) = interaction.actions else {
        return StatusCode::OK.into_response();
    };

    let Some(action) = actions.first() else {
        return StatusCode::OK.into_response();
    };

    let client = reqwest::Client::new();
    let token = env::var("SLACK_BOT_TOKEN").expect("SLACK_BOT_TOKEN must be set");

    if action.action_id == "clear_message" {
        if let Err(e) = client
            .post("https://slack.com/api/chat.delete")
            .header("Authorization", format!("Bearer {}", token))
            .json(&json!({
                "channel": interaction.channel.id,
                "ts": interaction.message.unwrap().ts
            }))
            .send()
            .await
        {
            tracing::error!("Failed to delete message: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
        return StatusCode::OK.into_response();
    }

    let Some(value) = &action.value else {
        return StatusCode::OK.into_response();
    };

    let (selected_answer, correct_answer) = match value.split_once(':') {
        Some((selected, correct)) => (selected, correct),
        None => {
            tracing::error!("Invalid value format in button: {}", value);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    tracing::debug!("Selected answer: {}, Correct answer: {}", selected_answer, correct_answer);

    let response_message = if selected_answer == correct_answer {
        json!({
            "response_type": "ephemeral",
            "blocks": [
                {
                    "type": "section",
                    "text": {
                        "type": "mrkdwn",
                        "text": format!("✅ Correct! Well done, <@{}>!", interaction.user.id)
                    }
                },
                {
                    "type": "actions",
                    "elements": [
                        {
                            "type": "button",
                            "text": {
                                "type": "plain_text",
                                "text": "🗑️ Clear",
                                "emoji": true
                            },
                            "action_id": "clear_message",
                            "value": "clear"
                        }
                    ]
                }
            ]
        })
    } else {
        json!({
            "response_type": "ephemeral",
            "blocks": [
                {
                    "type": "section",
                    "text": {
                        "type": "mrkdwn",
                        "text": format!("❌ Sorry <@{}>, that's not correct. Try again!", interaction.user.id)
                    }
                },
                {
                    "type": "actions",
                    "elements": [
                        {
                            "type": "button",
                            "text": {
                                "type": "plain_text",
                                "text": "🗑️ Clear",
                                "emoji": true
                            },
                            "action_id": "clear_message",
                            "value": "clear"
                        }
                    ]
                }
            ]
        })
    };

    if let Err(e) = client
        .post(&interaction.response_url)
        .json(&response_message)
        .send()
        .await
    {
        tracing::error!("Failed to send response: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    StatusCode::OK.into_response()
} 