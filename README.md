# Financial Data Pipeline - SQLite Edition

Local-first financial data system for market analysis, backtesting, and research. Built on SQLite with multi-API integration.

## Overview

**Philosophy:** Load once, query forever. Cache financial data locally to avoid API rate limits and enable offline analysis.

**Core Features:**
- 300k+ symbol catalog from FinanceDatabase
- Multi-source price data (Alpha Vantage, Finnhub, EOD)
- Macro economic indicators (FRED)
- Local SQLite storage with optimized schema
- Rate limit management
- Backtesting framework

## Quick Start

### Command Line Interface

```bash
# 1. Install dependencies
pip install -r requirements.txt

# 2. Configure API keys
cp config/config.example.json config/config.json
# Edit config.json with your API keys

# 3. Initialize database
python scripts/init_db.py

# 4. Load symbol catalog
python scripts/load_symbols.py

# 5. Fetch price data
python scripts/fetch_prices.py --symbols AAPL,MSFT,GOOGL

# 6. Query data
python scripts/query.py --sector Technology --limit 10
```

### Web GUI (Recommended)

```bash
# Launch the web interface
python launch_gui.py

# Or directly with streamlit
streamlit run app.py
```

**GUI Features:**
- 📊 Dashboard with real-time metrics
- 🔍 Symbol browser (search 300k+ symbols)
- 📥 Interactive data fetcher
- 💻 SQL query editor with examples
- ⭐ Watchlist management
- 📈 Price analysis and charting
- ⚙️ API rate limit monitoring

See `docs/GUI_GUIDE.md` for complete GUI documentation.

## Project Structure

```
FinancePipeline/
├── src/
│   ├── pipeline.py          # Main pipeline class
│   ├── sources/             # Data source adapters
│   │   ├── alpha_vantage.py
│   │   ├── finnhub.py
│   │   ├── fred.py
│   │   └── finance_db.py
│   ├── models.py            # Database models/schema
│   └── utils.py             # Helper functions
├── scripts/
│   ├── init_db.py           # Database initialization
│   ├── load_symbols.py      # Load symbol catalog
│   ├── fetch_prices.py      # Fetch price data
│   ├── fetch_macro.py       # Fetch FRED data
│   ├── backtest.py          # Backtesting runner
│   └── query.py             # Query examples
├── config/
│   ├── config.example.json  # Configuration template
│   └── schema.sql           # Database schema
├── data/
│   ├── finance.db           # SQLite database (created)
│   └── raw/                 # Downloaded source files
├── docs/
│   ├── API_SOURCES.md       # Data source documentation
│   ├── SCHEMA.md            # Database schema details
│   └── QUERIES.md           # Common query examples
├── tests/
│   └── test_pipeline.py     # Unit tests
├── requirements.txt         # Python dependencies
├── .env.example             # Environment variables template
└── README.md               # This file
```

## Data Sources

| Source | Free Tier | Best For |
|--------|-----------|----------|
| [FinanceDatabase](https://github.com/JerBouma/FinanceDatabase) | Unlimited | Symbol catalog |
| [Alpha Vantage](https://www.alphavantage.co/) | 25 calls/day | EOD prices |
| [Finnhub](https://finnhub.io/) | 60 calls/min | Real-time data |
| [FRED](https://fred.stlouisfed.org/) | Unlimited | Macro indicators |
| [Financial Modeling Prep](https://site.financialmodelingprep.com/) | 250 calls/day | Fundamentals |

## Usage Examples

### Load Symbol Catalog
```python
from src.pipeline import FinancePipeline

pipeline = FinancePipeline()
pipeline.load_symbols("data/raw/equities.json")
# Loaded 300,000+ symbols
```

### Fetch Price Data
```python
pipeline.configure(alpha_key="YOUR_KEY")
pipeline.fetch_prices_alpha("AAPL", outputsize="full")
# Loaded 20 years of daily prices
```

### Query Sector Performance
```python
result = pipeline.query("""
    SELECT s.sector, AVG(p.close) as avg_price, COUNT(*) as num_stocks
    FROM symbols s
    JOIN daily_prices p ON s.symbol = p.symbol
    WHERE p.timestamp >= date('now', '-30 days')
    GROUP BY s.sector
    ORDER BY avg_price DESC
""")
```

### Backtest Strategy
```python
from scripts.backtest import backtest_strategy

def simple_sma(data):
    # Your strategy logic
    return 'BUY' if data['sma_20'] > data['sma_50'] else 'HOLD'

results = backtest_strategy('2024-01-01', '2024-12-31', simple_sma)
```

## Rate Limit Management

**Alpha Vantage (25/day):**
- Load 24 symbols/day max
- ~1 hour delay between calls
- Cache results locally

**Finnhub (60/min):**
- Batch 60 symbols per minute
- Wait 60 seconds between batches

**Best Practice:** Load data overnight, query unlimited during day.

## Maintenance

**Daily Price Updates:**
```bash
# Add to cron: 0 18 * * 1-5
python scripts/fetch_prices.py --watchlist config/watchlist.txt
```

**Weekly Symbol Refresh:**
```bash
cd data/raw/FinanceDatabase
git pull origin main
python scripts/load_symbols.py --update
```

**Monthly Database Optimization:**
```bash
python scripts/optimize_db.py
```

## Configuration

Copy `config/config.example.json` to `config/config.json`:

```json
{
  "database": {
    "path": "data/finance.db"
  },
  "api_keys": {
    "alpha_vantage": "YOUR_KEY_HERE",
    "finnhub": "YOUR_KEY_HERE",
    "fmp": "YOUR_KEY_HERE"
  },
  "rate_limits": {
    "alpha_vantage_delay": 3600,
    "finnhub_batch_size": 60
  },
  "watchlist": [
    "AAPL", "MSFT", "GOOGL", "AMZN", "META"
  ]
}
```

## PhiSHRI Integration

This project implements **Door 865: Financial Data Pipeline - SQLite Edition**.

Related doors:
- **T01DATABASE** - SQLite operations
- **D05PYTHON_ENV** - Python environment setup
- **W101API_WORKFLOW** - API integration patterns

## Contributing

1. Fork the repo
2. Create feature branch
3. Add tests
4. Submit PR

## License

MIT License - see LICENSE file

## Support

- PhiSHRI Door: 865
- Issues: Use GitHub issues
- Docs: See `docs/` directory
