
use crate::config::ProviderConfig;
use crate::WikiToolError;

/// Supported LLM providers.
#[derive(Debug, Clone)]
pub enum Provider {
    OpenAI,
    Anthropic,
    Google,
    Ollama,
    Custom,
}

impl Provider {
    /// Detect the provider from the API URL.
    pub fn detect(api_url: &str) -> Self {
        if api_url.contains("anthropic.com") {
            Self::Anthropic
        } else if api_url.contains("openai.com") {
            Self::OpenAI
        } else if api_url.contains("googleapis.com") || api_url.contains("generativelanguage") {
            Self::Google
        } else if api_url.contains("localhost:11434") || api_url.contains("ollama") {
            Self::Ollama
        } else {
            Self::Custom // Assume OpenAI-compatible
        }
    }

    /// Build the HTTP request for this provider.
    /// Returns (url, headers, body).
    pub fn build_request(
        &self,
        config: &ProviderConfig,
        system: &str,
        user: &str,
        stream: bool,
    ) -> (String, Vec<(String, String)>, serde_json::Value) {
        match self {
            Self::Anthropic => self.build_anthropic(config, system, user, stream),
            Self::OpenAI | Self::Custom => self.build_openai(config, system, user, stream),
            Self::Google => self.build_google(config, system, user, stream),
            Self::Ollama => self.build_ollama(config, system, user, stream),
        }
    }

    fn build_openai(
        &self,
        config: &ProviderConfig,
        system: &str,
        user: &str,
        stream: bool,
    ) -> (String, Vec<(String, String)>, serde_json::Value) {
        let mut headers = vec![
            ("Content-Type".to_string(), "application/json".to_string()),
        ];
        if let Some(ref key) = config.api_key {
            headers.push(("Authorization".to_string(), format!("Bearer {}", key)));
        }

        let body = serde_json::json!({
            "model": config.model,
            "max_tokens": config.max_tokens,
            "stream": stream,
            "messages": [
                { "role": "system", "content": system },
                { "role": "user", "content": user }
            ]
        });

        (config.api_url.clone(), headers, body)
    }

    fn build_anthropic(
        &self,
        config: &ProviderConfig,
        system: &str,
        user: &str,
        stream: bool,
    ) -> (String, Vec<(String, String)>, serde_json::Value) {
        let mut headers = vec![
            ("Content-Type".to_string(), "application/json".to_string()),
            (
                "anthropic-version".to_string(),
                "2023-06-01".to_string(),
            ),
        ];
        if let Some(ref key) = config.api_key {
            headers.push(("x-api-key".to_string(), key.clone()));
        }

        let body = serde_json::json!({
            "model": config.model,
            "max_tokens": config.max_tokens,
            "stream": stream,
            "system": system,
            "messages": [
                { "role": "user", "content": user }
            ]
        });

        (config.api_url.clone(), headers, body)
    }

    fn build_google(
        &self,
        config: &ProviderConfig,
        system: &str,
        user: &str,
        stream: bool,
    ) -> (String, Vec<(String, String)>, serde_json::Value) {
        let headers = vec![
            ("Content-Type".to_string(), "application/json".to_string()),
        ];

        let mut url = config.api_url.clone();
        if let Some(ref key) = config.api_key {
            let sep = if url.contains('?') { "&" } else { "?" };
            url = format!("{}{}key={}", url, sep, key);
        }

        let body = serde_json::json!({
            "contents": [
                {
                    "role": "user",
                    "parts": [{ "text": format!("{}\n\n{}", system, user) }]
                }
            ],
            "generationConfig": {
                "maxOutputTokens": config.max_tokens,
            }
        });

        let _ = stream; // Google uses different endpoint for streaming
        (url, headers, body)
    }

    fn build_ollama(
        &self,
        config: &ProviderConfig,
        system: &str,
        user: &str,
        stream: bool,
    ) -> (String, Vec<(String, String)>, serde_json::Value) {
        let headers = vec![
            ("Content-Type".to_string(), "application/json".to_string()),
        ];

        let body = serde_json::json!({
            "model": config.model,
            "stream": stream,
            "messages": [
                { "role": "system", "content": system },
                { "role": "user", "content": user }
            ]
        });

        (config.api_url.clone(), headers, body)
    }

    /// Extract the content from a non-streaming response.
    pub fn extract_content(&self, response: &serde_json::Value) -> crate::Result<String> {
        match self {
            Self::Anthropic => {
                response["content"][0]["text"]
                    .as_str()
                    .map(|s| s.to_string())
                    .ok_or_else(|| {
                        WikiToolError::Llm("Failed to extract content from Anthropic response".to_string())
                    })
            }
            Self::OpenAI | Self::Custom => {
                response["choices"][0]["message"]["content"]
                    .as_str()
                    .map(|s| s.to_string())
                    .ok_or_else(|| {
                        WikiToolError::Llm("Failed to extract content from OpenAI response".to_string())
                    })
            }
            Self::Google => {
                response["candidates"][0]["content"]["parts"][0]["text"]
                    .as_str()
                    .map(|s| s.to_string())
                    .ok_or_else(|| {
                        WikiToolError::Llm("Failed to extract content from Google response".to_string())
                    })
            }
            Self::Ollama => {
                response["message"]["content"]
                    .as_str()
                    .map(|s| s.to_string())
                    .ok_or_else(|| {
                        WikiToolError::Llm("Failed to extract content from Ollama response".to_string())
                    })
            }
        }
    }

    /// Extract content from a streaming SSE event.
    pub fn extract_stream_content(&self, event: &serde_json::Value) -> Option<String> {
        match self {
            Self::Anthropic => {
                // Anthropic streaming: content_block_delta events
                if event.get("type").and_then(|t| t.as_str()) == Some("content_block_delta") {
                    event["delta"]["text"].as_str().map(|s| s.to_string())
                } else {
                    None
                }
            }
            Self::OpenAI | Self::Custom => {
                // OpenAI streaming: choices[0].delta.content
                event["choices"][0]["delta"]["content"]
                    .as_str()
                    .map(|s| s.to_string())
            }
            Self::Ollama => {
                event["message"]["content"]
                    .as_str()
                    .map(|s| s.to_string())
            }
            Self::Google => {
                event["candidates"][0]["content"]["parts"][0]["text"]
                    .as_str()
                    .map(|s| s.to_string())
            }
        }
    }
}
