use serde::{Deserialize, Serialize};

// 接口 GET /coding/v1/usages 返回的是 protobuf-JSON。
// 关键坑：proto3-JSON 会省略取值等于默认值的标量字段（0 / "" / false），
// 所以这里任何可能为 0 的字段都不能声明成必填，否则一旦额度用满、
// 该字段从响应里消失，整个响应就会解析失败。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UsageData {
    pub usages: Usages,
}

// usages 是接口自己算好的窗口用量，比用 usage/limits 里的 limit 和 remaining 手工相减更可靠：
// 不会有除零，也不会因为 limits 数组顺序变化而把 5 小时和 7 天的数据对调。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Usages {
    pub limit_5h: Option<WindowUsage>,
    pub limit_7d: Option<WindowUsage>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WindowUsage {
    // 字段缺失即用量为 0：proto3 不发送默认值
    #[serde(default)]
    pub used_ratio: f64,
    #[serde(default)]
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

#[cfg(test)]
mod tests {
    use super::*;

    // 真实抓包：5 小时窗口刚用满时的响应。
    // 注意 limits[0].detail 里只有 used 没有 remaining，正是导致旧实现在这里
    // 解析失败并清空整个界面的原因。
    const EXHAUSTED: &str = r#"{
      "usage": { "limit": "100", "used": "39", "remaining": "61", "resetTime": "2026-10-13T02:29:57.787501Z" },
      "limits": [
        {
          "window": { "duration": 300, "timeUnit": "TIME_UNIT_MINUTE" },
          "detail": { "limit": "100", "used": "100", "resetTime": "2026-10-09T10:29:57.787501Z" }
        }
      ],
      "booster_wallet": { "id": "00000000-0000-0000-0000-000000000000", "status": "STATUS_DISABLED" },
      "usages": {
        "limit_5h": { "used_ratio": 1, "reset_time": "2026-10-09T10:29:56Z" },
        "limit_7d": { "used_ratio": 0.39325, "reset_time": "2026-10-13T02:29:57Z" }
      }
    }"#;

    const HEALTHY: &str = r#"{
      "usages": {
        "limit_5h": { "used_ratio": 0.3, "reset_time": "2026-10-09T10:29:56Z" },
        "limit_7d": { "used_ratio": 0.39325, "reset_time": "2026-10-13T02:29:57Z" }
      }
    }"#;

    #[test]
    fn parses_exhausted_payload() {
        let data: UsageData = serde_json::from_str(EXHAUSTED).expect("额度用满时也必须能解析");
        let five_hour = data.usages.limit_5h.expect("5 小时窗口");
        assert_eq!(five_hour.used_ratio, 1.0);
        assert_eq!(five_hour.reset_time, "2026-10-09T10:29:56Z");
        assert_eq!(data.usages.limit_7d.unwrap().used_ratio, 0.39325);
    }

    #[test]
    fn parses_healthy_payload() {
        let data: UsageData = serde_json::from_str(HEALTHY).expect("正常状态必须能解析");
        assert_eq!(data.usages.limit_5h.unwrap().used_ratio, 0.3);
    }

    #[test]
    fn omitted_used_ratio_means_zero() {
        // 用量为 0 时 proto3 会直接省略 used_ratio
        let data: UsageData =
            serde_json::from_str(r#"{"usages":{"limit_5h":{"reset_time":"2026-10-09T10:29:56Z"}}}"#)
                .expect("缺省字段必须能解析");
        assert_eq!(data.usages.limit_5h.unwrap().used_ratio, 0.0);
    }

    #[test]
    fn missing_window_is_none() {
        // 没有 5 小时窗口时整块 message 都会被省略，只影响这一行，不该让整个响应失败
        let data: UsageData = serde_json::from_str(r#"{"usages":{"limit_7d":{"used_ratio":0.5}}}"#)
            .expect("缺省窗口必须能解析");
        assert!(data.usages.limit_5h.is_none());
        assert_eq!(data.usages.limit_7d.unwrap().used_ratio, 0.5);
    }
}
