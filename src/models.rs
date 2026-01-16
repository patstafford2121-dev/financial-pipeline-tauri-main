//! Data models for Financial Pipeline

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Stock symbol metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub symbol: String,
    pub name: Option<String>,
    pub sector: Option<String>,
    pub industry: Option<String>,
    pub market_cap: Option<f64>,
    pub country: Option<String>,
    pub exchange: Option<String>,
    pub currency: Option<String>,
    pub isin: Option<String>,
    pub asset_class: Option<String>,
}

/// Daily price data (OHLCV)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyPrice {
    pub symbol: String,
    pub date: NaiveDate,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: i64,
    pub source: String,
}

/// Macro economic indicator data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroData {
    pub indicator: String,
    pub date: NaiveDate,
    pub value: f64,
    pub source: String,
}

/// Watchlist definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Watchlist {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
}

/// API call log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiCall {
    pub id: i64,
    pub source: String,
    pub endpoint: String,
    pub symbol: String,
    pub timestamp: String,
}

/// Technical indicator value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalIndicator {
    pub symbol: String,
    pub date: NaiveDate,
    pub indicator_name: String,
    pub value: f64,
}

/// Price alert condition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertCondition {
    Above,
    Below,
}

/// Price alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceAlert {
    pub id: i64,
    pub symbol: String,
    pub target_price: f64,
    pub condition: AlertCondition,
    pub triggered: bool,
    pub created_at: String,
}

/// Position type (buy or sell/short)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PositionType {
    Buy,
    Sell,
}

/// Portfolio position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub id: i64,
    pub symbol: String,
    pub quantity: f64,
    pub price: f64,
    pub position_type: PositionType,
    pub date: String,
    pub notes: Option<String>,
}

// ============================================================================
// Signal Generation Types
// ============================================================================

/// Type of trading signal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalType {
    // RSI signals
    RsiOverbought,
    RsiOversold,
    // MACD signals
    MacdBullishCross,
    MacdBearishCross,
    // Bollinger Band signals
    BollingerUpperBreak,
    BollingerLowerBreak,
    // Moving Average signals
    MaCrossoverBullish,
    MaCrossoverBearish,
    // ADX signals
    AdxTrendStrong,
    AdxTrendWeak,
    // Stochastic signals
    StochBullishCross,
    StochBearishCross,
    // Williams %R signals
    WillrOverbought,
    WillrOversold,
    // CCI signals
    CciOverbought,
    CciOversold,
    // MFI signals
    MfiOverbought,
    MfiOversold,
}

impl SignalType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SignalType::RsiOverbought => "RSI_OVERBOUGHT",
            SignalType::RsiOversold => "RSI_OVERSOLD",
            SignalType::MacdBullishCross => "MACD_BULLISH_CROSS",
            SignalType::MacdBearishCross => "MACD_BEARISH_CROSS",
            SignalType::BollingerUpperBreak => "BB_UPPER_BREAK",
            SignalType::BollingerLowerBreak => "BB_LOWER_BREAK",
            SignalType::MaCrossoverBullish => "MA_BULLISH_CROSS",
            SignalType::MaCrossoverBearish => "MA_BEARISH_CROSS",
            SignalType::AdxTrendStrong => "ADX_TREND_STRONG",
            SignalType::AdxTrendWeak => "ADX_TREND_WEAK",
            SignalType::StochBullishCross => "STOCH_BULLISH_CROSS",
            SignalType::StochBearishCross => "STOCH_BEARISH_CROSS",
            SignalType::WillrOverbought => "WILLR_OVERBOUGHT",
            SignalType::WillrOversold => "WILLR_OVERSOLD",
            SignalType::CciOverbought => "CCI_OVERBOUGHT",
            SignalType::CciOversold => "CCI_OVERSOLD",
            SignalType::MfiOverbought => "MFI_OVERBOUGHT",
            SignalType::MfiOversold => "MFI_OVERSOLD",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "RSI_OVERBOUGHT" => Some(SignalType::RsiOverbought),
            "RSI_OVERSOLD" => Some(SignalType::RsiOversold),
            "MACD_BULLISH_CROSS" => Some(SignalType::MacdBullishCross),
            "MACD_BEARISH_CROSS" => Some(SignalType::MacdBearishCross),
            "BB_UPPER_BREAK" => Some(SignalType::BollingerUpperBreak),
            "BB_LOWER_BREAK" => Some(SignalType::BollingerLowerBreak),
            "MA_BULLISH_CROSS" => Some(SignalType::MaCrossoverBullish),
            "MA_BEARISH_CROSS" => Some(SignalType::MaCrossoverBearish),
            "ADX_TREND_STRONG" => Some(SignalType::AdxTrendStrong),
            "ADX_TREND_WEAK" => Some(SignalType::AdxTrendWeak),
            "STOCH_BULLISH_CROSS" => Some(SignalType::StochBullishCross),
            "STOCH_BEARISH_CROSS" => Some(SignalType::StochBearishCross),
            "WILLR_OVERBOUGHT" => Some(SignalType::WillrOverbought),
            "WILLR_OVERSOLD" => Some(SignalType::WillrOversold),
            "CCI_OVERBOUGHT" => Some(SignalType::CciOverbought),
            "CCI_OVERSOLD" => Some(SignalType::CciOversold),
            "MFI_OVERBOUGHT" => Some(SignalType::MfiOverbought),
            "MFI_OVERSOLD" => Some(SignalType::MfiOversold),
            _ => None,
        }
    }
}

