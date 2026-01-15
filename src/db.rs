//! SQLite database layer for Financial Pipeline

use chrono::{NaiveDate, Utc};
use rusqlite::{params, Connection, Result as SqliteResult};
use std::path::Path;

use crate::error::Result;
use crate::models::{
    AlertCondition, BacktestResult, BacktestTrade, DailyPrice, IndicatorAlert,
    IndicatorAlertCondition, IndicatorAlertType, MacroData, PerformanceMetrics, Position,
    PositionType, PriceAlert, Signal, SignalDirection, SignalType, Strategy,
    StrategyConditionType, Symbol, TechnicalIndicator, TradeDirection,
};
use crate::trends::TrendData;

/// Database wrapper for financial data storage
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open or create database at the given path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(path)?;
        Ok(Self { conn })
    }

    /// Open an in-memory database (for testing)
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        Ok(Self { conn })
    }

    /// Initialize database schema
    pub fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(SCHEMA_SQL)?;
        println!("[OK] Database schema initialized");
        Ok(())
    }

    /// Insert or update a symbol
    pub fn upsert_symbol(&self, symbol: &Symbol) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT OR REPLACE INTO symbols
            (symbol, name, sector, industry, market_cap, country, exchange, currency, isin, asset_class)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            params![
                symbol.symbol,
                symbol.name,
                symbol.sector,
                symbol.industry,
                symbol.market_cap,
                symbol.country,
                symbol.exchange,
                symbol.currency,
                symbol.isin,
                symbol.asset_class,
            ],
        )?;
        Ok(())
    }

    /// Insert or update daily price data
    pub fn upsert_daily_price(&self, price: &DailyPrice) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT OR REPLACE INTO daily_prices
            (symbol, timestamp, open, high, low, close, volume, source)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            params![
                price.symbol,
                price.date.to_string(),
                price.open,
                price.high,
                price.low,
                price.close,
                price.volume,
                price.source,
            ],
        )?;
        Ok(())
    }

    /// Batch insert daily prices (more efficient)
    pub fn upsert_daily_prices(&mut self, prices: &[DailyPrice]) -> Result<usize> {
        let tx = self.conn.transaction()?;
        let mut count = 0;

        {
            let mut stmt = tx.prepare(
                r#"
                INSERT OR REPLACE INTO daily_prices
                (symbol, timestamp, open, high, low, close, volume, source)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                "#,
            )?;

            for price in prices {
                stmt.execute(params![
                    price.symbol,
                    price.date.to_string(),
                    price.open,
                    price.high,
                    price.low,
                    price.close,
                    price.volume,
                    price.source,
                ])?;
                count += 1;
            }
        }

        tx.commit()?;
        Ok(count)
    }

    /// Insert macro data
    pub fn upsert_macro_data(&self, data: &MacroData) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT OR REPLACE INTO macro_data (indicator, date, value, source)
            VALUES (?1, ?2, ?3, ?4)
            "#,
            params![data.indicator, data.date.to_string(), data.value, data.source,],
        )?;
        Ok(())
    }

    /// Batch insert macro data
    pub fn upsert_macro_data_batch(&mut self, data: &[MacroData]) -> Result<usize> {
        let tx = self.conn.transaction()?;
        let mut count = 0;

        {
            let mut stmt = tx.prepare(
                r#"
                INSERT OR REPLACE INTO macro_data (indicator, date, value, source)
                VALUES (?1, ?2, ?3, ?4)
                "#,
            )?;

            for d in data {
                stmt.execute(params![d.indicator, d.date.to_string(), d.value, d.source,])?;
                count += 1;
            }
        }

        tx.commit()?;
        Ok(count)
    }

    /// Get macro data for an indicator (latest values)
    pub fn get_macro_data(&self, indicator: &str) -> Result<Vec<MacroData>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT indicator, date, value, source
            FROM macro_data
            WHERE indicator = ?1
            ORDER BY date DESC
            LIMIT 100
            "#,
        )?;

        let data = stmt
            .query_map(params![indicator], |row| {
                let date_str: String = row.get(1)?;
                Ok(MacroData {
                    indicator: row.get(0)?,
                    date: NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                        .unwrap_or_else(|_| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()),
                    value: row.get(2)?,
                    source: row.get(3)?,
                })
            })?
            .collect::<SqliteResult<Vec<_>>>()?;

        Ok(data)
    }

    /// Get all unique macro indicators
    pub fn get_macro_indicators(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT DISTINCT indicator FROM macro_data ORDER BY indicator
            "#,
        )?;

        let indicators = stmt
            .query_map([], |row| row.get(0))?
            .collect::<SqliteResult<Vec<_>>>()?;

        Ok(indicators)
    }

    /// Get latest value for each macro indicator
    pub fn get_macro_summary(&self) -> Result<Vec<MacroData>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT m.indicator, m.date, m.value, m.source
            FROM macro_data m
            INNER JOIN (
                SELECT indicator, MAX(date) as max_date
                FROM macro_data
                GROUP BY indicator
            ) latest ON m.indicator = latest.indicator AND m.date = latest.max_date
            ORDER BY m.indicator
            "#,
        )?;

        let data = stmt
            .query_map([], |row| {
                let date_str: String = row.get(1)?;
                Ok(MacroData {
                    indicator: row.get(0)?,
                    date: NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                        .unwrap_or_else(|_| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()),
                    value: row.get(2)?,
                    source: row.get(3)?,
                })
            })?
            .collect::<SqliteResult<Vec<_>>>()?;

        Ok(data)
    }

    /// Log an API call
    pub fn log_api_call(&self, source: &str, endpoint: &str, symbol: &str) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT INTO api_calls (source, endpoint, symbol, timestamp)
            VALUES (?1, ?2, ?3, ?4)
            "#,
            params![source, endpoint, symbol, Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    /// Get latest price for a symbol
    pub fn get_latest_price(&self, symbol: &str) -> Result<Option<f64>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT close FROM daily_prices
            WHERE symbol = ?1
            ORDER BY timestamp DESC
            LIMIT 1
            "#,
        )?;

        let result: SqliteResult<f64> = stmt.query_row(params![symbol], |row| row.get(0));

        match result {
            Ok(price) => Ok(Some(price)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Get all prices for a symbol
    pub fn get_prices(&self, symbol: &str) -> Result<Vec<DailyPrice>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT symbol, timestamp, open, high, low, close, volume, source
            FROM daily_prices
            WHERE symbol = ?1
            ORDER BY timestamp ASC
            "#,
        )?;

        let prices = stmt
            .query_map(params![symbol], |row| {
                let date_str: String = row.get(1)?;
                Ok(DailyPrice {
                    symbol: row.get(0)?,
                    date: NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                        .unwrap_or_else(|_| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()),
                    open: row.get(2)?,
                    high: row.get(3)?,
                    low: row.get(4)?,
                    close: row.get(5)?,
                    volume: row.get(6)?,
                    source: row.get(7)?,
                })
            })?
            .collect::<SqliteResult<Vec<_>>>()?;

        Ok(prices)
    }

    /// Get all symbols with price data
    pub fn get_symbols_with_data(&self) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT DISTINCT symbol FROM daily_prices")?;
        let symbols = stmt
            .query_map([], |row| row.get(0))?
            .collect::<SqliteResult<Vec<_>>>()?;
        Ok(symbols)
    }

    /// Clear price data for a symbol
    pub fn clear_symbol_prices(&self, symbol: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM daily_prices WHERE symbol = ?1",
            params![symbol],
        )?;
        println!("[OK] Cleared price data for {}", symbol);
        Ok(())
    }

    /// Create a watchlist
    pub fn create_watchlist(
        &self,
        name: &str,
        symbols: &[String],
        description: Option<&str>,
    ) -> Result<i64> {
        // Delete existing watchlist entries
        self.conn
            .execute("DELETE FROM watchlists WHERE name = ?1", params![name])?;

        // Create watchlist
        self.conn.execute(
            "INSERT INTO watchlists (name, description) VALUES (?1, ?2)",
            params![name, description],
        )?;

        let watchlist_id = self.conn.last_insert_rowid();

        // Add symbols
        let mut stmt = self
            .conn
            .prepare("INSERT INTO watchlist_symbols (watchlist_id, symbol) VALUES (?1, ?2)")?;

        for symbol in symbols {
            stmt.execute(params![watchlist_id, symbol])?;
        }

        Ok(watchlist_id)
    }

    /// Get symbols in a watchlist
    pub fn get_watchlist(&self, name: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT ws.symbol
            FROM watchlists w
            JOIN watchlist_symbols ws ON w.id = ws.watchlist_id
            WHERE w.name = ?1
            "#,
        )?;

        let symbols = stmt
            .query_map(params![name], |row| row.get(0))?
            .collect::<SqliteResult<Vec<_>>>()?;

        Ok(symbols)
    }

    /// Vacuum/optimize the database
    pub fn vacuum(&self) -> Result<()> {
        self.conn.execute_batch("VACUUM; ANALYZE;")?;
        println!("[OK] Database optimized");
        Ok(())
    }

    /// Store a technical indicator value
    pub fn upsert_indicator(&self, ind: &TechnicalIndicator) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT OR REPLACE INTO technical_indicators
            (symbol, timestamp, indicator_name, value)
            VALUES (?1, ?2, ?3, ?4)
            "#,
            params![ind.symbol, ind.date.to_string(), ind.indicator_name, ind.value],
        )?;
        Ok(())
    }

    /// Batch store indicators
    pub fn upsert_indicators(&mut self, indicators: &[TechnicalIndicator]) -> Result<usize> {
        let tx = self.conn.transaction()?;
        let mut count = 0;

        {
            let mut stmt = tx.prepare(
                r#"
                INSERT OR REPLACE INTO technical_indicators
                (symbol, timestamp, indicator_name, value)
                VALUES (?1, ?2, ?3, ?4)
                "#,
            )?;

            for ind in indicators {
                stmt.execute(params![
                    ind.symbol,
                    ind.date.to_string(),
                    ind.indicator_name,
                    ind.value
                ])?;
                count += 1;
            }
        }

        tx.commit()?;
        Ok(count)
    }

    /// Get latest indicators for a symbol
    pub fn get_latest_indicators(&self, symbol: &str) -> Result<Vec<TechnicalIndicator>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT t.symbol, t.timestamp, t.indicator_name, t.value
            FROM technical_indicators t
            INNER JOIN (
                SELECT symbol, indicator_name, MAX(timestamp) as max_date
                FROM technical_indicators
                WHERE symbol = ?1
                GROUP BY symbol, indicator_name
            ) latest ON t.symbol = latest.symbol
                AND t.indicator_name = latest.indicator_name
                AND t.timestamp = latest.max_date
            "#,
        )?;

        let indicators = stmt
            .query_map(params![symbol], |row| {
                let date_str: String = row.get(1)?;
                Ok(TechnicalIndicator {
                    symbol: row.get(0)?,
                    date: NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                        .unwrap_or_else(|_| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()),
                    indicator_name: row.get(2)?,
                    value: row.get(3)?,
                })
            })?
            .collect::<SqliteResult<Vec<_>>>()?;

        Ok(indicators)
    }

    /// Get indicator history for a symbol
    pub fn get_indicator_history(
        &self,
        symbol: &str,
        indicator_name: &str,
    ) -> Result<Vec<TechnicalIndicator>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT symbol, timestamp, indicator_name, value
            FROM technical_indicators
            WHERE symbol = ?1 AND indicator_name = ?2
            ORDER BY timestamp ASC
            "#,
        )?;

        let indicators = stmt
            .query_map(params![symbol, indicator_name], |row| {
                let date_str: String = row.get(1)?;
                Ok(TechnicalIndicator {
                    symbol: row.get(0)?,
                    date: NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                        .unwrap_or_else(|_| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()),
                    indicator_name: row.get(2)?,
                    value: row.get(3)?,
                })
            })?
            .collect::<SqliteResult<Vec<_>>>()?;

        Ok(indicators)
    }

    /// Add a price alert
    pub fn add_alert(&self, symbol: &str, target_price: f64, condition: AlertCondition) -> Result<i64> {
        let condition_str = match condition {
            AlertCondition::Above => "above",
            AlertCondition::Below => "below",
        };

        self.conn.execute(
            r#"
            INSERT INTO price_alerts (symbol, target_price, condition)
            VALUES (?1, ?2, ?3)
            "#,
            params![symbol, target_price, condition_str],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Get all alerts (optionally filter by triggered status)
    pub fn get_alerts(&self, only_active: bool) -> Result<Vec<PriceAlert>> {
        let sql = if only_active {
            "SELECT id, symbol, target_price, condition, triggered, created_at FROM price_alerts WHERE triggered = 0 ORDER BY created_at DESC"
        } else {
            "SELECT id, symbol, target_price, condition, triggered, created_at FROM price_alerts ORDER BY created_at DESC"
        };

        let mut stmt = self.conn.prepare(sql)?;

        let alerts = stmt
            .query_map([], |row| {
                let condition_str: String = row.get(3)?;
                let condition = if condition_str == "above" {
                    AlertCondition::Above
                } else {
                    AlertCondition::Below
                };

                Ok(PriceAlert {
                    id: row.get(0)?,
                    symbol: row.get(1)?,
                    target_price: row.get(2)?,
                    condition,
                    triggered: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })?
            .collect::<SqliteResult<Vec<_>>>()?;

        Ok(alerts)
    }

    /// Delete an alert
    pub fn delete_alert(&self, alert_id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM price_alerts WHERE id = ?1", params![alert_id])?;
        Ok(())
    }

    /// Mark an alert as triggered
    pub fn trigger_alert(&self, alert_id: i64) -> Result<()> {
        self.conn.execute("UPDATE price_alerts SET triggered = 1 WHERE id = ?1", params![alert_id])?;
        Ok(())
    }

    /// Check alerts against current prices, returns triggered alerts
    pub fn check_alerts(&self) -> Result<Vec<PriceAlert>> {
        let alerts = self.get_alerts(true)?;
        let mut triggered = Vec::new();

        for alert in alerts {
            if let Ok(Some(current_price)) = self.get_latest_price(&alert.symbol) {
                let should_trigger = match alert.condition {
                    AlertCondition::Above => current_price >= alert.target_price,
                    AlertCondition::Below => current_price <= alert.target_price,
                };

                if should_trigger {
                    self.trigger_alert(alert.id)?;
                    triggered.push(PriceAlert {
                        triggered: true,
                        ..alert
                    });
                }
            }
        }

        Ok(triggered)
    }

    /// Add a portfolio position
    pub fn add_position(
        &self,
        symbol: &str,
        quantity: f64,
        price: f64,
        position_type: PositionType,
        date: &str,
        notes: Option<&str>,
    ) -> Result<i64> {
        let type_str = match position_type {
            PositionType::Buy => "buy",
            PositionType::Sell => "sell",
        };

        self.conn.execute(
            r#"
            INSERT INTO portfolio_positions (symbol, quantity, price, position_type, date, notes)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            params![symbol, quantity, price, type_str, date, notes],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Get all portfolio positions
    pub fn get_positions(&self) -> Result<Vec<Position>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, symbol, quantity, price, position_type, date, notes
            FROM portfolio_positions
            ORDER BY date DESC
            "#,
        )?;

        let positions = stmt
            .query_map([], |row| {
                let type_str: String = row.get(4)?;
                let position_type = if type_str == "buy" {
                    PositionType::Buy
                } else {
                    PositionType::Sell
                };

                Ok(Position {
                    id: row.get(0)?,
                    symbol: row.get(1)?,
                    quantity: row.get(2)?,
                    price: row.get(3)?,
                    position_type,
                    date: row.get(5)?,
                    notes: row.get(6)?,
                })
            })?
            .collect::<SqliteResult<Vec<_>>>()?;

        Ok(positions)
    }

    /// Delete a portfolio position
    pub fn delete_position(&self, position_id: i64) -> Result<()> {
        self.conn.execute(
            "DELETE FROM portfolio_positions WHERE id = ?1",
            params![position_id],
        )?;
        Ok(())
    }

    /// Store Google Trends data
    pub fn upsert_trends(&mut self, data: &[TrendData]) -> Result<usize> {
        let tx = self.conn.transaction()?;
        let mut count = 0;

        {
            let mut stmt = tx.prepare(
                r#"
                INSERT OR REPLACE INTO trends_data (keyword, date, value)
                VALUES (?1, ?2, ?3)
                "#,
            )?;

            for point in data {
                stmt.execute(params![point.keyword, point.date.to_string(), point.value])?;
                count += 1;
            }
        }

        tx.commit()?;
        Ok(count)
    }

    /// Get trends data for a keyword
    pub fn get_trends(&self, keyword: &str) -> Result<Vec<TrendData>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT keyword, date, value
            FROM trends_data
            WHERE keyword = ?1
            ORDER BY date ASC
            "#,
        )?;

        let trends = stmt
            .query_map(params![keyword], |row| {
                let date_str: String = row.get(1)?;
                Ok(TrendData {
                    keyword: row.get(0)?,
                    date: NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                        .unwrap_or_else(|_| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()),
                    value: row.get(2)?,
                })
            })?
            .collect::<SqliteResult<Vec<_>>>()?;

        Ok(trends)
    }

    // ========================================================================
    // Signal Methods
    // ========================================================================

    /// Store a signal
    pub fn upsert_signal(&self, signal: &Signal) -> Result<i64> {
        self.conn.execute(
            r#"
            INSERT OR REPLACE INTO signals
            (symbol, signal_type, direction, strength, price_at_signal,
             triggered_by, trigger_value, timestamp, acknowledged)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            params![
                signal.symbol,
                signal.signal_type.as_str(),
                signal.direction.as_str(),
                signal.strength,
                signal.price_at_signal,
                signal.triggered_by,
                signal.trigger_value,
                signal.timestamp.to_string(),
                signal.acknowledged,
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Batch store signals
    pub fn upsert_signals(&mut self, signals: &[Signal]) -> Result<usize> {
        let tx = self.conn.transaction()?;
        let mut count = 0;

        {
            let mut stmt = tx.prepare(
                r#"
                INSERT OR REPLACE INTO signals
                (symbol, signal_type, direction, strength, price_at_signal,
                 triggered_by, trigger_value, timestamp, acknowledged)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                "#,
            )?;

            for signal in signals {
                stmt.execute(params![
                    signal.symbol,
                    signal.signal_type.as_str(),
                    signal.direction.as_str(),
                    signal.strength,
                    signal.price_at_signal,
                    signal.triggered_by,
                    signal.trigger_value,
                    signal.timestamp.to_string(),
                    signal.acknowledged,
                ])?;
                count += 1;
            }
        }

        tx.commit()?;
        Ok(count)
    }

    /// Get signals for a symbol
    pub fn get_signals(&self, symbol: &str, only_unacknowledged: bool) -> Result<Vec<Signal>> {
        let sql = if only_unacknowledged {
            r#"
            SELECT id, symbol, signal_type, direction, strength, price_at_signal,
                   triggered_by, trigger_value, timestamp, created_at, acknowledged
            FROM signals
            WHERE symbol = ?1 AND acknowledged = 0
            ORDER BY timestamp DESC
            "#
        } else {
            r#"
            SELECT id, symbol, signal_type, direction, strength, price_at_signal,
                   triggered_by, trigger_value, timestamp, created_at, acknowledged
            FROM signals
            WHERE symbol = ?1
            ORDER BY timestamp DESC
            "#
        };

        let mut stmt = self.conn.prepare(sql)?;

        let signals = stmt
            .query_map(params![symbol], |row| {
                let signal_type_str: String = row.get(2)?;
                let direction_str: String = row.get(3)?;
                let date_str: String = row.get(8)?;

                Ok(Signal {
                    id: row.get(0)?,
                    symbol: row.get(1)?,
                    signal_type: SignalType::from_str(&signal_type_str)
                        .unwrap_or(SignalType::RsiOversold),
                    direction: SignalDirection::from_str(&direction_str),
                    strength: row.get(4)?,
                    price_at_signal: row.get(5)?,
                    triggered_by: row.get(6)?,
                    trigger_value: row.get(7)?,
                    timestamp: NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                        .unwrap_or_else(|_| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()),
                    created_at: row.get(9)?,
                    acknowledged: row.get(10)?,
                })
            })?
            .collect::<SqliteResult<Vec<_>>>()?;

        Ok(signals)
    }

    /// Get recent signals across all symbols
    pub fn get_recent_signals(&self, limit: usize) -> Result<Vec<Signal>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, symbol, signal_type, direction, strength, price_at_signal,
                   triggered_by, trigger_value, timestamp, created_at, acknowledged
            FROM signals
            ORDER BY timestamp DESC, strength DESC
            LIMIT ?1
            "#,
        )?;

        let signals = stmt
            .query_map(params![limit as i64], |row| {
                let signal_type_str: String = row.get(2)?;
                let direction_str: String = row.get(3)?;
                let date_str: String = row.get(8)?;

                Ok(Signal {
                    id: row.get(0)?,
                    symbol: row.get(1)?,
                    signal_type: SignalType::from_str(&signal_type_str)
                        .unwrap_or(SignalType::RsiOversold),
                    direction: SignalDirection::from_str(&direction_str),
                    strength: row.get(4)?,
                    price_at_signal: row.get(5)?,
                    triggered_by: row.get(6)?,
                    trigger_value: row.get(7)?,
                    timestamp: NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                        .unwrap_or_else(|_| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()),
                    created_at: row.get(9)?,
                    acknowledged: row.get(10)?,
                })
            })?
            .collect::<SqliteResult<Vec<_>>>()?;

        Ok(signals)
    }

    /// Acknowledge a signal
    pub fn acknowledge_signal(&self, signal_id: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE signals SET acknowledged = 1 WHERE id = ?1",
            params![signal_id],
        )?;
        Ok(())
    }

    /// Acknowledge all signals for a symbol
    pub fn acknowledge_all_signals(&self, symbol: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE signals SET acknowledged = 1 WHERE symbol = ?1",
            params![symbol],
        )?;
        Ok(())
    }

    /// Delete old signals (cleanup)
    pub fn cleanup_old_signals(&self, days: i64) -> Result<usize> {
        let deleted = self.conn.execute(
            "DELETE FROM signals WHERE timestamp < date('now', ?1)",
            params![format!("-{} days", days)],
        )?;
        Ok(deleted)
    }

    /// Get all indicators for a symbol (for signal generation)
    pub fn get_all_indicators(&self, symbol: &str) -> Result<Vec<TechnicalIndicator>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT symbol, timestamp, indicator_name, value
            FROM technical_indicators
            WHERE symbol = ?1
            ORDER BY timestamp ASC
            "#,
        )?;

        let indicators = stmt
            .query_map(params![symbol], |row| {
                let date_str: String = row.get(1)?;
                Ok(TechnicalIndicator {
                    symbol: row.get(0)?,
                    date: NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                        .unwrap_or_else(|_| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()),
                    indicator_name: row.get(2)?,
                    value: row.get(3)?,
                })
            })?
            .collect::<SqliteResult<Vec<_>>>()?;

        Ok(indicators)
    }

    // ========================================================================
    // Indicator Alert Methods
    // ========================================================================

    /// Add an indicator alert
    pub fn add_indicator_alert(&self, alert: &IndicatorAlert) -> Result<i64> {
        self.conn.execute(
            r#"
            INSERT INTO indicator_alerts
            (symbol, alert_type, indicator_name, secondary_indicator, condition, threshold, message)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![
                alert.symbol,
                alert.alert_type.as_str(),
                alert.indicator_name,
                alert.secondary_indicator,
                alert.condition.as_str(),
                alert.threshold,
                alert.message,
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Get all indicator alerts
    pub fn get_indicator_alerts(&self, only_active: bool) -> Result<Vec<IndicatorAlert>> {
        let sql = if only_active {
            r#"
            SELECT id, symbol, alert_type, indicator_name, secondary_indicator,
                   condition, threshold, triggered, last_value, created_at, message
            FROM indicator_alerts
            WHERE triggered = 0
            ORDER BY created_at DESC
            "#
        } else {
            r#"
            SELECT id, symbol, alert_type, indicator_name, secondary_indicator,
                   condition, threshold, triggered, last_value, created_at, message
            FROM indicator_alerts
            ORDER BY created_at DESC
            "#
        };

        let mut stmt = self.conn.prepare(sql)?;

        let alerts = stmt
            .query_map([], |row| {
                let alert_type_str: String = row.get(2)?;
                let condition_str: String = row.get(5)?;

                Ok(IndicatorAlert {
                    id: row.get(0)?,
                    symbol: row.get(1)?,
                    alert_type: IndicatorAlertType::from_str(&alert_type_str)
                        .unwrap_or(IndicatorAlertType::Threshold),
                    indicator_name: row.get(3)?,
                    secondary_indicator: row.get(4)?,
                    condition: IndicatorAlertCondition::from_str(&condition_str)
                        .unwrap_or(IndicatorAlertCondition::CrossesAbove),
                    threshold: row.get(6)?,
                    triggered: row.get(7)?,
                    last_value: row.get(8)?,
                    created_at: row.get(9)?,
                    message: row.get(10)?,
                })
            })?
            .collect::<SqliteResult<Vec<_>>>()?;

        Ok(alerts)
    }

    /// Delete an indicator alert
    pub fn delete_indicator_alert(&self, alert_id: i64) -> Result<()> {
        self.conn.execute(
            "DELETE FROM indicator_alerts WHERE id = ?1",
            params![alert_id],
        )?;
        Ok(())
    }

    /// Mark an indicator alert as triggered
    pub fn trigger_indicator_alert(&self, alert_id: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE indicator_alerts SET triggered = 1 WHERE id = ?1",
            params![alert_id],
        )?;
        Ok(())
    }

    /// Update last_value for an indicator alert
    pub fn update_indicator_alert_state(&self, alert_id: i64, last_value: f64) -> Result<()> {
        self.conn.execute(
            "UPDATE indicator_alerts SET last_value = ?1 WHERE id = ?2",
            params![last_value, alert_id],
        )?;
        Ok(())
    }

    /// Get the latest value for a specific indicator
    pub fn get_latest_indicator_value(&self, symbol: &str, indicator_name: &str) -> Result<Option<f64>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT value FROM technical_indicators
            WHERE symbol = ?1 AND indicator_name = ?2
            ORDER BY timestamp DESC
            LIMIT 1
            "#,
        )?;

        let result: SqliteResult<f64> = stmt.query_row(params![symbol, indicator_name], |row| row.get(0));

        match result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Get the previous (second-to-last) indicator value
    pub fn get_previous_indicator_value(&self, symbol: &str, indicator_name: &str) -> Result<Option<f64>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT value FROM technical_indicators
            WHERE symbol = ?1 AND indicator_name = ?2
            ORDER BY timestamp DESC
            LIMIT 1 OFFSET 1
            "#,
        )?;

        let result: SqliteResult<f64> = stmt.query_row(params![symbol, indicator_name], |row| row.get(0));

        match result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Check all indicator alerts, returns triggered alerts
    pub fn check_indicator_alerts(&self) -> Result<Vec<IndicatorAlert>> {
        let alerts = self.get_indicator_alerts(true)?;
        let mut triggered_alerts = Vec::new();

        for alert in alerts {
            let current = self.get_latest_indicator_value(&alert.symbol, &alert.indicator_name)?;
            let previous = alert.last_value.or_else(|| {
                self.get_previous_indicator_value(&alert.symbol, &alert.indicator_name).ok().flatten()
            });

            let Some(current_val) = current else {
                continue;
            };

            let should_trigger = match alert.condition {
                IndicatorAlertCondition::CrossesAbove => {
                    if let (Some(prev), Some(threshold)) = (previous, alert.threshold) {
                        prev < threshold && current_val >= threshold
                    } else {
                        false
                    }
                }
                IndicatorAlertCondition::CrossesBelow => {
                    if let (Some(prev), Some(threshold)) = (previous, alert.threshold) {
                        prev > threshold && current_val <= threshold
                    } else {
                        false
                    }
                }
                IndicatorAlertCondition::BullishCrossover => {
                    if let Some(secondary) = &alert.secondary_indicator {
                        let secondary_current = self.get_latest_indicator_value(&alert.symbol, secondary)?;
                        let secondary_prev = self.get_previous_indicator_value(&alert.symbol, secondary)?;

                        match (previous, secondary_current, secondary_prev) {
                            (Some(prev_primary), Some(curr_sec), Some(prev_sec)) => {
                                prev_primary <= prev_sec && current_val > curr_sec
                            }
                            _ => false,
                        }
                    } else {
                        false
                    }
                }
                IndicatorAlertCondition::BearishCrossover => {
                    if let Some(secondary) = &alert.secondary_indicator {
                        let secondary_current = self.get_latest_indicator_value(&alert.symbol, secondary)?;
                        let secondary_prev = self.get_previous_indicator_value(&alert.symbol, secondary)?;

                        match (previous, secondary_current, secondary_prev) {
                            (Some(prev_primary), Some(curr_sec), Some(prev_sec)) => {
                                prev_primary >= prev_sec && current_val < curr_sec
                            }
                            _ => false,
                        }
                    } else {
                        false
                    }
                }
            };

            if should_trigger {
                self.trigger_indicator_alert(alert.id)?;
                triggered_alerts.push(IndicatorAlert {
                    triggered: true,
                    ..alert
                });
            } else {
                // Update last_value for next check
                self.update_indicator_alert_state(alert.id, current_val)?;
            }
        }

        Ok(triggered_alerts)
    }

    // ========================================================================
    // Backtest Methods
    // ========================================================================

    /// Save a strategy
    pub fn save_strategy(&self, strategy: &Strategy) -> Result<i64> {
        self.conn.execute(
            r#"
            INSERT OR REPLACE INTO strategies
            (name, description, entry_condition, entry_threshold,
             exit_condition, exit_threshold,
             stop_loss_percent, take_profit_percent, position_size_percent)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            params![
                strategy.name,
                strategy.description,
                strategy.entry_condition.as_str(),
                strategy.entry_threshold,
                strategy.exit_condition.as_str(),
                strategy.exit_threshold,
                strategy.stop_loss_percent,
                strategy.take_profit_percent,
                strategy.position_size_percent,
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Get all strategies
    pub fn get_strategies(&self) -> Result<Vec<Strategy>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, name, description, entry_condition, entry_threshold,
                   exit_condition, exit_threshold,
                   stop_loss_percent, take_profit_percent, position_size_percent, created_at
            FROM strategies
            ORDER BY name ASC
            "#,
        )?;

        let strategies = stmt
            .query_map([], |row| {
                let entry_cond_str: String = row.get(3)?;
                let exit_cond_str: String = row.get(5)?;

                Ok(Strategy {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    entry_condition: StrategyConditionType::from_str(&entry_cond_str)
                        .unwrap_or(StrategyConditionType::RsiOversold),
                    entry_threshold: row.get(4)?,
                    exit_condition: StrategyConditionType::from_str(&exit_cond_str)
                        .unwrap_or(StrategyConditionType::RsiOverbought),
                    exit_threshold: row.get(6)?,
                    stop_loss_percent: row.get(7)?,
                    take_profit_percent: row.get(8)?,
                    position_size_percent: row.get(9)?,
                    created_at: row.get(10)?,
                })
            })?
            .collect::<SqliteResult<Vec<_>>>()?;

        Ok(strategies)
    }

    /// Get a strategy by name
    pub fn get_strategy(&self, name: &str) -> Result<Option<Strategy>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, name, description, entry_condition, entry_threshold,
                   exit_condition, exit_threshold,
                   stop_loss_percent, take_profit_percent, position_size_percent, created_at
            FROM strategies
            WHERE name = ?1
            "#,
        )?;

        let result = stmt.query_row(params![name], |row| {
            let entry_cond_str: String = row.get(3)?;
            let exit_cond_str: String = row.get(5)?;

            Ok(Strategy {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                entry_condition: StrategyConditionType::from_str(&entry_cond_str)
                    .unwrap_or(StrategyConditionType::RsiOversold),
                entry_threshold: row.get(4)?,
                exit_condition: StrategyConditionType::from_str(&exit_cond_str)
                    .unwrap_or(StrategyConditionType::RsiOverbought),
                exit_threshold: row.get(6)?,
                stop_loss_percent: row.get(7)?,
                take_profit_percent: row.get(8)?,
                position_size_percent: row.get(9)?,
                created_at: row.get(10)?,
            })
        });

        match result {
            Ok(strategy) => Ok(Some(strategy)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Delete a strategy
    pub fn delete_strategy(&self, name: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM strategies WHERE name = ?1", params![name])?;
        Ok(())
    }

    /// Save a backtest result
    pub fn save_backtest_result(&self, result: &BacktestResult) -> Result<i64> {
        let tx = self.conn.unchecked_transaction()?;

        // Insert the backtest run
        tx.execute(
            r#"
            INSERT INTO backtest_runs
            (strategy_id, strategy_name, symbol, start_date, end_date,
             initial_capital, final_capital, total_return, total_return_dollars,
             max_drawdown, sharpe_ratio, win_rate, total_trades, winning_trades,
             losing_trades, avg_win_percent, avg_loss_percent, profit_factor,
             avg_trade_duration_days)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)
            "#,
            params![
                result.strategy_id,
                result.strategy_name,
                result.symbol,
                result.start_date.to_string(),
                result.end_date.to_string(),
                result.initial_capital,
                result.final_capital,
                result.metrics.total_return,
                result.metrics.total_return_dollars,
                result.metrics.max_drawdown,
                result.metrics.sharpe_ratio,
                result.metrics.win_rate,
                result.metrics.total_trades as i64,
                result.metrics.winning_trades as i64,
                result.metrics.losing_trades as i64,
                result.metrics.avg_win_percent,
                result.metrics.avg_loss_percent,
                result.metrics.profit_factor,
                result.metrics.avg_trade_duration_days,
            ],
        )?;

        let backtest_id = tx.last_insert_rowid();

        // Insert trades
        {
            let mut stmt = tx.prepare(
                r#"
                INSERT INTO backtest_trades
                (backtest_id, symbol, direction, entry_date, entry_price, entry_reason,
                 exit_date, exit_price, exit_reason, shares, profit_loss, profit_loss_percent)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                "#,
            )?;

            for trade in &result.trades {
                stmt.execute(params![
                    backtest_id,
                    trade.symbol,
                    trade.direction.as_str(),
                    trade.entry_date.to_string(),
                    trade.entry_price,
                    trade.entry_reason,
                    trade.exit_date.map(|d| d.to_string()),
                    trade.exit_price,
                    trade.exit_reason,
                    trade.shares,
                    trade.profit_loss,
                    trade.profit_loss_percent,
                ])?;
            }
        }

        tx.commit()?;
        Ok(backtest_id)
    }

    /// Get backtest history
    pub fn get_backtest_results(
        &self,
        strategy_name: Option<&str>,
        symbol: Option<&str>,
        limit: usize,
    ) -> Result<Vec<BacktestResult>> {
        let mut sql = String::from(
            r#"
            SELECT id, strategy_id, strategy_name, symbol, start_date, end_date,
                   initial_capital, final_capital, total_return, total_return_dollars,
                   max_drawdown, sharpe_ratio, win_rate, total_trades, winning_trades,
                   losing_trades, avg_win_percent, avg_loss_percent, profit_factor,
                   avg_trade_duration_days, created_at
            FROM backtest_runs
            WHERE 1=1
            "#,
        );

        if strategy_name.is_some() {
            sql.push_str(" AND strategy_name = ?1");
        }
        if symbol.is_some() {
            sql.push_str(if strategy_name.is_some() {
                " AND symbol = ?2"
            } else {
                " AND symbol = ?1"
            });
        }

        sql.push_str(" ORDER BY created_at DESC LIMIT ?");

        let mut stmt = self.conn.prepare(&sql)?;

        let results: Vec<BacktestResult> = match (strategy_name, symbol) {
            (Some(strat), Some(sym)) => {
                stmt.query_map(params![strat, sym, limit as i64], |row| self.map_backtest_row(row))?
                    .collect::<SqliteResult<Vec<_>>>()?
            }
            (Some(strat), None) => {
                stmt.query_map(params![strat, limit as i64], |row| self.map_backtest_row(row))?
                    .collect::<SqliteResult<Vec<_>>>()?
            }
            (None, Some(sym)) => {
                stmt.query_map(params![sym, limit as i64], |row| self.map_backtest_row(row))?
                    .collect::<SqliteResult<Vec<_>>>()?
            }
            (None, None) => {
                stmt.query_map(params![limit as i64], |row| self.map_backtest_row(row))?
                    .collect::<SqliteResult<Vec<_>>>()?
            }
        };

        Ok(results)
    }

    fn map_backtest_row(&self, row: &rusqlite::Row) -> SqliteResult<BacktestResult> {
        let start_str: String = row.get(4)?;
        let end_str: String = row.get(5)?;
        let total_trades_i64: i64 = row.get(13)?;
        let winning_trades_i64: i64 = row.get(14)?;
        let losing_trades_i64: i64 = row.get(15)?;

        Ok(BacktestResult {
            id: row.get(0)?,
            strategy_id: row.get(1)?,
            strategy_name: row.get(2)?,
            symbol: row.get(3)?,
            start_date: NaiveDate::parse_from_str(&start_str, "%Y-%m-%d")
                .unwrap_or_else(|_| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()),
            end_date: NaiveDate::parse_from_str(&end_str, "%Y-%m-%d")
                .unwrap_or_else(|_| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()),
            initial_capital: row.get(6)?,
            final_capital: row.get(7)?,
            metrics: PerformanceMetrics {
                total_return: row.get(8)?,
                total_return_dollars: row.get(9)?,
                max_drawdown: row.get(10)?,
                sharpe_ratio: row.get(11)?,
                win_rate: row.get(12)?,
                total_trades: total_trades_i64 as usize,
                winning_trades: winning_trades_i64 as usize,
                losing_trades: losing_trades_i64 as usize,
                avg_win_percent: row.get(16)?,
                avg_loss_percent: row.get(17)?,
                profit_factor: row.get(18)?,
                avg_trade_duration_days: row.get(19)?,
            },
            trades: Vec::new(), // Trades loaded separately if needed
            created_at: row.get(20)?,
        })
    }

    /// Get backtest detail with trades
    pub fn get_backtest_detail(&self, backtest_id: i64) -> Result<Option<BacktestResult>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, strategy_id, strategy_name, symbol, start_date, end_date,
                   initial_capital, final_capital, total_return, total_return_dollars,
                   max_drawdown, sharpe_ratio, win_rate, total_trades, winning_trades,
                   losing_trades, avg_win_percent, avg_loss_percent, profit_factor,
                   avg_trade_duration_days, created_at
            FROM backtest_runs
            WHERE id = ?1
            "#,
        )?;

        let result = stmt.query_row(params![backtest_id], |row| self.map_backtest_row(row));

        let mut backtest = match result {
            Ok(b) => b,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
            Err(e) => return Err(e.into()),
        };

        // Load trades
        let mut trade_stmt = self.conn.prepare(
            r#"
            SELECT id, backtest_id, symbol, direction, entry_date, entry_price, entry_reason,
                   exit_date, exit_price, exit_reason, shares, profit_loss, profit_loss_percent
            FROM backtest_trades
            WHERE backtest_id = ?1
            ORDER BY entry_date ASC
            "#,
        )?;

        let trades = trade_stmt
            .query_map(params![backtest_id], |row| {
                let dir_str: String = row.get(3)?;
                let entry_str: String = row.get(4)?;
                let exit_str: Option<String> = row.get(7)?;

                Ok(BacktestTrade {
                    id: row.get(0)?,
                    backtest_id: row.get(1)?,
                    symbol: row.get(2)?,
                    direction: TradeDirection::from_str(&dir_str),
                    entry_date: NaiveDate::parse_from_str(&entry_str, "%Y-%m-%d")
                        .unwrap_or_else(|_| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()),
                    entry_price: row.get(5)?,
                    entry_reason: row.get(6)?,
                    exit_date: exit_str.and_then(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()),
                    exit_price: row.get(8)?,
                    exit_reason: row.get(9)?,
                    shares: row.get(10)?,
                    profit_loss: row.get(11)?,
                    profit_loss_percent: row.get(12)?,
                })
            })?
            .collect::<SqliteResult<Vec<_>>>()?;

        backtest.trades = trades;

        Ok(Some(backtest))
    }

    /// Delete a backtest result and its trades
    pub fn delete_backtest(&self, backtest_id: i64) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            "DELETE FROM backtest_trades WHERE backtest_id = ?1",
            params![backtest_id],
        )?;
        tx.execute(
            "DELETE FROM backtest_runs WHERE id = ?1",
            params![backtest_id],
        )?;
        tx.commit()?;
        Ok(())
    }
}

