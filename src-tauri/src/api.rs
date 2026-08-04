use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UsageData {
    pub usage: Usage,
    pub limits: Vec<Limit>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Usage {
    pub limit: String,
    pub remaining: String,
    #[serde(rename = "resetTime")]
    pub reset_time: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Limit {
    pub window: Window,
    pub detail: Detail,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Window {
    pub duration: u32,
    #[serde(rename = "timeUnit")]
    pub time_unit: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Detail {
    pub limit: String,
    pub remaining: String,
    #[serde(rename = "resetTime")]
    pub reset_time: String,
}

pub async fn fetch_usage(token: &str) -> Result<UsageData, String> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.kimi.com/coding/v1/usages")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API error: {} {}", response.status().as_u16(), response.status().canonical_reason().unwrap_or("")));
    }

    response
        .json::<UsageData>()
        .await
        .map_err(|e| format!("failed to parse response: {}", e))
}
