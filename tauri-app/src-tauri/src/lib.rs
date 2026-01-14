//! Tauri GUI backend for Financial Pipeline

use financial_pipeline::{calculate_all, AlertCondition, Database, Fred, PositionType, YahooFinance};
use serde::Serialize;
use std::sync::Mutex;
use tauri::State;

/// Application state holding the database connection
struct AppState {
    db: Mutex<Database>,
}

/// Symbol with latest price and percent change
#[derive(Serialize)]
struct SymbolPrice {
    symbol: String,
    price: f64,
    change_percent: f64,
    change_direction: String, // "up", "down", or "unchanged"
}

/// Command result
#[derive(Serialize)]
struct CommandResult {
    success: bool,
    message: String,
}

/// Indicator data for frontend
#[derive(Serialize)]
struct IndicatorData {
    name: String,
    value: f64,
    date: String,
}

/// Get all symbols with their latest prices and percent change
#[tauri::command]
fn get_symbols(state: State<AppState>) -> Result<Vec<SymbolPrice>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let symbols = db.get_symbols_with_data().map_err(|e| e.to_string())?;

    let mut result = Vec::new();
    for symbol in symbols {
        // Get price history to calculate percent change
        if let Ok(prices) = db.get_prices(&symbol) {
            if prices.len() >= 2 {
                let current = prices.last().unwrap();
                let previous = &prices[prices.len() - 2];

                let change_percent = if previous.close > 0.0 {
                    ((current.close - previous.close) / previous.close) * 100.0
                } else {
                    0.0
                };

                let change_direction = if change_percent > 0.001 {
                    "up".to_string()
                } else if change_percent < -0.001 {
                    "down".to_string()
                } else {
                    "unchanged".to_string()
                };

                result.push(SymbolPrice {
                    symbol,
                    price: current.close,
                    change_percent,
                    change_direction,
                });
            } else if let Some(price) = prices.last() {
                result.push(SymbolPrice {
                    symbol,
                    price: price.close,
                    change_percent: 0.0,
                    change_direction: "unchanged".to_string(),
                });
            }
        }
    }

    Ok(result)
}

/// Fetch stock prices from Yahoo Finance
#[tauri::command]
fn fetch_prices(
    state: State<AppState>,
    symbols: String,
    period: String,
) -> Result<CommandResult, String> {
    let mut db = state.db.lock().map_err(|e| e.to_string())?;

    let symbol_list: Vec<String> = symbols
        .split(',')
        .map(|s| s.trim().to_uppercase())
        .filter(|s| !s.is_empty())
        .collect();

    if symbol_list.is_empty() {
        return Ok(CommandResult {
            success: false,
            message: "No symbols provided".to_string(),
        });
    }

    let yahoo = YahooFinance::new();

    let mut success_count = 0;
    let mut fail_count = 0;

    for symbol in &symbol_list {
        match yahoo.fetch_and_store(&mut db, symbol, &period) {
            Ok(_) => success_count += 1,
            Err(_) => fail_count += 1,
        }
    }

    Ok(CommandResult {
        success: fail_count == 0,
        message: format!(
            "Fetched {} symbols ({} success, {} failed)",
            symbol_list.len(),
            success_count,
            fail_count
        ),
    })
}

/// Fetch FRED macro data
#[tauri::command]
fn fetch_fred(state: State<AppState>, indicators: String) -> Result<CommandResult, String> {
    let mut db = state.db.lock().map_err(|e| e.to_string())?;

    let indicator_list: Vec<&str> = indicators
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if indicator_list.is_empty() {
        return Ok(CommandResult {
            success: false,
            message: "No indicators provided".to_string(),
        });
    }

    let fred = Fred::new();

    let mut success_count = 0;
    let mut fail_count = 0;

    for indicator in &indicator_list {
        match fred.fetch_and_store(&mut db, indicator) {
            Ok(_) => success_count += 1,
            Err(_) => fail_count += 1,
        }
    }

    Ok(CommandResult {
        success: fail_count == 0,
        message: format!(
            "Fetched {} indicators ({} success, {} failed)",
            indicator_list.len(),
            success_count,
            fail_count
        ),
    })
}

