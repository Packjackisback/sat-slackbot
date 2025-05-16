use anyhow::Result;
use crate::{models::*, utils::format_text_for_slack};
use rand::prelude::*;
use reqwest;
use std::env;

pub async fn fetch_question() -> Result<SATQuestion> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;
    
    let response = client
        .get("https://api.jsonsilo.com/public/942c3c3b-3a0c-4be3-81c2-12029def19f5")
        .send()
        .await?;

    let status = response.status();
    let response_text = response.text().await?;
    tracing::debug!("API Response status: {}", status);
    tracing::debug!("API Response body: {}", response_text);

    if !status.is_success() {
        return Err(anyhow::anyhow!(
            "API request failed with status {}: {}",
            status,
            response_text
        ));
    }

    match serde_json::from_str::<MathResponse>(&response_text) {
        Ok(data) => {
            let mut rng = rand::thread_rng();
            let question = data.math.choose(&mut rng)
                .ok_or_else(|| anyhow::anyhow!("No questions available in the response"))?;
            
            tracing::debug!("Successfully parsed question: {:?}", question);
            Ok(question.clone())
        },
        Err(e) => {
            tracing::error!("Failed to parse response as MathResponse: {}", e);
            tracing::error!("Response text: {}", response_text);
            
            match serde_json::from_str::<SATQuestion>(&response_text) {
                Ok(question) => {
                    tracing::debug!("Successfully parsed single question: {:?}", question);
                    Ok(question)
                },
                Err(e2) => {
                    tracing::error!("Also failed to parse as single question: {}", e2);
                    Err(anyhow::anyhow!("Failed to parse API response: {}", e))
                }
            }
        }
    }
}

pub fn create_question_blocks(question: &SATQuestion) -> Vec<SlackBlock> {
    tracing::debug!("Creating blocks for question: {:?}", question);
    
    let mut blocks = vec![
        SlackBlock {
            block_type: "section".to_string(),
            text: Some(SlackText {
                text_type: "mrkdwn".to_string(),
                text: format!(
                    "*Question:* {}\n*Domain:* {}\n*Difficulty:* {}",
                    format_text_for_slack(&question.question.question),
                    question.domain,
                    question.difficulty
                ),
                emoji: None,
            }),
            elements: None,
            accessory: None,
        },
    ];

    if question.question.paragraph != "null" {
        blocks.push(SlackBlock {
            block_type: "section".to_string(),
            text: Some(SlackText {
                text_type: "mrkdwn".to_string(),
                text: format!("*Paragraph:*\n{}", format_text_for_slack(&question.question.paragraph)),
                emoji: None,
            }),
            elements: None,
            accessory: None,
        });
    }

    let choices = vec![
        ("A", &question.question.choices.a),
        ("B", &question.question.choices.b),
        ("C", &question.question.choices.c),
        ("D", &question.question.choices.d),
    ];

    let correct_answer = &question.question.correct_answer;
    let buttons = choices
        .iter()
        .map(|(letter, text)| SlackElement {
            element_type: "button".to_string(),
            text: SlackText {
                text_type: "plain_text".to_string(),
                text: format!("{}. {}", letter, format_text_for_slack(text)),
                emoji: Some(true),
            },
            action_id: format!("answer_{}", letter.to_lowercase()),
            value: Some(format!("{}:{}", letter, correct_answer)),
        })
        .collect();

    blocks.push(SlackBlock {
        block_type: "actions".to_string(),
        text: None,
        elements: Some(buttons),
        accessory: None,
    });

    blocks.push(SlackBlock {
        block_type: "actions".to_string(),
        text: None,
        elements: Some(vec![SlackElement {
            element_type: "button".to_string(),
            text: SlackText {
                text_type: "plain_text".to_string(),
                text: "🗑️ Clear".to_string(),
                emoji: Some(true),
            },
            action_id: "clear_message".to_string(),
            value: Some("clear".to_string()),
        }]),
        accessory: None,
    });

    tracing::debug!("Generated blocks: {:?}", blocks);
    blocks
}

pub async fn post_message(token: &str, channel: &str, blocks: Vec<SlackBlock>) -> Result<()> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;
    let message = SlackMessageRequest {
        channel: channel.to_string(),
        blocks,
    };

    tracing::debug!("Sending message to Slack: {:?}", message);

    let response = client
        .post("https://slack.com/api/chat.postMessage")
        .header("Authorization", format!("Bearer {}", token))
        .json(&message)
        .send()
        .await?;

    let response_text = response.text().await?;
    tracing::debug!("Slack API response: {}", response_text);

    if !response_text.contains("\"ok\":true") {
        tracing::error!("Slack API error: {}", response_text);
        return Err(anyhow::anyhow!(
            "Failed to post message: {}",
            response_text
        ));
    }

    tracing::info!("Successfully posted message to Slack with response: {}", response_text);
    Ok(())
} 