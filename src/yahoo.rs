//! Yahoo Finance data fetcher
//!
//! Uses Yahoo Finance's public API to fetch stock price data.
//! FREE and UNLIMITED - no API key required!

use chrono::{DateTime, Utc};
use reqwest::blocking::Client;

use crate::db::Database;
use crate::error::{PipelineError, Result};
use crate::models::yahoo::ChartResponse;
use crate::models::DailyPrice;

/// Yahoo Finance API client
pub struct YahooFinance {
    client: Client,
}

impl Default for YahooFinance {
    fn default() -> Self {
        Self::new()
    }
}

impl YahooFinance {
    /// Create a new Yahoo Finance client
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    /// Fetch daily prices for a symbol
    ///
    /// # Arguments
    /// * `symbol` - Stock ticker symbol (e.g., "AAPL", "MSFT")
    /// * `period` - Time period: "1d", "5d", "1mo", "3mo", "6mo", "1y", "2y", "5y", "10y", "ytd", "max"
    ///
    /// # Returns
    /// Vector of daily price records
    pub fn fetch_prices(&self, symbol: &str, period: &str) -> Result<Vec<DailyPrice>> {
        println!(
            "[FETCH] Fetching {} from Yahoo Finance (period: {})...",
            symbol, period
        );

        // Yahoo Finance API endpoint
        let url = format!(
            "https://query1.finance.yahoo.com/v8/finance/chart/{}?interval=1d&range={}",
            symbol, period
        );

        let response = self.client.get(&url).send()?;

        if !response.status().is_success() {
            return Err(PipelineError::NoData(format!(
                "HTTP {} for {}",
                response.status(),
                symbol
            )));
        }

        let chart_response: ChartResponse = response.json()?;

        // Check for API errors
        if let Some(chart) = &chart_response.chart.result {
            if chart.is_empty() {
                return Err(PipelineError::NoData(symbol.to_string()));
            }
        } else if let Some(err) = &chart_response.chart.error {
            return Err(PipelineError::NoData(format!(
                "{}: {}",
                err.code, err.description
            )));
        }

        let result = chart_response
            .chart
            .result
            .ok_or_else(|| PipelineError::NoData(symbol.to_string()))?;

        let data = &result[0];
        let timestamps = data
            .timestamp
            .as_ref()
            .ok_or_else(|| PipelineError::NoData(symbol.to_string()))?;

        let quote = &data.indicators.quote[0];

        let mut prices = Vec::with_capacity(timestamps.len());

        for (i, &ts) in timestamps.iter().enumerate() {
            // Skip if any value is None
            let open = match quote.open.get(i).and_then(|v| *v) {
                Some(v) => v,
                None => continue,
            };
            let high = match quote.high.get(i).and_then(|v| *v) {
                Some(v) => v,
                None => continue,
            };
            let low = match quote.low.get(i).and_then(|v| *v) {
                Some(v) => v,
                None => continue,
            };
            let close = match quote.close.get(i).and_then(|v| *v) {
                Some(v) => v,
                None => continue,
            };
            let volume = quote.volume.get(i).and_then(|v| *v).unwrap_or(0);

            // Convert Unix timestamp to date
            let datetime = DateTime::from_timestamp(ts, 0)
                .unwrap_or_else(|| Utc::now());
            let date = datetime.date_naive();

            prices.push(DailyPrice {
                symbol: symbol.to_string(),
                date,
                open,
                high,
                low,
                close,
                volume,
                source: "yahoo_finance".to_string(),
            });
        }

        println!("[OK] Fetched {} records for {}", prices.len(), symbol);
        Ok(prices)
    }

    /// Fetch and store prices directly to database
    pub fn fetch_and_store(
        &self,
        db: &mut Database,
        symbol: &str,
        period: &str,
    ) -> Result<usize> {
        let prices = self.fetch_prices(symbol, period)?;
        let count = db.upsert_daily_prices(&prices)?;
        db.log_api_call("yahoo_finance", "history", symbol)?;
        println!("[OK] Stored {} records for {}", count, symbol);
        Ok(count)
    }

    /// Batch fetch multiple symbols
    pub fn fetch_batch(
        &self,
        db: &mut Database,
        symbols: &[String],
        period: &str,
    ) -> Result<(usize, usize)> {
        println!(
            "[FETCH] Batch fetching {} symbols from Yahoo Finance...",
            symbols.len()
        );
        println!("Period: {}", period);
        println!("{}", "=".repeat(60));

        let mut success_count = 0;
        let mut fail_count = 0;

        for (i, symbol) in symbols.iter().enumerate() {
            print!("\n[{}/{}] {}... ", i + 1, symbols.len(), symbol);

            match self.fetch_and_store(db, symbol, period) {
                Ok(_) => {
                    success_count += 1;
                    println!("[OK]");
                }
                Err(e) => {
                    fail_count += 1;
                    println!("[FAIL] {}", e);
                }
            }
        }

        println!("\n{}", "=".repeat(60));
        println!("[OK] Batch fetch complete!");
        println!("  Success: {}/{}", success_count, symbols.len());
        println!("  Failed: {}/{}", fail_count, symbols.len());

        Ok((success_count, fail_count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_aapl() {
        let client = YahooFinance::new();
        let prices = client.fetch_prices("AAPL", "5d").unwrap();
        assert!(!prices.is_empty());
        assert_eq!(prices[0].symbol, "AAPL");
    }
}
