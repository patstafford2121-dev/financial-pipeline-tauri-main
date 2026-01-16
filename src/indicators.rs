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

/// Calculate OBV (On-Balance Volume)
/// Cumulative volume indicator that adds volume on up days, subtracts on down days
pub fn calculate_obv(prices: &[DailyPrice]) -> Vec<TechnicalIndicator> {
    if prices.len() < 2 {
        return vec![];
    }

    let mut indicators = Vec::new();
    let mut obv: i64 = 0;

    // First day - just use volume as starting point
    obv = prices[0].volume;
    indicators.push(TechnicalIndicator {
        symbol: prices[0].symbol.clone(),
        date: prices[0].date,
        indicator_name: "OBV".to_string(),
        value: obv as f64,
    });

    // Calculate OBV for subsequent days
    for i in 1..prices.len() {
        if prices[i].close > prices[i - 1].close {
            obv += prices[i].volume; // Up day - add volume
        } else if prices[i].close < prices[i - 1].close {
            obv -= prices[i].volume; // Down day - subtract volume
        }
        // If close == prev close, OBV stays the same

        indicators.push(TechnicalIndicator {
            symbol: prices[0].symbol.clone(),
            date: prices[i].date,
            indicator_name: "OBV".to_string(),
            value: obv as f64,
        });
    }

    indicators
}

/// Calculate ADX (Average Directional Index)
/// Measures trend strength (not direction)
/// ADX > 25 = strong trend, ADX < 20 = weak/no trend
pub fn calculate_adx(prices: &[DailyPrice], period: usize) -> Vec<TechnicalIndicator> {
    if prices.len() < period * 2 + 1 {
        return vec![];
    }

    let mut indicators = Vec::new();

    // Calculate +DM, -DM, and TR for each day
    let mut plus_dm = Vec::new();
    let mut minus_dm = Vec::new();
    let mut tr = Vec::new();

    for i in 1..prices.len() {
        let high = prices[i].high;
        let low = prices[i].low;
        let prev_high = prices[i - 1].high;
        let prev_low = prices[i - 1].low;
        let prev_close = prices[i - 1].close;

        // Directional Movement
        let up_move = high - prev_high;
        let down_move = prev_low - low;

        let pdm = if up_move > down_move && up_move > 0.0 {
            up_move
        } else {
            0.0
        };
        let mdm = if down_move > up_move && down_move > 0.0 {
            down_move
        } else {
            0.0
        };

        plus_dm.push(pdm);
        minus_dm.push(mdm);

        // True Range
        let tr_val = (high - low)
            .max((high - prev_close).abs())
            .max((low - prev_close).abs());
        tr.push(tr_val);
    }

    // Smooth using Wilder's method
    let mut smooth_plus_dm: f64 = plus_dm[..period].iter().sum();
    let mut smooth_minus_dm: f64 = minus_dm[..period].iter().sum();
    let mut smooth_tr: f64 = tr[..period].iter().sum();

    let mut dx_values = Vec::new();

    for i in period..plus_dm.len() {
        // Wilder's smoothing
        smooth_plus_dm = smooth_plus_dm - (smooth_plus_dm / period as f64) + plus_dm[i];
        smooth_minus_dm = smooth_minus_dm - (smooth_minus_dm / period as f64) + minus_dm[i];
        smooth_tr = smooth_tr - (smooth_tr / period as f64) + tr[i];

        // Calculate +DI and -DI
        let plus_di = if smooth_tr != 0.0 {
            100.0 * smooth_plus_dm / smooth_tr
        } else {
            0.0
        };
        let minus_di = if smooth_tr != 0.0 {
            100.0 * smooth_minus_dm / smooth_tr
        } else {
            0.0
        };

        // Calculate DX
        let di_sum = plus_di + minus_di;
        let dx = if di_sum != 0.0 {
            100.0 * (plus_di - minus_di).abs() / di_sum
        } else {
            0.0
        };

        dx_values.push((prices[i + 1].date, dx, plus_di, minus_di));
    }

    // Calculate ADX (smoothed DX)
    if dx_values.len() >= period {
        let mut adx: f64 = dx_values[..period].iter().map(|d| d.1).sum::<f64>() / period as f64;

        for i in (period - 1)..dx_values.len() {
            if i >= period {
                adx = (adx * (period - 1) as f64 + dx_values[i].1) / period as f64;
            }

            indicators.push(TechnicalIndicator {
                symbol: prices[0].symbol.clone(),
                date: dx_values[i].0,
                indicator_name: format!("ADX_{}", period),
                value: adx,
            });

            indicators.push(TechnicalIndicator {
                symbol: prices[0].symbol.clone(),
                date: dx_values[i].0,
                indicator_name: format!("+DI_{}", period),
                value: dx_values[i].2,
            });

            indicators.push(TechnicalIndicator {
                symbol: prices[0].symbol.clone(),
                date: dx_values[i].0,
                indicator_name: format!("-DI_{}", period),
                value: dx_values[i].3,
            });
        }
    }

    indicators
}

