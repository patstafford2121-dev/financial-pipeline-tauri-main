# Financial Data Pipeline Project - Complete

**PhiSHRI Door:** 865  
**Category:** TOOLS.DATABASE.FINANCE  
**Created:** January 2026  
**Status:** Production Ready ✓

---

## Project Structure

```
FinancePipeline/
├── README.md                    # Main documentation
├── setup.py                     # Automated setup script
├── requirements.txt             # Python dependencies
├── .env.example                 # Environment template
├── .gitignore                   # Git ignore rules
│
├── config/
│   ├── config.example.json      # Configuration template
│   └── schema.sql               # Database schema
│
├── src/
│   └── pipeline.py              # Main FinancePipeline class
│
├── scripts/
│   ├── init_db.py               # Initialize database
│   ├── load_symbols.py          # Load symbol catalog
│   ├── fetch_prices.py          # Fetch price data
│   ├── fetch_macro.py           # Fetch FRED macro data
│   └── query.py                 # Query examples
│
├── docs/
│   └── QUICKSTART.md            # Quick start guide
│
├── examples/
│   └── workflow.py              # Example analysis workflows
│
├── data/                        # Data directory (created)
│   ├── finance.db               # SQLite database (created)
│   ├── raw/                     # Downloaded source files
│   └── backups/                 # Database backups
│
├── logs/                        # Log files (created)
└── tests/                       # Unit tests (placeholder)
```

---

## Features Implemented

### Core Pipeline
- ✓ SQLite database with optimized schema
- ✓ Multi-source API integration (Alpha Vantage, Finnhub, FRED)
- ✓ Rate limit management and tracking
- ✓ Automatic retry and error handling
- ✓ Database backup and optimization

### Data Sources
- ✓ FinanceDatabase integration (300k+ symbols)
- ✓ Alpha Vantage (daily prices, fundamentals)
- ✓ FRED (macro economic indicators)
- ✓ Support for multiple asset classes (equity, ETF, crypto, forex)

### Query & Analysis
- ✓ Pre-built query templates
- ✓ Sector performance analysis
- ✓ Price momentum tracking
- ✓ Macro correlation analysis
- ✓ Custom watchlist management
- ✓ Flexible SQL interface

### Automation
- ✓ Bulk data loading
- ✓ Scheduled update support
- ✓ Progress tracking
- ✓ API call logging

---

## Installation & Setup

### One-Command Setup
```bash
python setup.py
```

This will:
1. Check Python version
2. Install dependencies
3. Create configuration files
4. Initialize database
5. Prompt for API keys (optional)

### Manual Setup
```bash
# Install dependencies
pip install -r requirements.txt

# Create config
cp config/config.example.json config/config.json
# Edit config.json with your API keys

# Initialize database
python scripts/init_db.py
```

---

## Usage Examples

### Load Symbol Catalog
```bash
# Download and load FinanceDatabase
python scripts/load_symbols.py --download --asset-class equity
```

### Fetch Price Data
```bash
# Fetch prices for specific symbols
python scripts/fetch_prices.py --symbols AAPL,MSFT,GOOGL

# Use a watchlist
python scripts/fetch_prices.py --watchlist-name default

# Full historical data
python scripts/fetch_prices.py --symbols AAPL --outputsize full
```

### Fetch Macro Data
```bash
# Load common indicators
python scripts/fetch_macro.py

# Specific indicators
python scripts/fetch_macro.py --indicators GDP,UNRATE,DFF

# List available indicators
python scripts/fetch_macro.py --list
```

### Query Data
```bash
# Sector performance
python scripts/query.py --sector

# Top price movers
python scripts/query.py --movers --days 30

# Watchlist summary
python scripts/query.py --watchlist default

# Custom SQL
python scripts/query.py --sql "SELECT * FROM symbols WHERE sector='Technology' LIMIT 10"
```

### Python API
```python
from src.pipeline import FinancePipeline

# Initialize
pipeline = FinancePipeline()

# Load data
pipeline.load_symbols("data/raw/equities.json")
pipeline.fetch_prices_alpha("AAPL", outputsize="full")
pipeline.fetch_fred("GDP")

# Query
tech_stocks = pipeline.query("""
    SELECT s.symbol, s.name, p.close 
    FROM symbols s
    JOIN daily_prices p ON s.symbol = p.symbol
    WHERE s.sector = 'Technology'
    LIMIT 10
""")

# Get latest price
price = pipeline.get_latest_price("AAPL")

# Watchlist management
pipeline.create_watchlist("my_portfolio", ["AAPL", "MSFT", "GOOGL"])
symbols = pipeline.get_watchlist("my_portfolio")
```

