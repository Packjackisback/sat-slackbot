use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize, Default)]
struct Visuals {
    #[serde(rename = "type")]
    #[serde(default = "default_null_string")]
    visual_type: String,
    #[serde(default = "default_null_string")]
    svg_content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Question {
    #[serde(default = "default_null_string")]
    paragraph: String,
    question: String,
    choices: Choices,
    correct_answer: String,
    explanation: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Choices {
    A: String,
    B: String,
    C: String,
    D: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SATQuestion {
    id: String,
    domain: String,
    #[serde(default)]
    visuals: Visuals,
    question: Question,
    difficulty: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct MathResponse {
    math: Vec<SATQuestion>,
}

fn default_null_string() -> String {
    "null".to_string()
}

#[tokio::main]
async fn main() -> Result<()> {
    // Create an HTTP client
    let client = reqwest::Client::new();

    // Fetch SAT questions from JSON Silo API
    let response = client
        .get("https://api.jsonsilo.com/public/942c3c3b-3a0c-4be3-81c2-12029def19f5")
        .send()
        .await?;

    if response.status().is_success() {
        let data: MathResponse = response.json().await?;
        
        // Print each math question
        for (index, question) in data.math.iter().enumerate() {
            println!("\nQuestion #{}", index + 1);
            println!("ID: {}", question.id);
            println!("Domain: {}", question.domain);
            println!("Difficulty: {}", question.difficulty);
            println!("Question: {}", question.question.question);
            
            if question.question.paragraph != "null" {
                println!("\nParagraph: {}", question.question.paragraph);
            }
            
            println!("\nChoices:");
            println!("A) {}", question.question.choices.A);
            println!("B) {}", question.question.choices.B);
            println!("C) {}", question.question.choices.C);
            println!("D) {}", question.question.choices.D);
            println!("\nCorrect Answer: {}", question.question.correct_answer);
            println!("\nExplanation: {}", question.question.explanation);
            println!("\n{}", "-".repeat(80));
        }
    } else {
        println!("Error: {}", response.status());
    }

    Ok(())
} 