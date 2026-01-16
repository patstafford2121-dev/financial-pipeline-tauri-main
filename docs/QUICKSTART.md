# Quick Start Guide

Get up and running with the Financial Data Pipeline in 10 minutes.

## Prerequisites

- Python 3.8+
- Git (for downloading FinanceDatabase)
- API keys (free tier works fine)

## Step 1: Install Dependencies

```bash
pip install -r requirements.txt
```

## Step 2: Configure API Keys

```bash
# Copy config template
cp config/config.example.json config/config.json

# Edit config.json and add your API keys
# Get free keys:
# - Alpha Vantage: https://www.alphavantage.co/support/#api-key
# - Finnhub: https://finnhub.io/register
# - FRED: No key needed!
```

## Step 3: Initialize Database

```bash
python scripts/init_db.py
```

Output:
```
Financial Data Pipeline - Database Initialization
==============================================================
1. Creating database schema...
2. Creating default watchlists...
3. Verifying setup...

Created 12 tables:
  - symbols
  - daily_prices
  - fundamentals
  - macro_data
  - watchlists
  ...

✓ Database initialized at: data/finance.db
```

## Step 4: Load Symbol Catalog

```bash
# Download FinanceDatabase and load equities
python scripts/load_symbols.py --download --asset-class equity
```

This will:
- Clone FinanceDatabase repo (~300k symbols)
- Load equity symbols into database
- Show breakdown by sector

Takes ~2-3 minutes.

## Step 5: Fetch Price Data

```bash
# Fetch prices for a few symbols (starts with default watchlist)
python scripts/fetch_prices.py --symbols AAPL,MSFT,GOOGL --outputsize full
```

Note: Alpha Vantage free tier = 25 calls/day, so start small!

## Step 6: Fetch Macro Data

```bash
# Load economic indicators from FRED (unlimited, no key needed)
python scripts/fetch_macro.py
```

Loads GDP, unemployment, interest rates, etc.

## Step 7: Query Your Data

```bash
# Show sector performance
python scripts/query.py --sector

# Show top movers
python scripts/query.py --movers --days 30

# Custom SQL
python scripts/query.py --sql "SELECT * FROM symbols WHERE sector='Technology' LIMIT 10"
```

## Using in Python

```python
from src.pipeline import FinancePipeline

# Initialize
pipeline = FinancePipeline()

# Query data
tech_stocks = pipeline.query("""
    SELECT s.symbol, s.name, p.close 
    FROM symbols s
    JOIN daily_prices p ON s.symbol = p.symbol
    WHERE s.sector = 'Technology' 
    AND p.timestamp = (SELECT MAX(timestamp) FROM daily_prices)
    LIMIT 10
""")

print(tech_stocks)
```

## Daily Workflow

Once set up, your daily routine:

```bash
# Morning: Update prices (runs in ~5 min for 25 symbols)
python scripts/fetch_prices.py --watchlist-name default

# Analyze
python scripts/query.py --sector
python scripts/query.py --movers --days 1

# Custom analysis in Python
python -c "from src.pipeline import FinancePipeline; p = FinancePipeline(); ..."
```

## Next Steps

- **Backtesting**: See `docs/BACKTESTING.md`
- **Advanced queries**: See `docs/QUERIES.md`  
- **API sources**: See `docs/API_SOURCES.md`
- **Automation**: Set up cron jobs for daily updates

## Troubleshooting

**"API rate limit exceeded":**
- Check usage: `SELECT COUNT(*) FROM api_calls WHERE source='alpha_vantage' AND DATE(timestamp)=DATE('now')`
- Wait until tomorrow or use different API

**"No data returned":**
- Verify API key in config.json
- Check symbol exists: `SELECT * FROM symbols WHERE symbol='AAPL'`
- Try different data source

**"Database locked":**
- Close other connections
- Run: `pipeline.vacuum()` to optimize

## Support

- PhiSHRI Door: 865
- Full docs: See `docs/` directory
- Config reference: `config/config.example.json`
