use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::settings::AppSettings;

#[derive(Serialize, Deserialize, Debug)]
pub struct PromptRequest {
    pub selected_content: String,
    pub prompt: String,
    pub content_type: Option<String>, // e.g. "text", "table", "html"
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PromptResponse {
    pub modified_content: String,
    pub format: String, // "text" | "html" | "markdown"
}

pub async fn execute_prompt(
    settings: &AppSettings,
    req: &PromptRequest,
) -> Result<PromptResponse, String> {
    let client = reqwest::Client::builder()
        .use_rustls_tls()
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let system_prompt = "You are a professional Microsoft Word assistant. \
Your task is to modify, rewrite, format, or analyze the user's selected Word document content based on their prompt. \
Provide ONLY the modified or new content to be inserted back into the document. Do not include any conversational filler, conversational greetings, markdown code block backticks (like ```html), or explanations. Just output the raw resulting content. \
If the user asks to format something, generate tables, or add styling, output high-quality, clean HTML (without outer <html> or <body> tags, just structural tags like <table>, <p>, <strong>, <em>, <ul>, etc.) so that it can be cleanly inserted via Office.js selection.insertHtml(). \
If the request is simple plain text modification, output plain text.";

    if settings.active_provider == "openai" {
        if settings.openai_api_key.trim().is_empty() {
            return Err("OpenAI API key is missing. Please set it in Settings.".to_string());
        }

        let model = if settings.openai_model.is_empty() {
            "gpt-4o-mini"
        } else {
            &settings.openai_model
        };

        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", settings.openai_api_key))
            .json(&json!({
                "model": model,
                "messages": [
                    { "role": "system", "content": system_prompt },
                    { "role": "user", "content": format!("Selected Content:\n\"\"\"\n{}\n\"\"\"\n\nUser Instruction: {}", req.selected_content, req.prompt) }
                ],
                "temperature": 0.3
            }))
            .send()
            .await
            .map_err(|e| format!("API request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();
            return Err(format!("OpenAI Error ({}): {}", status, error_body));
        }

        let res_body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response JSON: {}", e))?;

        let ai_text = res_body["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| "Failed to extract content from OpenAI response".to_string())?
            .trim();

        // Clean up markdown block wrappers if AI accidentally added them
        let clean_text = clean_ai_response(ai_text);
        let format = detect_format(&clean_text);

        Ok(PromptResponse {
            modified_content: clean_text,
            format,
        })
    } else if settings.active_provider == "gemini" {
        if settings.gemini_api_key.trim().is_empty() {
            return Err("Gemini API key is missing. Please set it in Settings.".to_string());
        }

        let model = if settings.gemini_model.is_empty() {
            "gemini-1.5-flash"
        } else {
            &settings.gemini_model
        };

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            model, settings.gemini_api_key
        );

        let payload = json!({
            "contents": [{
                "parts": [{
                    "text": format!("Selected Content:\n\"\"\"\n{}\n\"\"\"\n\nUser Instruction: {}", req.selected_content, req.prompt)
                }]
            }],
            "systemInstruction": {
                "parts": [{
                    "text": system_prompt
                }]
            },
            "generationConfig": {
                "temperature": 0.3
            }
        });

        let response = client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("API request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();
            return Err(format!("Gemini Error ({}): {}", status, error_body));
        }

        let res_body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response JSON: {}", e))?;

        let ai_text = res_body["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .ok_or_else(|| "Failed to extract content from Gemini response".to_string())?
            .trim();

        let clean_text = clean_ai_response(ai_text);
        let format = detect_format(&clean_text);

        Ok(PromptResponse {
            modified_content: clean_text,
            format,
        })
    } else {
        Err(format!("Unsupported provider: {}", settings.active_provider))
    }
}

fn clean_ai_response(text: &str) -> String {
    let mut cleaned = text.to_string();
    
    // Strip leading ```html and trailing ```
    if cleaned.starts_with("```html") {
        cleaned = cleaned.replacen("```html", "", 1);
        if cleaned.ends_with("```") {
            cleaned.truncate(cleaned.len() - 3);
        }
    } else if cleaned.starts_with("```xml") {
        cleaned = cleaned.replacen("```xml", "", 1);
        if cleaned.ends_with("```") {
            cleaned.truncate(cleaned.len() - 3);
        }
    } else if cleaned.starts_with("```") {
        cleaned = cleaned.replacen("```", "", 1);
        if cleaned.ends_with("```") {
            cleaned.truncate(cleaned.len() - 3);
        }
    }
    
    cleaned.trim().to_string()
}

fn detect_format(text: &str) -> String {
    let text_lower = text.to_lowercase();
    if text_lower.contains("<table") || text_lower.contains("<p>") || text_lower.contains("<div") || text_lower.contains("<strong>") || text_lower.contains("<li>") {
        "html".to_string()
    } else {
        "text".to_string()
    }
}
