use serde::{Deserialize, Serialize};

static ZHI_PU_API_URL: &str =
    "https://api.z.ai/api/coding/paas/v4";
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ZhiPuMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ZhiPuRequest {
    pub model: String,
    pub messages: Vec<ZhiPuMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ZhiPuResponse {
    pub id: String,
    pub request_id: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<ZhiPuChoice>,
    pub usage: ZhiPuUsage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ZhiPuUsage {
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ZhiPuChoice {
    pub index: i32,
    pub message: ZhiPuResponseMessage,
    pub finish_reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ZhiPuResponseMessage {
    pub role: String,
    pub content: String,
    pub reasoning_content: Option<String>,
}

pub async fn zhi_pu_completion(
    api_key: &str,
    request: ZhiPuRequest,
) -> anyhow::Result<ZhiPuResponse> {
    let client = reqwest::Client::new();
    let mut retry_count = 0;
    const MAX_RETRIES: u32 = 3;

    loop {
        let response_result = client
            .post(format!(
                "{}/chat/completions",
                ZHI_PU_API_URL
            ))
            .header(
                "Authorization",
                format!("Bearer {}", api_key),
            )
            .json(&request)
            .send()
            .await;

        match response_result {
            Ok(response) => {
                if response.status().is_success() {
                    let zhi_pu_response: ZhiPuResponse =
                        response.json().await?;
                    return Ok(zhi_pu_response);
                } else {
                    let status = response.status();
                    if status.as_u16() == 500
                        || status.as_u16() == 502
                        || status.as_u16() == 503
                        || status.as_u16() == 504
                    {
                        if retry_count < MAX_RETRIES {
                            retry_count += 1;
                            let sleep_duration =
                                std::time::Duration::from_secs(2u64.pow(retry_count - 1));
                            log::warn!(
                                "ZhiPu API transient error {}, retrying in {:?}...",
                                status,
                                sleep_duration
                            );
                            tokio::time::sleep(
                                sleep_duration,
                            )
                            .await;
                            continue;
                        }
                    }

                    // Handle error response
                    let error_text = response
                        .text()
                        .await
                        .unwrap_or_else(|_| {
                            "Failed to read error body"
                                .to_string()
                        });

                    // Try to parse as JSON first
                    if let Ok(json_error) =
                        serde_json::from_str::<
                            serde_json::Value,
                        >(
                            &error_text
                        )
                    {
                        anyhow::bail!(
                            "ZhiPu API error: {}",
                            json_error
                        );
                    } else {
                        // If HTML or other text, truncate it to avoid huge logs
                        let truncated_error =
                            if error_text.len() > 200 {
                                format!(
                                    "{}...",
                                    &error_text[..200]
                                )
                            } else {
                                error_text
                            };
                        anyhow::bail!(
                            "ZhiPu API error ({}): {}",
                            status,
                            truncated_error
                        );
                    }
                }
            }
            Err(e) => {
                // Network error, maybe retry?
                if retry_count < MAX_RETRIES {
                    retry_count += 1;
                    let sleep_duration =
                        std::time::Duration::from_secs(
                            2u64.pow(retry_count - 1),
                        );
                    log::warn!(
                        "ZhiPu API network error {}, retrying in {:?}...",
                        e,
                        sleep_duration
                    );
                    tokio::time::sleep(sleep_duration)
                        .await;
                    continue;
                }
                anyhow::bail!(
                    "ZhiPu API network error: {}",
                    e
                );
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use utils::config::env::ENV_SETTINGS;

    #[tokio::test]
    async fn test_zhi_pu_completion() -> anyhow::Result<()>
    {
        let api_key = ENV_SETTINGS
            .zhi_pu_api_key
            .as_ref()
            .expect("ZHI_PU_API_KEY not set");

        let request = ZhiPuRequest {
            model: "glm-4.7".to_string(),
            messages: vec![ZhiPuMessage {
                role: "user".to_string(),
                content: "Hi".to_string(),
            }],
            stream: None,
            temperature: None,
        };

        let response =
            zhi_pu_completion(api_key, request).await?;

        assert!(!response.choices.is_empty());
        dbg!(&response);
        Ok(())
    }
}
