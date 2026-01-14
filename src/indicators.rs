//! Technical indicators calculator

use crate::models::{DailyPrice, TechnicalIndicator};

/// Calculate RSI (Relative Strength Index)
/// Period is typically 14
pub fn calculate_rsi(prices: &[DailyPrice], period: usize) -> Vec<TechnicalIndicator> {
    if prices.len() < period + 1 {
        return vec![];
    }

    let mut indicators = Vec::new();
    let mut gains = Vec::new();
    let mut losses = Vec::new();

    // Calculate price changes
    for i in 1..prices.len() {
        let change = prices[i].close - prices[i - 1].close;
        gains.push(if change > 0.0 { change } else { 0.0 });
        losses.push(if change < 0.0 { -change } else { 0.0 });
    }

    // Calculate initial average gain/loss
    let mut avg_gain: f64 = gains[..period].iter().sum::<f64>() / period as f64;
    let mut avg_loss: f64 = losses[..period].iter().sum::<f64>() / period as f64;

    // First RSI value
    let rs = if avg_loss == 0.0 {
        100.0
    } else {
        avg_gain / avg_loss
    };
    let rsi = 100.0 - (100.0 / (1.0 + rs));
    indicators.push(TechnicalIndicator {
        symbol: prices[0].symbol.clone(),
        date: prices[period].date,
        indicator_name: format!("RSI_{}", period),
        value: rsi,
    });

    // Calculate subsequent RSI values using smoothed averages
    for i in period..gains.len() {
        avg_gain = (avg_gain * (period - 1) as f64 + gains[i]) / period as f64;
        avg_loss = (avg_loss * (period - 1) as f64 + losses[i]) / period as f64;

        let rs = if avg_loss == 0.0 {
            100.0
        } else {
            avg_gain / avg_loss
        };
        let rsi = 100.0 - (100.0 / (1.0 + rs));

        indicators.push(TechnicalIndicator {
            symbol: prices[0].symbol.clone(),
            date: prices[i + 1].date,
            indicator_name: format!("RSI_{}", period),
            value: rsi,
        });
    }

    indicators
}

/// Calculate SMA (Simple Moving Average)
pub fn calculate_sma(prices: &[DailyPrice], period: usize) -> Vec<TechnicalIndicator> {
    if prices.len() < period {
        return vec![];
    }

    let mut indicators = Vec::new();

    for i in (period - 1)..prices.len() {
        let sum: f64 = prices[(i + 1 - period)..=i]
            .iter()
            .map(|p| p.close)
            .sum();
        let sma = sum / period as f64;

        indicators.push(TechnicalIndicator {
            symbol: prices[0].symbol.clone(),
            date: prices[i].date,
            indicator_name: format!("SMA_{}", period),
            value: sma,
        });
    }

    indicators
}

/// Calculate EMA (Exponential Moving Average)
pub fn calculate_ema(prices: &[DailyPrice], period: usize) -> Vec<TechnicalIndicator> {
    if prices.len() < period {
        return vec![];
    }

    let mut indicators = Vec::new();
    let multiplier = 2.0 / (period as f64 + 1.0);

    // First EMA is SMA
    let initial_sma: f64 = prices[..period].iter().map(|p| p.close).sum::<f64>() / period as f64;
    let mut ema = initial_sma;

    indicators.push(TechnicalIndicator {
        symbol: prices[0].symbol.clone(),
        date: prices[period - 1].date,
        indicator_name: format!("EMA_{}", period),
        value: ema,
    });

    // Calculate subsequent EMAs
    for i in period..prices.len() {
        ema = (prices[i].close - ema) * multiplier + ema;
        indicators.push(TechnicalIndicator {
            symbol: prices[0].symbol.clone(),
            date: prices[i].date,
            indicator_name: format!("EMA_{}", period),
            value: ema,
        });
    }

    indicators
}

