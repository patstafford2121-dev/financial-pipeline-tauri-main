//! Financial Pipeline - Native GUI (Slint)

use financial_pipeline::{Database, Fred, YahooFinance};
use slint::{ModelRc, SharedString, VecModel};
use std::rc::Rc;

slint::include_modules!();

fn main() {
    // Initialize database
    let db = Database::open("data/finance.db").expect("Failed to open database");
    db.init_schema().expect("Failed to initialize schema");

    // Create UI
    let ui = AppWindow::new().unwrap();

    // Wrap database in Rc for sharing between callbacks
    let db = Rc::new(std::cell::RefCell::new(db));

    // Initial refresh
    refresh_symbols(&ui, &db);

    // Set up callbacks
    let ui_weak = ui.as_weak();
    let db_clone = db.clone();
    ui.on_fetch_prices(move |symbols, period| {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_status_text(SharedString::from(format!(
                "Fetching {}...",
                symbols.as_str()
            )));

            let symbol_list: Vec<String> = symbols
                .as_str()
                .split(',')
                .map(|s| s.trim().to_uppercase())
                .filter(|s| !s.is_empty())
                .collect();

            let mut db = db_clone.borrow_mut();
            let yahoo = YahooFinance::new();

            let mut success = 0;
            let mut failed = 0;

            for symbol in &symbol_list {
                match yahoo.fetch_and_store(&mut db, symbol, period.as_str()) {
                    Ok(_) => success += 1,
                    Err(_) => failed += 1,
                }
            }

            drop(db);

            ui.set_status_text(SharedString::from(format!(
                "Fetched {} symbols ({} success, {} failed)",
                symbol_list.len(),
                success,
                failed
            )));

            refresh_symbols(&ui, &db_clone);
        }
    });

    let ui_weak = ui.as_weak();
    let db_clone = db.clone();
    ui.on_fetch_fred(move |indicators| {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_status_text(SharedString::from(format!(
                "Fetching FRED: {}...",
                indicators.as_str()
            )));

            let indicator_list: Vec<&str> = indicators
                .as_str()
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();

            let mut db = db_clone.borrow_mut();
            let fred = Fred::new();

            let mut success = 0;
            let mut failed = 0;

            for indicator in &indicator_list {
                match fred.fetch_and_store(&mut db, indicator) {
                    Ok(_) => success += 1,
                    Err(_) => failed += 1,
                }
            }

            ui.set_status_text(SharedString::from(format!(
                "Fetched {} indicators ({} success, {} failed)",
                indicator_list.len(),
                success,
                failed
            )));
        }
    });

    let ui_weak = ui.as_weak();
    let db_clone = db.clone();
    ui.on_refresh_list(move || {
        if let Some(ui) = ui_weak.upgrade() {
            refresh_symbols(&ui, &db_clone);
        }
    });

    // Run the UI
    ui.run().unwrap();
}

fn refresh_symbols(ui: &AppWindow, db: &Rc<std::cell::RefCell<Database>>) {
    let db = db.borrow();

    let symbols = db.get_symbols_with_data().unwrap_or_default();

    let stocks: Vec<StockPrice> = symbols
        .iter()
        .filter_map(|symbol| {
            db.get_latest_price(symbol).ok().flatten().map(|price| {
                StockPrice {
                    symbol: SharedString::from(symbol.as_str()),
                    price: price as f32,
                }
            })
        })
        .collect();

    ui.set_symbol_count(stocks.len() as i32);
    ui.set_symbols(ModelRc::new(VecModel::from(stocks)));
    ui.set_status_text(SharedString::from(format!(
        "Loaded {} symbols",
        ui.get_symbol_count()
    )));
}
