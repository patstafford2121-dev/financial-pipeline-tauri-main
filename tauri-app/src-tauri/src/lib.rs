//! Tauri GUI backend for Financial Pipeline

use financial_pipeline::{calculate_all, Database, Fred, YahooFinance};
use serde::Serialize;
use std::sync::Mutex;
use tauri::State;

/// Application state holding the database connection
struct AppState {
    db: Mutex<Database>,
}

/// Symbol with latest price
#[derive(Serialize)]
struct SymbolPrice {
    symbol: String,
    price: f64,
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

/// Get all symbols with their latest prices
#[tauri::command]
fn get_symbols(state: State<AppState>) -> Result<Vec<SymbolPrice>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let symbols = db.get_symbols_with_data().map_err(|e| e.to_string())?;

    let mut result = Vec::new();
    for symbol in symbols {
        if let Ok(Some(price)) = db.get_latest_price(&symbol) {
            result.push(SymbolPrice { symbol, price });
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