/// Calculate MACD (Moving Average Convergence Divergence)
/// Returns MACD line, signal line, and histogram
pub fn calculate_macd(
    prices: &[DailyPrice],
    fast: usize,
    slow: usize,
    signal: usize,
) -> Vec<TechnicalIndicator> {
    if prices.len() < slow + signal {
        return vec![];
    }

    let mut indicators = Vec::new();
    let fast_mult = 2.0 / (fast as f64 + 1.0);
    let slow_mult = 2.0 / (slow as f64 + 1.0);
    let signal_mult = 2.0 / (signal as f64 + 1.0);

    // Calculate EMAs
    let fast_sma: f64 = prices[..fast].iter().map(|p| p.close).sum::<f64>() / fast as f64;
    let slow_sma: f64 = prices[..slow].iter().map(|p| p.close).sum::<f64>() / slow as f64;

    let mut fast_ema = fast_sma;
    let mut slow_ema = slow_sma;
    let mut macd_values = Vec::new();

    // Calculate MACD line (fast EMA - slow EMA)
    for i in slow..prices.len() {
        // Update EMAs
        if i >= fast {
            fast_ema = (prices[i].close - fast_ema) * fast_mult + fast_ema;
        }
        slow_ema = (prices[i].close - slow_ema) * slow_mult + slow_ema;

        let macd = fast_ema - slow_ema;
        macd_values.push((prices[i].date, macd));
    }

    // Calculate signal line (EMA of MACD)
    if macd_values.len() >= signal {
        let signal_sma: f64 = macd_values[..signal].iter().map(|m| m.1).sum::<f64>() / signal as f64;
        let mut signal_ema = signal_sma;

        for (idx, (date, macd)) in macd_values.iter().enumerate().skip(signal - 1) {
            if idx >= signal {
                signal_ema = (macd - signal_ema) * signal_mult + signal_ema;
            }

            let histogram = macd - signal_ema;

            indicators.push(TechnicalIndicator {
                symbol: prices[0].symbol.clone(),
                date: *date,
                indicator_name: format!("MACD_{}_{}", fast, slow),
                value: *macd,
            });

            indicators.push(TechnicalIndicator {
                symbol: prices[0].symbol.clone(),
                date: *date,
                indicator_name: format!("MACD_SIGNAL_{}", signal),
                value: signal_ema,
            });

            indicators.push(TechnicalIndicator {
                symbol: prices[0].symbol.clone(),
                date: *date,
                indicator_name: "MACD_HIST".to_string(),
                value: histogram,
            });
        }
    }

    indicators
}

/// Calculate Bollinger Bands
/// Returns upper band, middle band (SMA), and lower band
/// Default: 20-period SMA with 2 standard deviations
pub fn calculate_bollinger_bands(
    prices: &[DailyPrice],
    period: usize,
    std_dev_mult: f64,
) -> Vec<TechnicalIndicator> {
    if prices.len() < period {
        return vec![];
    }

    let mut indicators = Vec::new();

    for i in (period - 1)..prices.len() {
        let window = &prices[(i + 1 - period)..=i];

        // Calculate SMA (middle band)
        let sum: f64 = window.iter().map(|p| p.close).sum();
        let sma = sum / period as f64;

        // Calculate standard deviation
        let variance: f64 = window
            .iter()
            .map(|p| {
                let diff = p.close - sma;
                diff * diff
            })
            .sum::<f64>()
            / period as f64;
        let std_dev = variance.sqrt();

        // Calculate bands
        let upper = sma + (std_dev_mult * std_dev);
        let lower = sma - (std_dev_mult * std_dev);

        indicators.push(TechnicalIndicator {
            symbol: prices[0].symbol.clone(),
            date: prices[i].date,
            indicator_name: format!("BB_UPPER_{}", period),
            value: upper,
        });

        indicators.push(TechnicalIndicator {
            symbol: prices[0].symbol.clone(),
            date: prices[i].date,
            indicator_name: format!("BB_MIDDLE_{}", period),
            value: sma,
        });

        indicators.push(TechnicalIndicator {
            symbol: prices[0].symbol.clone(),
            date: prices[i].date,
            indicator_name: format!("BB_LOWER_{}", period),
            value: lower,
        });
    }

    indicators
}