/// Calculate Williams %R
/// Momentum indicator ranging from 0 to -100
/// Similar to Stochastic but inverted scale
/// Default period is 14
pub fn calculate_williams_r(prices: &[DailyPrice], period: usize) -> Vec<TechnicalIndicator> {
    if prices.len() < period {
        return vec![];
    }

    let mut indicators = Vec::new();

    for i in (period - 1)..prices.len() {
        let window = &prices[(i + 1 - period)..=i];

        let highest_high = window
            .iter()
            .map(|p| p.high)
            .fold(f64::NEG_INFINITY, f64::max);
        let lowest_low = window
            .iter()
            .map(|p| p.low)
            .fold(f64::INFINITY, f64::min);

        let close = prices[i].close;
        let range = highest_high - lowest_low;

        let wr = if range == 0.0 {
            -50.0 // Neutral if no range
        } else {
            ((highest_high - close) / range) * -100.0
        };

        indicators.push(TechnicalIndicator {
            symbol: prices[0].symbol.clone(),
            date: prices[i].date,
            indicator_name: format!("WILLR_{}", period),
            value: wr,
        });
    }

    indicators
}

/// Calculate CCI (Commodity Channel Index)
/// Measures price deviation from statistical mean
/// CCI > 100 = overbought, CCI < -100 = oversold
/// Default period is 20
pub fn calculate_cci(prices: &[DailyPrice], period: usize) -> Vec<TechnicalIndicator> {
    if prices.len() < period {
        return vec![];
    }

    let mut indicators = Vec::new();

    for i in (period - 1)..prices.len() {
        let window = &prices[(i + 1 - period)..=i];

        // Calculate Typical Prices for window
        let typical_prices: Vec<f64> = window
            .iter()
            .map(|p| (p.high + p.low + p.close) / 3.0)
            .collect();

        // SMA of Typical Price
        let tp_sma: f64 = typical_prices.iter().sum::<f64>() / period as f64;

        // Mean Deviation
        let mean_dev: f64 = typical_prices
            .iter()
            .map(|tp| (tp - tp_sma).abs())
            .sum::<f64>()
            / period as f64;

        // CCI = (TP - SMA(TP)) / (0.015 * Mean Deviation)
        let current_tp = typical_prices[period - 1];
        let cci = if mean_dev == 0.0 {
            0.0
        } else {
            (current_tp - tp_sma) / (0.015 * mean_dev)
        };

        indicators.push(TechnicalIndicator {
            symbol: prices[0].symbol.clone(),
            date: prices[i].date,
            indicator_name: format!("CCI_{}", period),
            value: cci,
        });
    }

    indicators
}

/// Calculate MFI (Money Flow Index)
/// Volume-weighted RSI, measures buying/selling pressure
/// MFI > 80 = overbought, MFI < 20 = oversold
/// Default period is 14
pub fn calculate_mfi(prices: &[DailyPrice], period: usize) -> Vec<TechnicalIndicator> {
    if prices.len() < period + 1 {
        return vec![];
    }

    let mut indicators = Vec::new();

    // Calculate Typical Price and Raw Money Flow for each day
    let typical_prices: Vec<f64> = prices
        .iter()
        .map(|p| (p.high + p.low + p.close) / 3.0)
        .collect();

    let raw_money_flows: Vec<f64> = prices
        .iter()
        .zip(typical_prices.iter())
        .map(|(p, tp)| tp * p.volume as f64)
        .collect();

    // Calculate MFI for each valid period
    for i in period..prices.len() {
        let mut positive_mf = 0.0;
        let mut negative_mf = 0.0;

        for j in (i + 1 - period)..=i {
            if j > 0 {
                if typical_prices[j] > typical_prices[j - 1] {
                    positive_mf += raw_money_flows[j];
                } else if typical_prices[j] < typical_prices[j - 1] {
                    negative_mf += raw_money_flows[j];
                }
            }
        }

        let mfi = if negative_mf == 0.0 {
            100.0
        } else if positive_mf == 0.0 {
            0.0
        } else {
            let mfr = positive_mf / negative_mf;
            100.0 - (100.0 / (1.0 + mfr))
        };

        indicators.push(TechnicalIndicator {
            symbol: prices[0].symbol.clone(),
            date: prices[i].date,
            indicator_name: format!("MFI_{}", period),
            value: mfi,
        });
    }

    indicators
}

/// Calculate ROC (Rate of Change)
/// Momentum oscillator measuring percentage change over N periods
/// Default period is 12
pub fn calculate_roc(prices: &[DailyPrice], period: usize) -> Vec<TechnicalIndicator> {
    if prices.len() <= period {
        return vec![];
    }

    let mut indicators = Vec::new();

    for i in period..prices.len() {
        let current_close = prices[i].close;
        let past_close = prices[i - period].close;

        let roc = if past_close == 0.0 {
            0.0
        } else {
            ((current_close - past_close) / past_close) * 100.0
        };

        indicators.push(TechnicalIndicator {
            symbol: prices[0].symbol.clone(),
            date: prices[i].date,
            indicator_name: format!("ROC_{}", period),
            value: roc,
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

    // OBV
    all.extend(calculate_obv(prices));

    // ADX 14
    all.extend(calculate_adx(prices, 14));

    // Williams %R 14
    all.extend(calculate_williams_r(prices, 14));

    // CCI 20
    all.extend(calculate_cci(prices, 20));

    // MFI 14
    all.extend(calculate_mfi(prices, 14));

    // ROC 12
    all.extend(calculate_roc(prices, 12));

    all
}
