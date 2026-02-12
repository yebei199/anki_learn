use std::fmt::format;
use serde::{Deserialize, Serialize};

static ZHI_PU_API_URL: &str =
    "https://api.z.ai/api/coding/paas/v4";
#[derive(Debug, Serialize, Deserialize)]
pub struct ZhiPuMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
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
    let response = client
        .post(format!("{}/chat/completions", ZHI_PU_API_URL))
        .header(
            "Authorization",
            format!("Bearer {}", api_key),
        )
        .json(&request)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        anyhow::bail!("ZhiPu API error: {}", error_text);
    }

    let zhi_pu_response: ZhiPuResponse =
        response.json().await?;
    Ok(zhi_pu_response)
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
        Ok(())
    }
}
