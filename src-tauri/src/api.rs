use serde::{Deserialize, Serialize};
use crate::BalanceInfo;

#[derive(Debug, Deserialize)]
struct ApiResponse {
    data: Option<ApiData>,
    success: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct ApiData {
    token: Option<TokenInfo>,
}

#[derive(Debug, Deserialize)]
struct TokenInfo {
    used_quota_usd: f64,
    remain_quota_usd: f64,
    used_quota: i64,
    remain_quota: i64,
    initial_quota: i64,
    unlimited_quota: bool,
    next_reset_time: Option<i64>,
    expired_time: Option<i64>,
}

pub async fn fetch_balance(token: &str) -> Result<BalanceInfo, String> {
    let client = reqwest::Client::builder()
        .gzip(true)
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .get("https://viber.claudegateway.site/api/balance/check")
        .header("Authorization", format!("Bearer {}", token))
        .header("Accept", "application/json")
        .header("New-API-User", "-1")
        .header("Cache-Control", "no-store")
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    if resp.status() == 401 || resp.status() == 403 {
        return Err("Auth error: invalid token".to_string());
    }
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }

    let body: ApiResponse = resp.json().await
        .map_err(|e| format!("Parse error: {e}"))?;

    let token_info = body.data
        .and_then(|d| d.token)
        .ok_or("No token data in response")?;

    parse_balance(&token_info)
}

fn parse_balance(t: &TokenInfo) -> Result<BalanceInfo, String> {
    if t.unlimited_quota {
        return Ok(BalanceInfo {
            percent: 100.0,
            used_usd: t.used_quota_usd,
            remain_usd: t.remain_quota_usd,
            total_usd: t.used_quota_usd + t.remain_quota_usd,
            unlimited: true,
            next_reset_time: t.next_reset_time,
            expired_time: t.expired_time,
        });
    }

    let total_usd = t.used_quota_usd + t.remain_quota_usd;
    let percent = if total_usd > 0.0 {
        (t.remain_quota_usd / total_usd * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };

    Ok(BalanceInfo {
        percent,
        used_usd: t.used_quota_usd,
        remain_usd: t.remain_quota_usd,
        total_usd,
        unlimited: false,
        next_reset_time: t.next_reset_time,
        expired_time: t.expired_time,
    })
}

// ── Logs API ────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogItem {
    pub id: i64,
    pub created_at: i64,
    #[serde(rename = "type")]
    pub log_type: i32,
    pub content: String,
    pub token_name: String,
    pub model_name: String,
    pub quota: i64,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub use_time: i64,
    pub is_stream: bool,
    pub group: String,
    pub request_id: String,
    #[serde(default)]
    pub other: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogsPage {
    pub page: i32,
    pub page_size: i32,
    pub total: i64,
    pub items: Vec<LogItem>,
}

#[derive(Debug, Deserialize)]
struct LogsApiResponse {
    data: Option<LogsPage>,
    success: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogStatItem {
    pub model_name: String,
    pub quota: i64,
    pub count: i64,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
}

#[derive(Debug, Deserialize)]
struct LogStatsApiResponse {
    data: Option<serde_json::Value>,
    success: Option<bool>,
}

fn build_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .gzip(true)
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())
}

pub async fn fetch_logs(
    token: &str,
    page: i32,
    page_size: i32,
    log_type: i32,
    model_name: Option<&str>,
    start_timestamp: Option<i64>,
    end_timestamp: Option<i64>,
) -> Result<LogsPage, String> {
    let client = build_client()?;

    let mut url = format!(
        "https://viber.claudegateway.site/api/balance/logs?p={}&page_size={}&type={}",
        page, page_size, log_type
    );
    if let Some(m) = model_name {
        if !m.is_empty() {
            url.push_str(&format!("&model_name={}", m));
        }
    }
    if let Some(ts) = start_timestamp {
        url.push_str(&format!("&start_timestamp={}", ts));
    }
    if let Some(ts) = end_timestamp {
        url.push_str(&format!("&end_timestamp={}", ts));
    }

    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Accept", "application/json")
        .header("New-API-User", "-1")
        .header("Cache-Control", "no-store")
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }

    let body: LogsApiResponse = resp.json().await
        .map_err(|e| format!("Parse error: {e}"))?;

    body.data.ok_or("No log data in response".to_string())
}

pub async fn fetch_log_stats(
    token: &str,
    log_type: i32,
    model_name: Option<&str>,
    start_timestamp: Option<i64>,
    end_timestamp: Option<i64>,
) -> Result<serde_json::Value, String> {
    let client = build_client()?;

    let mut url = format!(
        "https://viber.claudegateway.site/api/balance/logs/stat?type={}",
        log_type
    );
    if let Some(m) = model_name {
        if !m.is_empty() {
            url.push_str(&format!("&model_name={}", m));
        }
    }
    if let Some(ts) = start_timestamp {
        url.push_str(&format!("&start_timestamp={}", ts));
    }
    if let Some(ts) = end_timestamp {
        url.push_str(&format!("&end_timestamp={}", ts));
    }

    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Accept", "application/json")
        .header("New-API-User", "-1")
        .header("Cache-Control", "no-store")
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }

    let body: LogStatsApiResponse = resp.json().await
        .map_err(|e| format!("Parse error: {e}"))?;

    body.data.ok_or("No stat data in response".to_string())
}
