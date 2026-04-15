use futures_util::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

use crate::state::AppState;

const CLOUD_STREAM_EVENT: &str = "cloud_message_stream";

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CloudStreamEvent {
    pub request_id: String,
    pub delta: Option<String>,
    pub done: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
struct ProviderConfig {
    url: String,
    headers: HeaderMap,
    is_anthropic: bool,
}

fn provider_secret_key(provider: &str) -> Option<&'static str> {
    match provider {
        "anthropic" => Some("anthropic_api_key"),
        "ollama" => Some("ollama_endpoint"),
        "openrouter" => Some("openrouter_api_key"),
        "gemini" => Some("gemini_api_key"),
        "kimi" => Some("kimi_api_key"),
        "minimax" => Some("minimax_api_key"),
        _ => None,
    }
}

fn provider_base_url(provider: &str) -> Option<&'static str> {
    match provider {
        "anthropic" => Some("https://api.anthropic.com/v1"),
        "openrouter" => Some("https://openrouter.ai/api/v1"),
        "gemini" => Some("https://generativelanguage.googleapis.com/v1beta"),
        "kimi" => Some("https://api.moonshot.cn/v1"),
        "minimax" => Some("https://api.minimax.chat/v1"),
        _ => None,
    }
}

fn resolve_provider_config(provider: &str, secret: &str) -> Result<ProviderConfig, String> {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    if provider == "ollama" {
        let host = secret.trim().trim_end_matches('/');
        return Ok(ProviderConfig {
            url: format!("{host}/v1/chat/completions"),
            headers,
            is_anthropic: false,
        });
    }

    if provider == "anthropic" {
        let base = provider_base_url(provider).ok_or_else(|| "Unsupported provider".to_string())?;
        headers.insert(
            "x-api-key",
            HeaderValue::from_str(secret.trim())
                .map_err(|_| "Invalid provider secret".to_string())?,
        );
        headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));
        return Ok(ProviderConfig {
            url: format!("{base}/messages"),
            headers,
            is_anthropic: true,
        });
    }

    let base = provider_base_url(provider).ok_or_else(|| "Unsupported provider".to_string())?;
    let auth_value = format!("Bearer {}", secret.trim());
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&auth_value).map_err(|_| "Invalid provider secret".to_string())?,
    );

    Ok(ProviderConfig {
        url: format!("{base}/chat/completions"),
        headers,
        is_anthropic: false,
    })
}

fn emit_stream_event(app: &AppHandle, payload: CloudStreamEvent) {
    if let Err(err) = app.emit(CLOUD_STREAM_EVENT, payload) {
        log::error!("failed to emit cloud stream event: {err}");
    }
}

fn parse_sse_delta(provider: &str, line: &str) -> Option<String> {
    if !line.starts_with("data: ") {
        return None;
    }

    let json_str = line.trim_start_matches("data: ").trim();
    if json_str == "[DONE]" {
        return None;
    }

    let value: serde_json::Value = serde_json::from_str(json_str).ok()?;

    if provider == "anthropic" {
        return value
            .get("delta")
            .and_then(|d| d.get("text"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
    }

    value
        .get("choices")
        .and_then(|choices| choices.get(0))
        .and_then(|choice| choice.get("delta"))
        .and_then(|delta| delta.get("content"))
        .and_then(|content| content.as_str())
        .map(|s| s.to_string())
}

async fn run_cloud_stream(
    app: AppHandle,
    request_id: String,
    provider: String,
    model: Option<String>,
    messages: Vec<ProviderMessage>,
    config: ProviderConfig,
) {
    let mut body = serde_json::json!({
        "messages": messages,
        "stream": true,
        "max_tokens": 1024,
    });

    if let Some(m) = model {
        body["model"] = serde_json::Value::String(m);
    }

    if config.is_anthropic {
        let system = messages
            .iter()
            .find(|m| m.role == "system")
            .map(|m| m.content.clone())
            .unwrap_or_default();
        let anthropic_messages: Vec<serde_json::Value> = messages
            .into_iter()
            .filter(|m| m.role != "system")
            .map(|m| serde_json::json!({ "role": m.role, "content": m.content }))
            .collect();

        body = serde_json::json!({
            "model": body.get("model").and_then(|v| v.as_str()).unwrap_or("claude-sonnet-4-5"),
            "system": system,
            "messages": anthropic_messages,
            "stream": true,
            "max_tokens": 1024,
        });
    }

    let client = reqwest::Client::new();
    let response = client
        .post(&config.url)
        .headers(config.headers)
        .json(&body)
        .send()
        .await;

    let response = match response {
        Ok(r) => r,
        Err(err) => {
            emit_stream_event(
                &app,
                CloudStreamEvent {
                    request_id,
                    delta: None,
                    done: true,
                    error: Some(format!("Cannot reach provider {provider}: {err}")),
                },
            );
            return;
        }
    };

    if !response.status().is_success() {
        let status = response.status();
        let message = response
            .text()
            .await
            .unwrap_or_else(|_| "Connection failed".to_string());
        emit_stream_event(
            &app,
            CloudStreamEvent {
                request_id,
                delta: None,
                done: true,
                error: Some(format!("{provider} error ({status}): {message}")),
            },
        );
        return;
    }

    let mut stream = response.bytes_stream();
    let mut buffer = String::new();

    while let Some(chunk) = stream.next().await {
        let chunk = match chunk {
            Ok(c) => c,
            Err(err) => {
                emit_stream_event(
                    &app,
                    CloudStreamEvent {
                        request_id: request_id.clone(),
                        delta: None,
                        done: true,
                        error: Some(format!("stream error: {err}")),
                    },
                );
                return;
            }
        };

        buffer.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(idx) = buffer.find('\n') {
            let mut line = buffer[..idx].to_string();
            buffer.drain(..=idx);

            if line.ends_with('\r') {
                line.pop();
            }
            if line.is_empty() || line.starts_with(':') {
                continue;
            }
            if line.trim() == "data: [DONE]" {
                emit_stream_event(
                    &app,
                    CloudStreamEvent {
                        request_id: request_id.clone(),
                        delta: None,
                        done: true,
                        error: None,
                    },
                );
                return;
            }
            if let Some(delta) = parse_sse_delta(&provider, &line) {
                emit_stream_event(
                    &app,
                    CloudStreamEvent {
                        request_id: request_id.clone(),
                        delta: Some(delta),
                        done: false,
                        error: None,
                    },
                );
            }
        }
    }

    emit_stream_event(
        &app,
        CloudStreamEvent {
            request_id,
            delta: None,
            done: true,
            error: None,
        },
    );
}

#[tauri::command]
pub async fn stream_cloud_message(
    provider: String,
    model: Option<String>,
    messages: Vec<ProviderMessage>,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let secret_key = provider_secret_key(&provider)
        .ok_or_else(|| format!("Unsupported cloud provider: {provider}"))?;

    let secret = state
        .security
        .get_secret(secret_key)
        .map_err(|err| err.to_string())?
        .unwrap_or_default();

    if secret.trim().is_empty() {
        return Err(format!("Provider not connected: {provider}"));
    }

    let config = resolve_provider_config(&provider, &secret)?;
    let request_id = uuid::Uuid::new_v4().to_string();

    tauri::async_runtime::spawn(run_cloud_stream(
        app,
        request_id.clone(),
        provider,
        model,
        messages,
        config,
    ));

    Ok(request_id)
}
