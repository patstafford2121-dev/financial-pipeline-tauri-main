//! FRED (Federal Reserve Economic Data) fetcher
//!
//! Fetches macro economic data from FRED's public CSV endpoint.
//! FREE - no API key required for basic access!

use chrono::NaiveDate;
use csv::ReaderBuilder;
use reqwest::blocking::Client;

use crate::db::Database;
use crate::error::{PipelineError, Result};
use crate::models::MacroData;

/// FRED API client
pub struct Fred {
    client: Client,
}

impl Default for Fred {
    fn default() -> Self {
        Self::new()
    }
}

impl Fred {
    /// Create a new FRED client
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    /// Fetch macro data for an indicator
    ///
    /// # Arguments
    /// * `indicator` - FRED series ID (e.g., "GDP", "UNRATE", "DFF", "CPI")
    ///
    /// # Returns
    /// Vector of macro data records
    pub fn fetch_indicator(&self, indicator: &str) -> Result<Vec<MacroData>> {
        println!("[FETCH] Fetching {} from FRED...", indicator);

        // FRED CSV endpoint (no API key required)
        let url = format!(
            "https://fred.stlouisfed.org/graph/fredgraph.csv?id={}",
            indicator
        );

        let response = self.client.get(&url).send()?;

        if !response.status().is_success() {
            return Err(PipelineError::NoData(format!(
                "HTTP {} for {}",
                response.status(),
                indicator
            )));
        }

        let csv_text = response.text()?;

        // Parse CSV
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(csv_text.as_bytes());

        let mut data = Vec::new();

        for result in reader.records() {
            let record = result?;

            // First column is date, second is value
            if record.len() < 2 {
                continue;
            }

            let date_str = &record[0];
            let value_str = &record[1];

            // Skip missing values (FRED uses "." for missing)
            if value_str == "." || value_str.is_empty() {
                continue;
            }

            // Parse date (YYYY-MM-DD format)
            let date = match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                Ok(d) => d,
                Err(_) => continue,
            };

            // Parse value
            let value: f64 = match value_str.parse() {
                Ok(v) => v,
                Err(_) => continue,
            };

            data.push(MacroData {
                indicator: indicator.to_string(),
                date,
                value,
                source: "FRED".to_string(),
            });
        }

        println!("[OK] Fetched {} records for {}", data.len(), indicator);
        Ok(data)
    }

    /// Fetch and store indicator data directly to database
    pub fn fetch_and_store(&self, db: &mut Database, indicator: &str) -> Result<usize> {
        let data = self.fetch_indicator(indicator)?;
        let count = db.upsert_macro_data_batch(&data)?;
        db.log_api_call("FRED", "graph", indicator)?;
        println!("[OK] Stored {} records for {}", count, indicator);
        Ok(count)
    }

    /// Fetch multiple indicators
    pub fn fetch_batch(
        &self,
        db: &mut Database,
        indicators: &[&str],
    ) -> Result<(usize, usize)> {
        println!(
            "[FETCH] Batch fetching {} indicators from FRED...",
            indicators.len()
        );
        println!("{}", "=".repeat(60));

        let mut success_count = 0;
        let mut fail_count = 0;

        for (i, indicator) in indicators.iter().enumerate() {
            print!("\n[{}/{}] {}... ", i + 1, indicators.len(), indicator);

            match self.fetch_and_store(db, indicator) {
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
        println!("  Success: {}/{}", success_count, indicators.len());
        println!("  Failed: {}/{}", fail_count, indicators.len());

        Ok((success_count, fail_count))
    }
}

/// Common FRED indicators
pub mod indicators {
    /// Federal Funds Effective Rate (daily)
    pub const FED_FUNDS_RATE: &str = "DFF";

    /// Unemployment Rate (monthly)
    pub const UNEMPLOYMENT: &str = "UNRATE";

    /// Real GDP (quarterly)
    pub const GDP: &str = "GDP";

    /// Consumer Price Index (monthly)
    pub const CPI: &str = "CPIAUCSL";

    /// 10-Year Treasury Yield (daily)
    pub const TREASURY_10Y: &str = "DGS10";

    /// 2-Year Treasury Yield (daily)
    pub const TREASURY_2Y: &str = "DGS2";

    /// S&P 500 Index (daily)
    pub const SP500: &str = "SP500";

    /// VIX Volatility Index (daily)
    pub const VIX: &str = "VIXCLS";

    /// Personal Savings Rate (monthly)
    pub const SAVINGS_RATE: &str = "PSAVERT";

    /// Industrial Production Index (monthly)
    pub const INDUSTRIAL_PROD: &str = "INDPRO";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_fed_funds() {
        let client = Fred::new();
        let data = client.fetch_indicator(indicators::FED_FUNDS_RATE).unwrap();
        assert!(!data.is_empty());
        assert_eq!(data[0].indicator, "DFF");
    }
}