/// Direction of the signal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalDirection {
    Bullish,
    Bearish,
    Neutral,
}

impl SignalDirection {
    pub fn as_str(&self) -> &'static str {
        match self {
            SignalDirection::Bullish => "bullish",
            SignalDirection::Bearish => "bearish",
            SignalDirection::Neutral => "neutral",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "bullish" => SignalDirection::Bullish,
            "bearish" => SignalDirection::Bearish,
            _ => SignalDirection::Neutral,
        }
    }
}

/// A generated trading signal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    pub id: i64,
    pub symbol: String,
    pub signal_type: SignalType,
    pub direction: SignalDirection,
    pub strength: f64,
    pub price_at_signal: f64,
    pub triggered_by: String,
    pub trigger_value: f64,
    pub timestamp: NaiveDate,
    pub created_at: String,
    pub acknowledged: bool,
}

// ============================================================================
// Indicator Alert Types
// ============================================================================

/// Type of indicator alert
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndicatorAlertType {
    Threshold,    // RSI crosses 30, ADX crosses 25, etc.
    Crossover,    // MACD crosses signal, SMA20 crosses SMA50
    BandTouch,    // Price touches Bollinger bands
}

impl IndicatorAlertType {
    pub fn as_str(&self) -> &'static str {
        match self {
            IndicatorAlertType::Threshold => "threshold",
            IndicatorAlertType::Crossover => "crossover",
            IndicatorAlertType::BandTouch => "band_touch",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "threshold" => Some(IndicatorAlertType::Threshold),
            "crossover" => Some(IndicatorAlertType::Crossover),
            "band_touch" => Some(IndicatorAlertType::BandTouch),
            _ => None,
        }
    }
}

/// Condition for indicator alerts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndicatorAlertCondition {
    CrossesAbove,
    CrossesBelow,
    BullishCrossover,
    BearishCrossover,
}

impl IndicatorAlertCondition {
    pub fn as_str(&self) -> &'static str {
        match self {
            IndicatorAlertCondition::CrossesAbove => "crosses_above",
            IndicatorAlertCondition::CrossesBelow => "crosses_below",
            IndicatorAlertCondition::BullishCrossover => "bullish_crossover",
            IndicatorAlertCondition::BearishCrossover => "bearish_crossover",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "crosses_above" => Some(IndicatorAlertCondition::CrossesAbove),
            "crosses_below" => Some(IndicatorAlertCondition::CrossesBelow),
            "bullish_crossover" => Some(IndicatorAlertCondition::BullishCrossover),
            "bearish_crossover" => Some(IndicatorAlertCondition::BearishCrossover),
            _ => None,
        }
    }
}

/// An indicator-based alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorAlert {
    pub id: i64,
    pub symbol: String,
    pub alert_type: IndicatorAlertType,
    pub indicator_name: String,
    pub secondary_indicator: Option<String>,
    pub condition: IndicatorAlertCondition,
    pub threshold: Option<f64>,
    pub triggered: bool,
    pub last_value: Option<f64>,
    pub created_at: String,
    pub message: Option<String>,
}

// ============================================================================
// Backtesting Types
// ============================================================================

/// Strategy entry/exit condition type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrategyConditionType {
    RsiOversold,      // RSI < threshold (buy signal)
    RsiOverbought,    // RSI > threshold (sell signal)
    MacdCrossUp,      // MACD crosses above signal line
    MacdCrossDown,    // MACD crosses below signal line
    PriceAboveSma,    // Price > SMA
    PriceBelowSma,    // Price < SMA
    SmaCrossUp,       // Fast SMA crosses above slow SMA
    SmaCrossDown,     // Fast SMA crosses below slow SMA
    StopLoss,         // Price falls below entry - threshold%
    TakeProfit,       // Price rises above entry + threshold%
}

