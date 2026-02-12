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

/// 调用智谱AI的Completion API。
///
/// 此函数用于向智谱AI发送请求，获取文本补全或聊天消息的响应。
/// 它处理API请求的发送、成功响应的解析以及错误（包括可重试错误）的处理。
///
/// # 参数
/// - `api_key`: 用于认证的API密钥。
/// - `request`: 包含模型、消息和其他参数的智谱AI请求体。
///
/// # 返回
/// `anyhow::Result<ZhiPuResponse>`: 成功时返回 `ZhiPuResponse`，失败时返回 `anyhow::Error`。
pub async fn zhi_pu_completion(
    api_key: &str,
    request: ZhiPuRequest,
) -> anyhow::Result<ZhiPuResponse> {
    let client = reqwest::Client::new();
    let mut retry_count = 0;
    const MAX_RETRIES: u32 = 3;

    loop {
        retry_count += 1; // Increment retry count at the beginning of each loop iteration

        let response_result = execute_zhi_pu_request(
            &client, api_key, &request,
        )
        .await;

        let response = match response_result {
            Ok(res) => res,
            Err(e) => {
                if retry_count <= MAX_RETRIES {
                    wait_before_retry(
                        retry_count,
                        &format!("network error: {}", e),
                    )
                    .await;
                    continue;
                }
                anyhow::bail!(
                    "ZhiPu API network error: {}",
                    e
                );
            }
        };

        let status = response.status(); // Capture status before response is moved

        match handle_http_response(
            response,
            retry_count,
            MAX_RETRIES,
        )
        .await?
        {
            Some(zhi_pu_response) => {
                return Ok(zhi_pu_response);
            }
            None => {
                // This means it's a retryable HTTP error and retry_count < MAX_RETRIES
                wait_before_retry(
                    retry_count,
                    &format!("transient error {}", status),
                )
                .await;
                continue;
            }
        }
    }
}

/// Handles the HTTP response from the ZhiPu API.
///
/// This function checks if the response was successful, a retryable error, or a non-retryable error.
///
/// # Arguments
/// - `response`: The `reqwest::Response` received from the API.
/// - `retry_count`: The current retry attempt number.
/// - `max_retries`: The maximum number of retries allowed.
///
/// # Returns
/// `Ok(Some(ZhiPuResponse))`: If the request was successful and the response was parsed.
/// `Ok(None)`: If the error is retryable and `retry_count` is less than `max_retries`.
/// `Err(anyhow::Error)`: If the error is not retryable or `retry_count` has exceeded `max_retries`.
async fn handle_http_response(
    response: reqwest::Response,
    retry_count: u32,
    max_retries: u32,
) -> anyhow::Result<Option<ZhiPuResponse>> {
    let status = response.status();

    if status.is_success() {
        let zhi_pu_response: ZhiPuResponse =
            response.json().await?;
        return Ok(Some(zhi_pu_response));
    }

    if is_retryable_error(status.as_u16()) {
        if retry_count < max_retries {
            return Ok(None); // Indicate that a retry is needed
        }
    }

    anyhow::bail!(
        "{}",
        format_error_response(response, status).await?
    );
}

/// Waits for a calculated duration before retrying an API call.
///
/// This function implements an exponential backoff strategy.
///
/// # Arguments
/// - `retry_count`: The current retry attempt number (1-indexed).
/// - `error_message`: A string slice describing the error that triggered the retry.
async fn wait_before_retry(
    retry_count: u32,
    error_message: &str,
) {
    let sleep_duration = std::time::Duration::from_secs(
        2u64.pow(retry_count - 1),
    );
    log::warn!(
        "ZhiPu API {} retrying in {:?}...",
        error_message,
        sleep_duration
    );
    tokio::time::sleep(sleep_duration).await;
}

async fn execute_zhi_pu_request(
    client: &reqwest::Client,
    api_key: &str,
    request_body: &ZhiPuRequest,
) -> Result<reqwest::Response, reqwest::Error> {
    client
        .post(format!(
            "{}/chat/completions",
            ZHI_PU_API_URL
        ))
        .header(
            "Authorization",
            format!("Bearer {}", api_key),
        )
        .json(request_body)
        .send()
        .await
}

fn is_retryable_error(status_code: u16) -> bool {
    matches!(status_code, 500 | 502 | 503 | 504)
}

async fn format_error_response(
    response: reqwest::Response,
    status: reqwest::StatusCode,
) -> anyhow::Result<String> {
    let error_text =
        response.text().await.unwrap_or_else(|_| {
            "Failed to read error body".to_string()
        });

    if let Ok(json_error) = serde_json::from_str::<
        serde_json::Value,
    >(&error_text)
    {
        Ok(format!("ZhiPu API error: {}", json_error))
    } else {
        let truncated_error = if error_text.len() > 200 {
            format!("{}...", &error_text[..200])
        } else {
            error_text
        };
        Ok(format!(
            "ZhiPu API error ({}): {}",
            status, truncated_error
        ))
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