/// Get price for a single symbol
#[tauri::command]
fn get_price(state: State<AppState>, symbol: String) -> Result<Option<f64>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_latest_price(&symbol.to_uppercase())
        .map_err(|e| e.to_string())
}

/// Calculate indicators for a symbol
#[tauri::command]
fn calculate_indicators(state: State<AppState>, symbol: String) -> Result<CommandResult, String> {
    let mut db = state.db.lock().map_err(|e| e.to_string())?;
    let symbol = symbol.to_uppercase();

    // Get price history
    let prices = db.get_prices(&symbol).map_err(|e| e.to_string())?;

    if prices.is_empty() {
        return Ok(CommandResult {
            success: false,
            message: format!("No price data for {}", symbol),
        });
    }

    // Calculate all indicators
    let indicators = calculate_all(&prices);
    let count = indicators.len();

    // Store them
    db.upsert_indicators(&indicators)
        .map_err(|e| e.to_string())?;

    println!("[OK] Calculated {} indicator values for {}", count, symbol);

    Ok(CommandResult {
        success: true,
        message: format!("Calculated {} indicator values for {}", count, symbol),
    })
}

/// Get latest indicators for a symbol
#[tauri::command]
fn get_indicators(state: State<AppState>, symbol: String) -> Result<Vec<IndicatorData>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let symbol = symbol.to_uppercase();

    let indicators = db
        .get_latest_indicators(&symbol)
        .map_err(|e| e.to_string())?;

    Ok(indicators
        .into_iter()
        .map(|i| IndicatorData {
            name: i.indicator_name,
            value: i.value,
            date: i.date.to_string(),
        })
        .collect())
}

/// Get indicator history for charting
#[tauri::command]
fn get_indicator_history(
    state: State<AppState>,
    symbol: String,
    indicator_name: String,
) -> Result<Vec<IndicatorData>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let symbol = symbol.to_uppercase();

    let indicators = db
        .get_indicator_history(&symbol, &indicator_name)
        .map_err(|e| e.to_string())?;

    Ok(indicators
        .into_iter()
        .map(|i| IndicatorData {
            name: i.indicator_name,
            value: i.value,
            date: i.date.to_string(),
        })
        .collect())
}

/// Price point for charting
#[derive(Serialize)]
struct PricePoint {
    date: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: i64,
}

/// Get price history for charting
#[tauri::command]
fn get_price_history(state: State<AppState>, symbol: String) -> Result<Vec<PricePoint>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let symbol = symbol.to_uppercase();

    let prices = db.get_prices(&symbol).map_err(|e| e.to_string())?;

    Ok(prices
        .into_iter()
        .map(|p| PricePoint {
            date: p.date.to_string(),
            open: p.open,
            high: p.high,
            low: p.low,
            close: p.close,
            volume: p.volume,
        })
        .collect())
}

/// Export data to CSV
#[tauri::command]
fn export_csv(state: State<AppState>, symbol: String) -> Result<CommandResult, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let symbol = symbol.to_uppercase();

    // Get price data
    let prices = db.get_prices(&symbol).map_err(|e| e.to_string())?;
    if prices.is_empty() {
        return Ok(CommandResult {
            success: false,
            message: format!("No data for {}", symbol),
        });
    }

    // Get indicators
    let indicators = db.get_latest_indicators(&symbol).map_err(|e| e.to_string())?;

    // Create export directory
    std::fs::create_dir_all("exports").ok();

    // Export prices
    let price_file = format!("exports/{}_prices.csv", symbol);
    let mut wtr = std::fs::File::create(&price_file).map_err(|e| e.to_string())?;
    use std::io::Write;
    writeln!(wtr, "date,open,high,low,close,volume").map_err(|e| e.to_string())?;
    for p in &prices {
        writeln!(wtr, "{},{},{},{},{},{}", p.date, p.open, p.high, p.low, p.close, p.volume)
            .map_err(|e| e.to_string())?;
    }

    // Export indicators
    let ind_file = format!("exports/{}_indicators.csv", symbol);
    let mut wtr = std::fs::File::create(&ind_file).map_err(|e| e.to_string())?;
    writeln!(wtr, "indicator,value,date").map_err(|e| e.to_string())?;
    for i in &indicators {
        writeln!(wtr, "{},{},{}", i.indicator_name, i.value, i.date).map_err(|e| e.to_string())?;
    }

    println!("[OK] Exported {} to CSV", symbol);

    Ok(CommandResult {
        success: true,
        message: format!("Exported to exports/{}_prices.csv and exports/{}_indicators.csv", symbol, symbol),
    })
}

