//! Google Trends data fetcher
//!
//! Fetches search interest data from Google Trends for stock symbols.
//! Note: Uses unofficial API endpoints - may be blocked by Google's bot detection.

use chrono::NaiveDate;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, REFERER, USER_AGENT};
use serde::{Deserialize, Serialize};

use crate::db::Database;
use crate::error::Result;

/// Google Trends data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendData {
    pub keyword: String,
    pub date: NaiveDate,
    pub value: i32, // 0-100 relative interest
}

/// Google Trends fetcher
pub struct GoogleTrends {
    client: Client,
}

impl GoogleTrends {
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
        ));
        headers.insert(ACCEPT, HeaderValue::from_static(
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8"
        ));
        headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));

        let client = Client::builder()
            .default_headers(headers)
            .cookie_store(true)
            .build()
            .unwrap_or_else(|_| Client::new());

        Self { client }
    }

    /// Fetch trends data for a keyword (symbol or company name)
    /// Returns interest over time data (0-100 scale)
    pub fn fetch(&self, keyword: &str) -> Result<Vec<TrendData>> {
        // First, visit the main trends page to get cookies
        let _homepage = self.client
            .get("https://trends.google.com/trends/")
            .send();

        // Small delay to appear more human-like
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Build the explore URL with proper encoding
        let req_json = format!(
            r#"{{"comparisonItem":[{{"keyword":"{}","geo":"","time":"today 12-m"}}],"category":0,"property":""}}"#,
            keyword
        );

        let explore_url = format!(
            "https://trends.google.com/trends/api/explore?hl=en-US&tz=360&req={}",
            urlencoding::encode(&req_json)
        );

        // Get the explore page to extract tokens
        let explore_resp = self.client
            .get(&explore_url)
            .header(REFERER, "https://trends.google.com/trends/explore")
            .header(ACCEPT, "application/json, text/plain, */*")
            .send()?;

        let status = explore_resp.status();
        let explore_text = explore_resp.text()?;

        // Check for error responses
        if !status.is_success() {
            return Err(crate::error::PipelineError::ApiError(
                format!("Google Trends returned status {}: Request may be blocked", status)
            ));
        }

        // Debug: Check what we got
        if explore_text.contains("<!DOCTYPE") || explore_text.contains("<html") {
            return Err(crate::error::PipelineError::ApiError(
                "Google Trends returned HTML (likely blocked or requires captcha). Try again later.".to_string()
            ));
        }

        // Google prefixes response with ")]}'" - remove it
        let json_text = explore_text.trim_start_matches(")]}'").trim();

        if json_text.is_empty() {
            return Err(crate::error::PipelineError::ApiError(
                "Empty response from Google Trends".to_string()
            ));
        }

        // Parse to extract the token for interest over time widget
        let explore_data: ExploreResponse = serde_json::from_str(json_text)
            .map_err(|e| {
                // Log the actual response for debugging
                println!("[DEBUG] Response preview: {}", &explore_text[..explore_text.len().min(200)]);
                crate::error::PipelineError::ApiError(
                    format!("Failed to parse explore response: {}. Google may have changed their API or blocked the request.", e)
                )
            })?;

        // Find the TIMESERIES widget
        let timeseries_widget = explore_data.widgets.iter()
            .find(|w| w.id == "TIMESERIES")
            .ok_or_else(|| crate::error::PipelineError::ApiError(
                "No TIMESERIES widget found in response".to_string()
            ))?;

        std::thread::sleep(std::time::Duration::from_millis(300));

        // Fetch the actual trend data using the token
        let multiline_url = format!(
            "https://trends.google.com/trends/api/widgetdata/multiline?hl=en-US&tz=360&req={}&token={}",
            urlencoding::encode(&timeseries_widget.request),
            urlencoding::encode(&timeseries_widget.token)
        );

        let data_resp = self.client
            .get(&multiline_url)
            .header(REFERER, "https://trends.google.com/trends/explore")
            .header(ACCEPT, "application/json, text/plain, */*")
            .send()?;

        let data_status = data_resp.status();
        let data_text = data_resp.text()?;

        // Debug: show what we got
        println!("[DEBUG] Trend data status: {}", data_status);
        println!("[DEBUG] Trend data preview: {}", &data_text[..data_text.len().min(300)]);

        if data_text.is_empty() {
            return Err(crate::error::PipelineError::ApiError(
                "Google Trends returned empty trend data. May be rate-limited.".to_string()
            ));
        }

        let data_json = data_text.trim_start_matches(")]}'").trim();

        let trend_response: TrendResponse = serde_json::from_str(data_json)
            .map_err(|e| crate::error::PipelineError::ApiError(
                format!("Failed to parse trend data: {}. Preview: {}", e, &data_json[..data_json.len().min(100)])
            ))?;

        // Convert to our TrendData format
        let mut results = Vec::new();

        if let Some(timeline) = trend_response.default.timeline_data {
            for point in timeline {
                if let (Some(time), Some(values)) = (point.time, point.value) {
                    // Parse timestamp (Google returns Unix timestamp as string)
                    if let Ok(ts) = time.parse::<i64>() {
                        let date = chrono::DateTime::from_timestamp(ts, 0)
                            .map(|dt| dt.date_naive())
                            .unwrap_or_else(|| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());

                        let value = values.first().copied().unwrap_or(0);

                        results.push(TrendData {
                            keyword: keyword.to_string(),
                            date,
                            value,
                        });
                    }
                }
            }
        }

        if results.is_empty() {
            return Err(crate::error::PipelineError::ApiError(
                "No trend data returned. The keyword may not have enough search volume.".to_string()
            ));
        }

        println!("[OK] Fetched {} trend data points for {}", results.len(), keyword);
        Ok(results)
    }

    /// Fetch and store trends data
    pub fn fetch_and_store(&self, db: &mut Database, keyword: &str) -> Result<usize> {
        let data = self.fetch(keyword)?;
        let count = data.len();
        db.upsert_trends(&data)?;
        Ok(count)
    }
}