/// Calculate ATR (Average True Range)
/// Measures volatility based on price range
/// Default period is 14
pub fn calculate_atr(prices: &[DailyPrice], period: usize) -> Vec<TechnicalIndicator> {
    if prices.len() < period + 1 {
        return vec![];
    }

    let mut indicators = Vec::new();
    let mut true_ranges = Vec::new();

    // Calculate True Range for each day (starting from day 1)
    for i in 1..prices.len() {
        let high = prices[i].high;
        let low = prices[i].low;
        let prev_close = prices[i - 1].close;

        let tr = (high - low)
            .max((high - prev_close).abs())
            .max((low - prev_close).abs());
        true_ranges.push(tr);
    }

    // First ATR is simple average of first 'period' true ranges
    let first_atr: f64 = true_ranges[..period].iter().sum::<f64>() / period as f64;
    let mut atr = first_atr;

    indicators.push(TechnicalIndicator {
        symbol: prices[0].symbol.clone(),
        date: prices[period].date,
        indicator_name: format!("ATR_{}", period),
        value: atr,
    });

    // Subsequent ATRs use smoothed average (Wilder's smoothing)
    for i in period..true_ranges.len() {
        atr = (atr * (period - 1) as f64 + true_ranges[i]) / period as f64;

        indicators.push(TechnicalIndicator {
            symbol: prices[0].symbol.clone(),
            date: prices[i + 1].date,
            indicator_name: format!("ATR_{}", period),
            value: atr,
        });
    }

    indicators
}

/// Calculate Stochastic Oscillator
/// %K = (Close - Lowest Low) / (Highest High - Lowest Low) * 100
/// %D = SMA of %K
/// Default: 14-period %K, 3-period %D
pub fn calculate_stochastic(
    prices: &[DailyPrice],
    k_period: usize,
    d_period: usize,
) -> Vec<TechnicalIndicator> {
    if prices.len() < k_period + d_period {
        return vec![];
    }

    let mut indicators = Vec::new();
    let mut k_values = Vec::new();

    // Calculate %K for each day
    for i in (k_period - 1)..prices.len() {
        let window = &prices[(i + 1 - k_period)..=i];

        let lowest_low = window
            .iter()
            .map(|p| p.low)
            .fold(f64::INFINITY, f64::min);
        let highest_high = window
            .iter()
            .map(|p| p.high)
            .fold(f64::NEG_INFINITY, f64::max);

        let close = prices[i].close;
        let range = highest_high - lowest_low;

        let k = if range == 0.0 {
            50.0 // Neutral if no range
        } else {
            ((close - lowest_low) / range) * 100.0
        };

        k_values.push((prices[i].date, k));

        indicators.push(TechnicalIndicator {
            symbol: prices[0].symbol.clone(),
            date: prices[i].date,
            indicator_name: format!("STOCH_K_{}", k_period),
            value: k,
        });
    }

    // Calculate %D (SMA of %K)
    for i in (d_period - 1)..k_values.len() {
        let d_sum: f64 = k_values[(i + 1 - d_period)..=i]
            .iter()
            .map(|(_, k)| k)
            .sum();
        let d = d_sum / d_period as f64;

        indicators.push(TechnicalIndicator {
            symbol: prices[0].symbol.clone(),
            date: k_values[i].0,
            indicator_name: format!("STOCH_D_{}", d_period),
            value: d,
        });
    }

    indicators
}

/// Calculate all standard indicators for a symbol
pub fn calculate_all(prices: &[DailyPrice]) -> Vec<TechnicalIndicator> {
    let mut all = Vec::new();

    // RSI 14
    all.extend(calculate_rsi(prices, 14));

    // SMA 20, 50
    all.extend(calculate_sma(prices, 20));
    all.extend(calculate_sma(prices, 50));

    // EMA 12, 26
    all.extend(calculate_ema(prices, 12));
    all.extend(calculate_ema(prices, 26));

    // MACD 12, 26, 9
    all.extend(calculate_macd(prices, 12, 26, 9));

    // Bollinger Bands 20, 2
    all.extend(calculate_bollinger_bands(prices, 20, 2.0));

    // ATR 14
    all.extend(calculate_atr(prices, 14));

    // Stochastic 14, 3
    all.extend(calculate_stochastic(prices, 14, 3));

    all
}