/// Company name to symbol mapping for fuzzy search
fn get_symbol_mapping() -> std::collections::HashMap<&'static str, &'static str> {
    let mut map = std::collections::HashMap::new();
    // Tech
    map.insert("apple", "AAPL");
    map.insert("microsoft", "MSFT");
    map.insert("google", "GOOGL");
    map.insert("alphabet", "GOOGL");
    map.insert("amazon", "AMZN");
    map.insert("meta", "META");
    map.insert("facebook", "META");
    map.insert("nvidia", "NVDA");
    map.insert("tesla", "TSLA");
    map.insert("netflix", "NFLX");
    map.insert("intel", "INTC");
    map.insert("amd", "AMD");
    map.insert("cisco", "CSCO");
    map.insert("oracle", "ORCL");
    map.insert("ibm", "IBM");
    map.insert("salesforce", "CRM");
    map.insert("adobe", "ADBE");
    map.insert("paypal", "PYPL");
    map.insert("uber", "UBER");
    map.insert("airbnb", "ABNB");
    map.insert("spotify", "SPOT");
    map.insert("snap", "SNAP");
    map.insert("snapchat", "SNAP");
    map.insert("twitter", "X");
    map.insert("palantir", "PLTR");
    // Finance
    map.insert("jpmorgan", "JPM");
    map.insert("jp morgan", "JPM");
    map.insert("goldman", "GS");
    map.insert("goldman sachs", "GS");
    map.insert("morgan stanley", "MS");
    map.insert("bank of america", "BAC");
    map.insert("wells fargo", "WFC");
    map.insert("visa", "V");
    map.insert("mastercard", "MA");
    map.insert("berkshire", "BRK.B");
    // Retail/Consumer
    map.insert("walmart", "WMT");
    map.insert("costco", "COST");
    map.insert("target", "TGT");
    map.insert("home depot", "HD");
    map.insert("lowes", "LOW");
    map.insert("nike", "NKE");
    map.insert("starbucks", "SBUX");
    map.insert("mcdonalds", "MCD");
    map.insert("coca cola", "KO");
    map.insert("coke", "KO");
    map.insert("pepsi", "PEP");
    map.insert("disney", "DIS");
    // Healthcare
    map.insert("johnson", "JNJ");
    map.insert("pfizer", "PFE");
    map.insert("moderna", "MRNA");
    map.insert("unitedhealth", "UNH");
    // Energy
    map.insert("exxon", "XOM");
    map.insert("chevron", "CVX");
    // ETFs
    map.insert("s&p", "SPY");
    map.insert("s&p 500", "SPY");
    map.insert("spy", "SPY");
    map.insert("nasdaq", "QQQ");
    map.insert("qqq", "QQQ");
    map.insert("dow", "DIA");
    map.insert("dow jones", "DIA");
    map
}