/// Database schema SQL
const SCHEMA_SQL: &str = r#"
-- Symbol master table
CREATE TABLE IF NOT EXISTS symbols (
    symbol TEXT PRIMARY KEY,
    name TEXT,
    sector TEXT,
    industry TEXT,
    market_cap REAL,
    country TEXT,
    exchange TEXT,
    currency TEXT,
    isin TEXT,
    asset_class TEXT,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Daily price data
CREATE TABLE IF NOT EXISTS daily_prices (
    symbol TEXT,
    timestamp DATE,
    open REAL,
    high REAL,
    low REAL,
    close REAL,
    volume INTEGER,
    adjusted_close REAL,
    source TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (symbol, timestamp)
);

-- Macro economic indicators
CREATE TABLE IF NOT EXISTS macro_data (
    indicator TEXT,
    date DATE,
    value REAL,
    frequency TEXT,
    source TEXT DEFAULT 'FRED',
    PRIMARY KEY (indicator, date)
);

-- Watchlists
CREATE TABLE IF NOT EXISTS watchlists (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE,
    description TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS watchlist_symbols (
    watchlist_id INTEGER,
    symbol TEXT,
    added_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    notes TEXT,
    PRIMARY KEY (watchlist_id, symbol),
    FOREIGN KEY (watchlist_id) REFERENCES watchlists(id)
);

-- API call tracking
CREATE TABLE IF NOT EXISTS api_calls (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source TEXT,
    endpoint TEXT,
    symbol TEXT,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    success BOOLEAN,
    error_message TEXT
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_prices_symbol ON daily_prices(symbol);
CREATE INDEX IF NOT EXISTS idx_prices_timestamp ON daily_prices(timestamp);
CREATE INDEX IF NOT EXISTS idx_prices_source ON daily_prices(source);
CREATE INDEX IF NOT EXISTS idx_symbols_sector ON symbols(sector);
CREATE INDEX IF NOT EXISTS idx_macro_indicator ON macro_data(indicator);
CREATE INDEX IF NOT EXISTS idx_macro_date ON macro_data(date);
CREATE INDEX IF NOT EXISTS idx_api_calls_source ON api_calls(source);
CREATE INDEX IF NOT EXISTS idx_api_calls_timestamp ON api_calls(timestamp);

-- Views
CREATE VIEW IF NOT EXISTS latest_prices AS
SELECT p.*
FROM daily_prices p
INNER JOIN (
    SELECT symbol, MAX(timestamp) as max_date
    FROM daily_prices
    GROUP BY symbol
) latest ON p.symbol = latest.symbol AND p.timestamp = latest.max_date;

CREATE VIEW IF NOT EXISTS api_rate_limits AS
SELECT
    source,
    COUNT(*) as calls_today,
    MAX(timestamp) as last_call
FROM api_calls
WHERE DATE(timestamp) = DATE('now')
GROUP BY source;

-- Technical indicators
CREATE TABLE IF NOT EXISTS technical_indicators (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol TEXT NOT NULL,
    timestamp DATE NOT NULL,
    indicator_name TEXT NOT NULL,
    value REAL NOT NULL,
    params TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(symbol, timestamp, indicator_name)
);

CREATE INDEX IF NOT EXISTS idx_ti_symbol_date ON technical_indicators(symbol, timestamp);
CREATE INDEX IF NOT EXISTS idx_ti_indicator ON technical_indicators(indicator_name);

-- Price alerts
CREATE TABLE IF NOT EXISTS price_alerts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol TEXT NOT NULL,
    target_price REAL NOT NULL,
    condition TEXT NOT NULL CHECK(condition IN ('above', 'below')),
    triggered BOOLEAN DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_alerts_symbol ON price_alerts(symbol);
CREATE INDEX IF NOT EXISTS idx_alerts_triggered ON price_alerts(triggered);

-- Portfolio positions
CREATE TABLE IF NOT EXISTS portfolio_positions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol TEXT NOT NULL,
    quantity REAL NOT NULL,
    price REAL NOT NULL,
    position_type TEXT NOT NULL CHECK(position_type IN ('buy', 'sell')),
    date TEXT NOT NULL,
    notes TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_positions_symbol ON portfolio_positions(symbol);

-- Google Trends data
CREATE TABLE IF NOT EXISTS trends_data (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    keyword TEXT NOT NULL,
    date DATE NOT NULL,
    value INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(keyword, date)
);

CREATE INDEX IF NOT EXISTS idx_trends_keyword ON trends_data(keyword);
CREATE INDEX IF NOT EXISTS idx_trends_date ON trends_data(date);

-- Trading signals
CREATE TABLE IF NOT EXISTS signals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol TEXT NOT NULL,
    signal_type TEXT NOT NULL,
    direction TEXT NOT NULL CHECK(direction IN ('bullish', 'bearish', 'neutral')),
    strength REAL NOT NULL CHECK(strength >= 0.0 AND strength <= 1.0),
    price_at_signal REAL NOT NULL,
    triggered_by TEXT NOT NULL,
    trigger_value REAL NOT NULL,
    timestamp DATE NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    acknowledged BOOLEAN DEFAULT 0,
    UNIQUE(symbol, signal_type, timestamp)
);

CREATE INDEX IF NOT EXISTS idx_signals_symbol ON signals(symbol);
CREATE INDEX IF NOT EXISTS idx_signals_type ON signals(signal_type);
CREATE INDEX IF NOT EXISTS idx_signals_timestamp ON signals(timestamp);
CREATE INDEX IF NOT EXISTS idx_signals_direction ON signals(direction);
CREATE INDEX IF NOT EXISTS idx_signals_acknowledged ON signals(acknowledged);

-- Indicator-based alerts
CREATE TABLE IF NOT EXISTS indicator_alerts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol TEXT NOT NULL,
    alert_type TEXT NOT NULL CHECK(alert_type IN ('threshold', 'crossover', 'band_touch')),
    indicator_name TEXT NOT NULL,
    secondary_indicator TEXT,
    condition TEXT NOT NULL CHECK(condition IN (
        'crosses_above', 'crosses_below', 'bullish_crossover', 'bearish_crossover'
    )),
    threshold REAL,
    triggered BOOLEAN DEFAULT 0,
    last_value REAL,
    message TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_ind_alerts_symbol ON indicator_alerts(symbol);
CREATE INDEX IF NOT EXISTS idx_ind_alerts_triggered ON indicator_alerts(triggered);

-- Backtesting strategies
CREATE TABLE IF NOT EXISTS strategies (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL,
    description TEXT,
    entry_condition TEXT NOT NULL,
    entry_threshold REAL NOT NULL,
    exit_condition TEXT NOT NULL,
    exit_threshold REAL NOT NULL,
    stop_loss_percent REAL,
    take_profit_percent REAL,
    position_size_percent REAL NOT NULL DEFAULT 100.0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_strategies_name ON strategies(name);

-- Backtest runs
CREATE TABLE IF NOT EXISTS backtest_runs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    strategy_id INTEGER NOT NULL,
    strategy_name TEXT NOT NULL,
    symbol TEXT NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    initial_capital REAL NOT NULL,
    final_capital REAL NOT NULL,
    total_return REAL NOT NULL,
    total_return_dollars REAL NOT NULL,
    max_drawdown REAL NOT NULL,
    sharpe_ratio REAL NOT NULL,
    win_rate REAL NOT NULL,
    total_trades INTEGER NOT NULL,
    winning_trades INTEGER NOT NULL,
    losing_trades INTEGER NOT NULL,
    avg_win_percent REAL NOT NULL,
    avg_loss_percent REAL NOT NULL,
    profit_factor REAL NOT NULL,
    avg_trade_duration_days REAL NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (strategy_id) REFERENCES strategies(id)
);

CREATE INDEX IF NOT EXISTS idx_backtest_runs_strategy ON backtest_runs(strategy_id);
CREATE INDEX IF NOT EXISTS idx_backtest_runs_symbol ON backtest_runs(symbol);
CREATE INDEX IF NOT EXISTS idx_backtest_runs_date ON backtest_runs(created_at);

-- Backtest trades
CREATE TABLE IF NOT EXISTS backtest_trades (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    backtest_id INTEGER NOT NULL,
    symbol TEXT NOT NULL,
    direction TEXT NOT NULL CHECK(direction IN ('long', 'short')),
    entry_date DATE NOT NULL,
    entry_price REAL NOT NULL,
    entry_reason TEXT NOT NULL,
    exit_date DATE,
    exit_price REAL,
    exit_reason TEXT,
    shares REAL NOT NULL,
    profit_loss REAL,
    profit_loss_percent REAL,
    FOREIGN KEY (backtest_id) REFERENCES backtest_runs(id)
);

CREATE INDEX IF NOT EXISTS idx_backtest_trades_run ON backtest_trades(backtest_id);
CREATE INDEX IF NOT EXISTS idx_backtest_trades_symbol ON backtest_trades(symbol);
"#;