impl Default for GoogleTrends {
    fn default() -> Self {
        Self::new()
    }
}

// Internal response structures for Google Trends API

#[derive(Debug, Deserialize)]
struct ExploreResponse {
    widgets: Vec<Widget>,
}

#[derive(Debug, Deserialize)]
struct Widget {
    id: String,
    token: String,
    /// The request field can be either a JSON string or a JSON object
    /// We need to serialize it back to a string for the API call
    #[serde(deserialize_with = "deserialize_request")]
    request: String,
}

/// Custom deserializer that handles both string and object formats for request field
fn deserialize_request<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};

    struct RequestVisitor;

    impl<'de> Visitor<'de> for RequestVisitor {
        type Value = String;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string or a JSON object")
        }

        fn visit_str<E>(self, value: &str) -> std::result::Result<String, E>
        where
            E: de::Error,
        {
            Ok(value.to_string())
        }

        fn visit_map<M>(self, map: M) -> std::result::Result<String, M::Error>
        where
            M: de::MapAccess<'de>,
        {
            // Deserialize the map to a Value, then serialize back to string
            let value = serde_json::Value::deserialize(de::value::MapAccessDeserializer::new(map))
                .map_err(de::Error::custom)?;
            serde_json::to_string(&value).map_err(de::Error::custom)
        }
    }

    deserializer.deserialize_any(RequestVisitor)
}

#[derive(Debug, Deserialize)]
struct TrendResponse {
    default: TrendDefault,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TrendDefault {
    timeline_data: Option<Vec<TimelinePoint>>,
}

#[derive(Debug, Deserialize)]
struct TimelinePoint {
    time: Option<String>,
    value: Option<Vec<i32>>,
}