/// Search for symbol by name (fuzzy match)
#[tauri::command]
fn search_symbol(query: String) -> Result<Vec<String>, String> {
    let query = query.to_lowercase();
    let mapping = get_symbol_mapping();

    let mut results = Vec::new();

    // Direct match first
    if let Some(symbol) = mapping.get(query.as_str()) {
        results.push(symbol.to_string());
    }

    // Partial match
    for (name, symbol) in &mapping {
        if name.contains(&query) || query.contains(name) {
            if !results.contains(&symbol.to_string()) {
                results.push(symbol.to_string());
            }
        }
    }

    // If query looks like a symbol, add it directly
    if query.len() <= 5 && query.chars().all(|c| c.is_alphabetic()) {
        let upper = query.to_uppercase();
        if !results.contains(&upper) {
            results.push(upper);
        }
    }

    Ok(results)
}

/// Alert data for frontend
#[derive(Serialize)]
struct AlertData {
    id: i64,
    symbol: String,
    target_price: f64,
    condition: String,
    triggered: bool,
    created_at: String,
}

/// Add a price alert
#[tauri::command]
fn add_alert(
    state: State<AppState>,
    symbol: String,
    target_price: f64,
    condition: String,
) -> Result<CommandResult, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let symbol = symbol.to_uppercase();

    let alert_condition = match condition.to_lowercase().as_str() {
        "above" => AlertCondition::Above,
        "below" => AlertCondition::Below,
        _ => return Err("Invalid condition. Use 'above' or 'below'".to_string()),
    };

    db.add_alert(&symbol, target_price, alert_condition)
        .map_err(|e| e.to_string())?;

    println!("[OK] Added alert for {} {} ${:.2}", symbol, condition, target_price);

    Ok(CommandResult {
        success: true,
        message: format!("Alert set: {} {} ${:.2}", symbol, condition, target_price),
    })
}

/// Get all alerts
#[tauri::command]
fn get_alerts(state: State<AppState>, only_active: bool) -> Result<Vec<AlertData>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let alerts = db.get_alerts(only_active).map_err(|e| e.to_string())?;

    Ok(alerts
        .into_iter()
        .map(|a| AlertData {
            id: a.id,
            symbol: a.symbol,
            target_price: a.target_price,
            condition: match a.condition {
                AlertCondition::Above => "above".to_string(),
                AlertCondition::Below => "below".to_string(),
            },
            triggered: a.triggered,
            created_at: a.created_at,
        })
        .collect())
}

/// Delete an alert
#[tauri::command]
fn delete_alert(state: State<AppState>, alert_id: i64) -> Result<CommandResult, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    db.delete_alert(alert_id).map_err(|e| e.to_string())?;

    Ok(CommandResult {
        success: true,
        message: "Alert deleted".to_string(),
    })
}

/// Check alerts against current prices
#[tauri::command]
fn check_alerts(state: State<AppState>) -> Result<Vec<AlertData>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let triggered = db.check_alerts().map_err(|e| e.to_string())?;

    Ok(triggered
        .into_iter()
        .map(|a| AlertData {
            id: a.id,
            symbol: a.symbol,
            target_price: a.target_price,
            condition: match a.condition {
                AlertCondition::Above => "above".to_string(),
                AlertCondition::Below => "below".to_string(),
            },
            triggered: a.triggered,
            created_at: a.created_at,
        })
        .collect())
}

/// Position data for frontend
#[derive(Serialize)]
struct PositionData {
    id: i64,
    symbol: String,
    quantity: f64,
    price: f64,
    position_type: String,
    date: String,
    notes: Option<String>,
    current_price: f64,
    current_value: f64,
    cost_basis: f64,
    profit_loss: f64,
    profit_loss_percent: f64,
}

/// Portfolio summary for frontend
#[derive(Serialize)]
struct PortfolioSummary {
    positions: Vec<PositionData>,
    total_value: f64,
    total_cost: f64,
    total_profit_loss: f64,
    total_profit_loss_percent: f64,
}

