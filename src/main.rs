//! Financial Pipeline CLI
//!
//! Command-line interface for the financial data pipeline.

use clap::{Parser, Subcommand};
use financial_pipeline::{Database, Fred, YahooFinance};

/// Financial Data Pipeline CLI
#[derive(Parser)]
#[command(name = "financial_pipeline")]
#[command(author = "STRYK")]
#[command(version = "0.1.0")]
#[command(about = "Financial data pipeline with SQLite storage", long_about = None)]
struct Cli {
    /// Database path
    #[arg(short, long, default_value = "data/finance.db")]
    database: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize database schema
    Init,

    /// Fetch stock prices from Yahoo Finance
    Fetch {
        /// Stock symbols (comma-separated)
        #[arg(short, long)]
        symbols: String,

        /// Time period (1d, 5d, 1mo, 3mo, 6mo, 1y, 2y, 5y, 10y, ytd, max)
        #[arg(short, long, default_value = "1y")]
        period: String,
    },

    /// Fetch macro data from FRED
    Macro {
        /// FRED indicator(s) (comma-separated, e.g., DFF,UNRATE,GDP)
        #[arg(short, long)]
        indicators: String,
    },

    /// Show latest price for a symbol
    Price {
        /// Stock symbol
        symbol: String,
    },

    /// List all symbols with data
    List,

    /// Create a watchlist
    Watchlist {
        /// Watchlist name
        #[arg(short, long)]
        name: String,

        /// Symbols (comma-separated)
        #[arg(short, long)]
        symbols: String,

        /// Description
        #[arg(short, long)]
        description: Option<String>,
    },

    /// Show common FRED indicators
    Indicators,

    /// Optimize database (vacuum)
    Vacuum,

    /// Refetch all existing symbols
    Refetch {
        /// Time period
        #[arg(short, long, default_value = "1y")]
        period: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Open database
    let mut db = Database::open(&cli.database)?;

    match cli.command {
        Commands::Init => {
            println!("{}", "=".repeat(60));
            println!("Financial Data Pipeline - Database Initialization");
            println!("{}", "=".repeat(60));
            db.init_schema()?;
            println!("\nDatabase initialized at: {}", cli.database);
        }

        Commands::Fetch { symbols, period } => {
            let symbol_list: Vec<String> =
                symbols.split(',').map(|s| s.trim().to_uppercase()).collect();

            let yahoo = YahooFinance::new();

            if symbol_list.len() == 1 {
                yahoo.fetch_and_store(&mut db, &symbol_list[0], &period)?;
            } else {
                yahoo.fetch_batch(&mut db, &symbol_list, &period)?;
            }
        }

        Commands::Macro { indicators } => {
            let indicator_list: Vec<&str> = indicators.split(',').map(|s| s.trim()).collect();

            let fred_client = Fred::new();
            fred_client.fetch_batch(&mut db, &indicator_list)?;
        }

        Commands::Price { symbol } => {
            let symbol = symbol.to_uppercase();
            match db.get_latest_price(&symbol)? {
                Some(price) => println!("{}: ${:.2}", symbol, price),
                None => println!("No data for {}", symbol),
            }
        }

        Commands::List => {
            let symbols = db.get_symbols_with_data()?;
            if symbols.is_empty() {
                println!("No symbols with price data");
            } else {
                println!("Symbols with price data ({}):", symbols.len());
                for symbol in symbols {
                    if let Some(price) = db.get_latest_price(&symbol)? {
                        println!("  {} - ${:.2}", symbol, price);
                    } else {
                        println!("  {}", symbol);
                    }
                }
            }
        }

        Commands::Watchlist {
            name,
            symbols,
            description,
        } => {
            let symbol_list: Vec<String> =
                symbols.split(',').map(|s| s.trim().to_uppercase()).collect();

            let id = db.create_watchlist(&name, &symbol_list, description.as_deref())?;
            println!(
                "Created watchlist '{}' (id: {}) with {} symbols",
                name,
                id,
                symbol_list.len()
            );
        }

        Commands::Indicators => {
            println!("Common FRED Indicators:");
            println!("  DFF      - Federal Funds Effective Rate (daily)");
            println!("  UNRATE   - Unemployment Rate (monthly)");
            println!("  GDP      - Real GDP (quarterly)");
            println!("  CPIAUCSL - Consumer Price Index (monthly)");
            println!("  DGS10    - 10-Year Treasury Yield (daily)");
            println!("  DGS2     - 2-Year Treasury Yield (daily)");
            println!("  SP500    - S&P 500 Index (daily)");
            println!("  VIXCLS   - VIX Volatility Index (daily)");
            println!("  PSAVERT  - Personal Savings Rate (monthly)");
            println!("  INDPRO   - Industrial Production Index (monthly)");
        }

        Commands::Vacuum => {
            db.vacuum()?;
        }

        Commands::Refetch { period } => {
            let symbols = db.get_symbols_with_data()?;
            if symbols.is_empty() {
                println!("No symbols to refetch");
                return Ok(());
            }

            println!("Refetching {} symbols...", symbols.len());

            for symbol in &symbols {
                db.clear_symbol_prices(symbol)?;
            }

            let yahoo = YahooFinance::new();
            yahoo.fetch_batch(&mut db, &symbols, &period)?;
        }
    }

    Ok(())
}
