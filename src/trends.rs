//! Google Trends data fetcher
//!
//! Fetches search interest data from Google Trends for stock symbols.
//! Note: Uses unofficial API endpoints - may require updates if Google changes their interface.

use chrono::NaiveDate;
use reqwest::blocking::Client;
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
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .build()
            .unwrap_or_else(|_| Client::new());

        Self { client }
    }

    /// Fetch trends data for a keyword (symbol or company name)
    /// Returns interest over time data (0-100 scale)
    pub fn fetch(&self, keyword: &str) -> Result<Vec<TrendData>> {
        // Google Trends explore URL to get widget tokens
        let explore_url = format!(
            "https://trends.google.com/trends/api/explore?hl=en-US&tz=360&req={{\"comparisonItem\":[{{\"keyword\":\"{}\",\"geo\":\"\",\"time\":\"today 12-m\"}}],\"category\":0,\"property\":\"\"}}",
            keyword
        );

        // First, get the explore page to extract tokens
        let explore_resp = self.client
            .get(&explore_url)
            .header("Accept", "application/json")
            .send()?;

        let explore_text = explore_resp.text()?;

        // Google prefixes response with ")]}'" - remove it
        let json_text = explore_text.trim_start_matches(")]}'\n");

        // Parse to extract the token for interest over time widget
        let explore_data: ExploreResponse = serde_json::from_str(json_text)
            .map_err(|e| crate::error::PipelineError::ApiError(format!("Failed to parse explore response: {}", e)))?;

        // Find the TIMESERIES widget
        let timeseries_widget = explore_data.widgets.iter()
            .find(|w| w.id == "TIMESERIES")
            .ok_or_else(|| crate::error::PipelineError::ApiError("No TIMESERIES widget found".to_string()))?;

        // Fetch the actual trend data using the token
        let multiline_url = format!(
            "https://trends.google.com/trends/api/widgetdata/multiline?hl=en-US&tz=360&req={}&token={}",
            urlencoding::encode(&timeseries_widget.request),
            urlencoding::encode(&timeseries_widget.token)
        );

        let data_resp = self.client
            .get(&multiline_url)
            .header("Accept", "application/json")
            .send()?;

        let data_text = data_resp.text()?;
        let data_json = data_text.trim_start_matches(")]}'\n");

        let trend_response: TrendResponse = serde_json::from_str(data_json)
            .map_err(|e| crate::error::PipelineError::ApiError(format!("Failed to parse trend data: {}", e)))?;

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
    request: String,
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