/// Add a portfolio position
#[tauri::command]
fn add_position(
    state: State<AppState>,
    symbol: String,
    quantity: f64,
    price: f64,
    position_type: String,
    date: String,
    notes: Option<String>,
) -> Result<CommandResult, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let symbol = symbol.to_uppercase();

    let pos_type = match position_type.to_lowercase().as_str() {
        "buy" => PositionType::Buy,
        "sell" => PositionType::Sell,
        _ => return Err("Invalid position type. Use 'buy' or 'sell'".to_string()),
    };

    db.add_position(&symbol, quantity, price, pos_type, &date, notes.as_deref())
        .map_err(|e| e.to_string())?;

    println!(
        "[OK] Added {} position: {} x {} @ ${:.2}",
        position_type, quantity, symbol, price
    );

    Ok(CommandResult {
        success: true,
        message: format!(
            "Added {} {} shares of {} @ ${:.2}",
            position_type, quantity, symbol, price
        ),
    })
}

/// Get portfolio with current values and P&L
#[tauri::command]
fn get_portfolio(state: State<AppState>) -> Result<PortfolioSummary, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let positions = db.get_positions().map_err(|e| e.to_string())?;

    let mut position_data = Vec::new();
    let mut total_value = 0.0;
    let mut total_cost = 0.0;

    for pos in positions {
        let current_price = db
            .get_latest_price(&pos.symbol)
            .map_err(|e| e.to_string())?
            .unwrap_or(pos.price);

        let cost_basis = pos.quantity * pos.price;
        let current_value = pos.quantity * current_price;

        // For sell positions, P&L is inverted (profit when price drops)
        let (profit_loss, profit_loss_percent) = match pos.position_type {
            PositionType::Buy => {
                let pl = current_value - cost_basis;
                let pl_pct = if cost_basis > 0.0 {
                    (pl / cost_basis) * 100.0
                } else {
                    0.0
                };
                total_value += current_value;
                total_cost += cost_basis;
                (pl, pl_pct)
            }
            PositionType::Sell => {
                // Short position: profit when price goes down
                let pl = cost_basis - current_value;
                let pl_pct = if cost_basis > 0.0 {
                    (pl / cost_basis) * 100.0
                } else {
                    0.0
                };
                // For shorts, we track the liability
                total_value -= current_value;
                total_cost -= cost_basis;
                (pl, pl_pct)
            }
        };

        position_data.push(PositionData {
            id: pos.id,
            symbol: pos.symbol,
            quantity: pos.quantity,
            price: pos.price,
            position_type: match pos.position_type {
                PositionType::Buy => "buy".to_string(),
                PositionType::Sell => "sell".to_string(),
            },
            date: pos.date,
            notes: pos.notes,
            current_price,
            current_value,
            cost_basis,
            profit_loss,
            profit_loss_percent,
        });
    }

    let total_profit_loss = total_value - total_cost;
    let total_profit_loss_percent = if total_cost.abs() > 0.0 {
        (total_profit_loss / total_cost.abs()) * 100.0
    } else {
        0.0
    };

    Ok(PortfolioSummary {
        positions: position_data,
        total_value,
        total_cost,
        total_profit_loss,
        total_profit_loss_percent,
    })
}

/// Delete a portfolio position
#[tauri::command]
fn delete_position(state: State<AppState>, position_id: i64) -> Result<CommandResult, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    db.delete_position(position_id).map_err(|e| e.to_string())?;

    Ok(CommandResult {
        success: true,
        message: "Position deleted".to_string(),
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize database
    let db = Database::open("data/finance.db").expect("Failed to open database");
    db.init_schema().expect("Failed to initialize schema");

    tauri::Builder::default()
        .manage(AppState { db: Mutex::new(db) })
        .invoke_handler(tauri::generate_handler![
            get_symbols,
            fetch_prices,
            fetch_fred,
            get_price,
            calculate_indicators,
            get_indicators,
            get_indicator_history,
            get_price_history,
            export_csv,
            search_symbol,
            add_alert,
            get_alerts,
            delete_alert,
            check_alerts,
            add_position,
            get_portfolio,
            delete_position,
        ])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