impl StrategyConditionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            StrategyConditionType::RsiOversold => "rsi_oversold",
            StrategyConditionType::RsiOverbought => "rsi_overbought",
            StrategyConditionType::MacdCrossUp => "macd_cross_up",
            StrategyConditionType::MacdCrossDown => "macd_cross_down",
            StrategyConditionType::PriceAboveSma => "price_above_sma",
            StrategyConditionType::PriceBelowSma => "price_below_sma",
            StrategyConditionType::SmaCrossUp => "sma_cross_up",
            StrategyConditionType::SmaCrossDown => "sma_cross_down",
            StrategyConditionType::StopLoss => "stop_loss",
            StrategyConditionType::TakeProfit => "take_profit",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "rsi_oversold" => Some(StrategyConditionType::RsiOversold),
            "rsi_overbought" => Some(StrategyConditionType::RsiOverbought),
            "macd_cross_up" => Some(StrategyConditionType::MacdCrossUp),
            "macd_cross_down" => Some(StrategyConditionType::MacdCrossDown),
            "price_above_sma" => Some(StrategyConditionType::PriceAboveSma),
            "price_below_sma" => Some(StrategyConditionType::PriceBelowSma),
            "sma_cross_up" => Some(StrategyConditionType::SmaCrossUp),
            "sma_cross_down" => Some(StrategyConditionType::SmaCrossDown),
            "stop_loss" => Some(StrategyConditionType::StopLoss),
            "take_profit" => Some(StrategyConditionType::TakeProfit),
            _ => None,
        }
    }
}

/// A trading strategy definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Strategy {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub entry_condition: StrategyConditionType,
    pub entry_threshold: f64,
    pub exit_condition: StrategyConditionType,
    pub exit_threshold: f64,
    pub stop_loss_percent: Option<f64>,
    pub take_profit_percent: Option<f64>,
    pub position_size_percent: f64, // % of capital per trade
    pub created_at: String,
}

/// Trade direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TradeDirection {
    Long,
    Short,
}

impl TradeDirection {
    pub fn as_str(&self) -> &'static str {
        match self {
            TradeDirection::Long => "long",
            TradeDirection::Short => "short",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "short" => TradeDirection::Short,
            _ => TradeDirection::Long,
        }
    }
}

/// A single trade from backtesting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestTrade {
    pub id: i64,
    pub backtest_id: i64,
    pub symbol: String,
    pub direction: TradeDirection,
    pub entry_date: NaiveDate,
    pub entry_price: f64,
    pub exit_date: Option<NaiveDate>,
    pub exit_price: Option<f64>,
    pub shares: f64,
    pub entry_reason: String,
    pub exit_reason: Option<String>,
    pub profit_loss: Option<f64>,
    pub profit_loss_percent: Option<f64>,
}

/// Performance metrics from backtesting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub total_return: f64,
    pub total_return_dollars: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: f64,
    pub win_rate: f64,
    pub total_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
    pub avg_win_percent: f64,
    pub avg_loss_percent: f64,
    pub profit_factor: f64,
    pub avg_trade_duration_days: f64,
}

/// Complete backtest result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResult {
    pub id: i64,
    pub strategy_id: i64,
    pub strategy_name: String,
    pub symbol: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub initial_capital: f64,
    pub final_capital: f64,
    pub metrics: PerformanceMetrics,
    pub trades: Vec<BacktestTrade>,
    pub created_at: String,
}

/// Yahoo Finance chart response structures
pub mod yahoo {
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    pub struct ChartResponse {
        pub chart: Chart,
    }

    #[derive(Debug, Deserialize)]
    pub struct Chart {
        pub result: Option<Vec<ChartResult>>,
        pub error: Option<ChartError>,
    }

    #[derive(Debug, Deserialize)]
    pub struct ChartError {
        pub code: String,
        pub description: String,
    }

    #[derive(Debug, Deserialize)]
    pub struct ChartResult {
        pub meta: ChartMeta,
        pub timestamp: Option<Vec<i64>>,
        pub indicators: Indicators,
    }

    #[derive(Debug, Deserialize)]
    pub struct ChartMeta {
        pub symbol: String,
        pub currency: Option<String>,
        #[serde(rename = "exchangeName")]
        pub exchange_name: Option<String>,
    }

    #[derive(Debug, Deserialize)]
    pub struct Indicators {
        pub quote: Vec<Quote>,
        pub adjclose: Option<Vec<AdjClose>>,
    }

    #[derive(Debug, Deserialize)]
    pub struct Quote {
        pub open: Vec<Option<f64>>,
        pub high: Vec<Option<f64>>,
        pub low: Vec<Option<f64>>,
        pub close: Vec<Option<f64>>,
        pub volume: Vec<Option<i64>>,
    }

    #[derive(Debug, Deserialize)]
    pub struct AdjClose {
        pub adjclose: Vec<Option<f64>>,
    }
}