---

## Database Schema

### Core Tables
- **symbols** - Symbol catalog (name, sector, industry, market cap)
- **daily_prices** - Daily OHLCV data
- **fundamentals** - Financial statements and ratios
- **macro_data** - Economic indicators from FRED
- **watchlists** - User-defined symbol lists

### Support Tables
- **api_calls** - Rate limit tracking
- **backtest_results** - Strategy performance
- **news** - News and events

### Views
- **latest_prices** - Most recent price per symbol
- **symbol_summary** - Symbols with latest prices
- **api_rate_limits** - Current API usage

---

## Configuration

Edit `config/config.json`:

```json
{
  "database": {
    "path": "data/finance.db"
  },
  "api_keys": {
    "alpha_vantage": "YOUR_KEY",
    "finnhub": "YOUR_KEY",
    "fmp": "YOUR_KEY"
  },
  "rate_limits": {
    "alpha_vantage": {
      "calls_per_day": 25,
      "delay_seconds": 3600
    }
  },
  "watchlists": {
    "default": ["AAPL", "MSFT", "GOOGL"]
  }
}
```

---

## Rate Limits (Free Tier)

| Source | Free Tier | Best For |
|--------|-----------|----------|
| Alpha Vantage | 25/day | EOD prices |
| Finnhub | 60/min | Real-time |
| FRED | Unlimited | Macro data |
| FMP | 250/day | Fundamentals |

**Strategy:** Load data once, query unlimited times locally.

---

## Maintenance

### Daily Updates
```bash
# Update prices for watchlist (5 min for 25 symbols)
python scripts/fetch_prices.py --watchlist-name default
```

### Weekly Tasks
```bash
# Update symbol catalog
python scripts/load_symbols.py --download --all

# Backup database
python -c "from src.pipeline import FinancePipeline; p = FinancePipeline(); p.backup()"
```

### Monthly Tasks
```bash
# Optimize database
python -c "from src.pipeline import FinancePipeline; p = FinancePipeline(); p.vacuum()"
```

---

## Automation (Cron)

Add to crontab:

```bash
# Daily price updates (6 PM EST, Mon-Fri)
0 18 * * 1-5 cd /path/to/FinancePipeline && python scripts/fetch_prices.py --watchlist-name default

# Weekly symbol refresh (Sunday 2 AM)
0 2 * * 0 cd /path/to/FinancePipeline && python scripts/load_symbols.py --download --all

# Monthly optimization (1st of month, 3 AM)
0 3 1 * * cd /path/to/FinancePipeline && python -c "from src.pipeline import FinancePipeline; p = FinancePipeline(); p.backup(); p.vacuum()"
```

---

## Integration Points

### PhiVector/PhiSHRI
- Use with multiplexor for async loading
- Integrate with database MCP tools
- Coordinate with other agents (DC, STRYK, QUANT)

### External Tools
- Export to CSV for Excel/Google Sheets
- Connect to visualization tools (Plotly, Matplotlib)
- Feed into backtesting frameworks
- API integration with trading platforms

---

## Troubleshooting

**"API rate limit exceeded"**
```python
# Check usage
pipeline = FinancePipeline()
usage = pipeline.get_api_usage('alpha_vantage', hours=24)
print(f"Used {usage}/25 calls today")
```

**"Database locked"**
```python
# Optimize and rebuild
pipeline.vacuum()
```

**"No data returned"**
- Verify API key is configured
- Check symbol exists in database
- Try different data source

**"Slow queries"**
- Schema includes optimized indexes
- Run VACUUM and ANALYZE monthly
- Consider date-based partitioning for large datasets

---

## Documentation

- **Quick Start:** `docs/QUICKSTART.md`
- **PhiSHRI Door:** 865 (complete reference)
- **Examples:** `examples/workflow.py`
- **Config Reference:** `config/config.example.json`

---

## Next Steps

1. **Backtesting Framework** - Add strategy testing engine
2. **Real-time Streaming** - WebSocket integration
3. **Advanced Analytics** - Technical indicators, correlations
4. **Visualization** - Dashboard with Plotly/Dash
5. **Alert System** - Price alerts, condition triggers
6. **Portfolio Tracking** - Position tracking, P&L

---

## License

MIT License

---

## Support

- PhiSHRI Door: 865
- Category: TOOLS.DATABASE.FINANCE
- Agent Affinity: DC, STRYK, QUANT
- Prerequisites: Door 800 (Database Tools)

**Created for:** Local-first financial analysis without API dependency  
**Philosophy:** Load once, analyze forever
