use axum::http::{HeaderMap, StatusCode};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::{env, time::{SystemTime, UNIX_EPOCH}};

pub fn format_text_for_slack(text: &str) -> String {
    let mut formatted = text
        .replace("\\(", "`")
        .replace("\\)", "`")
        .replace("\\[", "```")
        .replace("\\]", "```")
        .replace("\\frac{", "(")
        .replace("}{", ")/(")
        .replace("}", ")")
        .replace("\\cdot", "×")
        .replace("\\times", "×")
        .replace("\\div", "÷")
        .replace("\\sqrt{", "√(")
        .replace("^{", "^(")
        .replace("_{", "_(")
        .replace("\\pi", "π")
        .replace("\\alpha", "α")
        .replace("\\beta", "β")
        .replace("\\gamma", "γ")
        .replace("\\delta", "δ")
        .replace("\\theta", "θ")
        .replace("\\lambda", "λ")
        .replace("\\mu", "μ")
        .replace("\\sigma", "σ")
        .replace("\\omega", "ω")
        .replace("\\pm", "±")
        .replace("\\leq", "≤")
        .replace("\\geq", "≥")
        .replace("\\neq", "≠")
        .replace("\\approx", "≈")
        .replace("\\infty", "∞")
        .replace("\\sin", "sin")
        .replace("\\cos", "cos")
        .replace("\\tan", "tan")
        .replace("\\log", "log")
        .replace("\\ln", "ln");

    formatted = formatted
        .replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;");

    formatted
}

pub async fn verify_slack_signature(
    headers: &HeaderMap,
    body: &str,
) -> Result<(), StatusCode> {
    let timestamp = headers
        .get("x-slack-request-timestamp")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .ok_or_else(|| {
            tracing::error!("Missing or invalid timestamp header");
            StatusCode::BAD_REQUEST
        })?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    tracing::debug!("Timestamp validation: now={}, request_time={}, diff={}", now, timestamp, now.abs_diff(timestamp));
    
    if now.abs_diff(timestamp) > 300 {
        tracing::error!("Request timestamp too far from current time: {} (now: {})", timestamp, now);
        return Err(StatusCode::BAD_REQUEST);
    }

    let signing_secret = env::var("SLACK_SIGNING_SECRET").map_err(|e| {
        tracing::error!("Failed to get SLACK_SIGNING_SECRET: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let sig_basestring = format!("v0:{}:{}", timestamp, body);
    tracing::debug!("Signature base string: {}", sig_basestring);
    
    let mut mac = Hmac::<Sha256>::new_from_slice(signing_secret.as_bytes())
        .map_err(|e| {
            tracing::error!("Failed to create HMAC: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    mac.update(sig_basestring.as_bytes());
    let expected_signature = format!("v0={}", hex::encode(mac.finalize().into_bytes()));

    let slack_signature = headers
        .get("x-slack-signature")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| {
            tracing::error!("Missing Slack signature header");
            StatusCode::BAD_REQUEST
        })?;

    tracing::debug!("Expected signature: {}", expected_signature);
    tracing::debug!("Received signature: {}", slack_signature);

    if slack_signature != expected_signature {
        tracing::error!("Signature mismatch");
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(())
} 