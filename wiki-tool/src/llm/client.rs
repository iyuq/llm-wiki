use futures_util::StreamExt;
use reqwest::Client;
use std::time::Duration;

use crate::config::ProviderConfig;
use crate::llm::providers::Provider;
use crate::WikiToolError;

/// Streaming LLM client using reqwest + tokio.
pub struct LlmClient {
    client: Client,
    provider: Provider,
    config: ProviderConfig,
}

impl LlmClient {
    /// Create a new LLM client from config.
    pub fn new(config: ProviderConfig) -> crate::Result<Self> {
        let provider = Provider::detect(&config.api_url);
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|e| WikiToolError::Http(e))?;

        Ok(Self {
            client,
            provider,
            config,
        })
    }

    /// Send a prompt and collect the full response (non-streaming).
    pub async fn complete(&self, system: &str, user: &str) -> crate::Result<String> {
        let (url, headers, body) =
            self.provider
                .build_request(&self.config, system, user, false);

        let mut request = self.client.post(&url);
        for (key, value) in &headers {
            request = request.header(key.as_str(), value.as_str());
        }

        let response = request.json(&body).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(WikiToolError::Llm(format!(
                "API returned {}: {}",
                status, text
            )));
        }

        let response_body: serde_json::Value = response.json().await?;
        self.provider.extract_content(&response_body)
    }

    /// Send a prompt and stream the response via SSE.
    pub async fn complete_streaming(
        &self,
        system: &str,
        user: &str,
        on_chunk: impl Fn(&str),
    ) -> crate::Result<String> {
        let (url, headers, body) =
            self.provider
                .build_request(&self.config, system, user, true);

        let mut request = self.client.post(&url);
        for (key, value) in &headers {
            request = request.header(key.as_str(), value.as_str());
        }

        let response = request.json(&body).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(WikiToolError::Llm(format!(
                "API returned {}: {}",
                status, text
            )));
        }

        let mut full_response = String::new();
        let mut stream = response.bytes_stream();

        let mut buffer = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            let text = String::from_utf8_lossy(&chunk);
            buffer.push_str(&text);

            // Process complete SSE lines
            while let Some(newline_pos) = buffer.find('\n') {
                let line = buffer[..newline_pos].trim().to_string();
                buffer = buffer[newline_pos + 1..].to_string();

                if line.is_empty() || line.starts_with(':') {
                    continue;
                }

                if line == "data: [DONE]" {
                    break;
                }

                if let Some(data) = line.strip_prefix("data: ") {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                        if let Some(content) =
                            self.provider.extract_stream_content(&json)
                        {
                            on_chunk(&content);
                            full_response.push_str(&content);
                        }
                    }
                }
            }
        }

        Ok(full_response)
    }
}
