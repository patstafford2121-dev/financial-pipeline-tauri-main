# Financial Data Pipeline - Web GUI Guide

**Modern web-based interface** for your financial data pipeline. Built with Streamlit for intuitive data management and analysis.

---

## Quick Start

### Launch the GUI

```bash
# Option 1: Using launch script
python launch_gui.py

# Option 2: Direct launch
streamlit run app.py

# Option 3: Specify port
streamlit run app.py --server.port 8502
```

The GUI will open in your browser at `http://localhost:8501`

---

## Features Overview

### 🏠 Dashboard
**At-a-glance overview of your financial data system**

**Metrics:**
- Total symbols loaded
- Price records stored
- API usage (24-hour window)
- Latest data timestamp

**Views:**
- Recent price updates table
- Sector breakdown chart
- Database status and size
- Quick backup button

**Use Cases:**
- Check system health before trading hours
- Monitor API quota usage
- Quick validation after data loads
- Database maintenance

---

### 🔍 Symbol Browser
**Search and explore 300k+ symbols from FinanceDatabase**

**Features:**
- Full-text search across symbol, name, sector
- Filter by asset class (equity, ETF, crypto, forex, fund)
- Customizable result limit (10-1000)
- Column selector for custom views
- CSV export

**Search Examples:**
```
"Apple"          → Find all Apple-related symbols
"AAPL"           → Exact symbol match
"Technology"     → All tech sector stocks
"dividend"       → Search in descriptions
```

**Workflow:**
1. Enter search term
2. Select asset class filter
3. Choose columns to display
4. Export results to CSV for further analysis

---

### 📥 Data Fetcher
**Download financial data from multiple sources**

#### Price Data Tab
**Load historical and real-time price data**

**Options:**
- **Source:** Alpha Vantage (EOD) or Finnhub (real-time)
- **Time Range:** 
  - Compact: Last 100 days
  - Full: 20+ years of history

**API Usage Tracking:**
- Shows calls used today vs. quota
- Warns before exceeding limits
- Displays remaining calls

**Batch Loading:**
```
Symbols: AAPL, MSFT, GOOGL, AMZN, META
Source: alpha_vantage
Range: full
```
Result: 5 API calls, ~20 years of data each

**Best Practices:**
- Load full data once, then use compact for updates
- Monitor API quota (Alpha Vantage: 25/day)
- Use watchlists for regular updates

#### Fundamentals Tab
*(Coming Soon)*
- Income statements
- Balance sheets
- Cash flow
- Financial ratios

#### Macro Data Tab
**Economic indicators from FRED (unlimited)**

**Available Indicators:**
- **GDP** - Gross Domestic Product
- **UNRATE** - Unemployment Rate
- **DFF** - Federal Funds Rate
- **CPIAUCSL** - Consumer Price Index
- **DEXUSEU** - USD/EUR Exchange Rate
- **DGS10** - 10-Year Treasury Rate

**No Rate Limits:** FRED API is free and unlimited

---

### 📊 Query & Analysis
**Powerful data exploration and visualization**

#### Quick Queries Tab
**Pre-built analytical queries**

**Top Movers (30 days):**
- Identifies biggest price changes
- Configurable result count
- Shows percentage change

**Sector Performance:**
- Average prices by sector
- Symbol counts
- Visual bar chart

**Latest Prices:**
- Most recent price updates
- Sortable by timestamp
- Export to CSV

**Price History:**
- Complete historical data
- Symbol-specific
- Date range filtering

#### SQL Editor Tab
**Full SQL access to your database**

**Features:**
- Syntax-highlighted editor
- Pre-loaded examples
- Execute any SELECT query
- Results displayed as interactive table
- CSV export

**Example Queries:**
```sql
-- Find tech stocks above $100
SELECT s.symbol, s.name, p.close
FROM symbols s
JOIN daily_prices p ON s.symbol = p.symbol
WHERE s.sector = 'Technology'
AND p.close > 100
ORDER BY p.close DESC;

-- Moving average analysis
SELECT symbol, 
       AVG(close) OVER (ORDER BY timestamp ROWS BETWEEN 19 PRECEDING AND CURRENT ROW) as sma_20
FROM daily_prices
WHERE symbol = 'AAPL'
ORDER BY timestamp DESC;
```

#### Analysis Tab
**Interactive price charting and statistics**

**Features:**
- Time-series price chart (Plotly interactive)
- Key statistics (current, high, low, change%)
- Volume analysis
- Zoom, pan, export capabilities

**Workflow:**
1. Enter symbol (e.g., AAPL)
2. Click "Analyze"
3. View interactive chart
4. Hover for exact values
5. Zoom into date ranges

---

### ⭐ Watchlists
**Organize and track your favorite symbols**

**Features:**
- Create unlimited watchlists
- Add multiple symbols at once
- View latest prices for all symbols
- Batch fetch prices for entire watchlist

**Use Cases:**
- **Portfolio tracking:** Monitor your holdings
- **Sector analysis:** Group tech stocks, energy, etc.
- **Earnings calendar:** Track stocks reporting this week
- **Research lists:** Potential investments

**Creating a Watchlist:**
```
Name: Tech Leaders
Symbols: AAPL, MSFT, GOOGL, AMZN, META, NVDA, TSLA
```

**Batch Update:**
Click "Fetch Prices for All" to update entire watchlist in one operation.

---

### ⚙️ Settings
**System configuration and maintenance**

#### API Keys Tab
**Manage your data source credentials**

- View configured API keys (masked for security)
- Edit via `config/config.json`
- Supports multiple providers

**Providers:**
- Alpha Vantage
- Finnhub
- Financial Modeling Prep
- (FRED requires no key)

#### Rate Limits Tab
**Monitor API usage**

**Alpha Vantage:**
- 25 calls per day limit
- Progress bar shows usage
- Resets every 24 hours

**Finnhub:**
- 60 calls per minute
- Shows recent hour usage

**Features:**
- Clear call history button
- Real-time tracking
- Prevents quota overruns

#### Database Tab
**Maintenance and optimization**

**Operations:**

**Vacuum (Optimize):**
- Reclaims unused space
- Rebuilds indexes
- Improves query speed
- Run monthly

**Backup:**
- Creates timestamped backup
- Saved to `data/backups/`
- Preserves complete database

**Analyze:**
- Updates query planner statistics
- Optimizes query execution
- Run after major data loads

**Info Display:**
- Database file size
- Location on disk
- Growth tracking

---

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+R` | Refresh page |
| `R` | Rerun app (when in focus) |
| `Ctrl+K` | Clear cache |

---

## Tips & Tricks

### Performance Optimization

**Large Datasets:**
- Use LIMIT clauses in custom queries
- Filter by date range when possible
- Index frequently queried columns

**Slow Queries:**
1. Run VACUUM monthly
2. Run ANALYZE after big loads
3. Check query execution plan

### Data Loading Strategy

**Initial Setup:**
```bash
# Day 1: Load symbol catalog
python scripts/load_symbols.py --download --all

# Day 2-4: Load price data (25 symbols/day)
python scripts/fetch_prices.py --symbols AAPL,MSFT,... (25 max)

# Day 5: Load macro indicators (unlimited)
python scripts/fetch_macro.py
```

**Daily Maintenance:**
```bash
# Update watchlist (< 25 symbols)
# Via GUI: Watchlists → Select → Fetch All
```

### Avoiding API Limits

**Alpha Vantage (25/day):**
- Create focused watchlists (≤25 symbols)
- Use "compact" for daily updates
- Use "full" only for initial load
- Schedule updates for after-hours

**Best Practice:**
Load all data once with "full", then only fetch "compact" updates for actively tracked symbols.

---

## Troubleshooting

### "API Rate Limit Exceeded"

**Check Usage:**
1. Go to Settings → Rate Limits
2. View current usage
3. Wait for reset (24h for Alpha Vantage)

**Or Clear History:**
Settings → Rate Limits → Clear API Call History

### "No Data Available"

**Possible Causes:**
1. Symbols not loaded → Use Symbol Browser
2. Prices not fetched → Use Data Fetcher
3. Database empty → Run `scripts/init_db.py`

**Solution:**
```bash
# Initialize database
python scripts/init_db.py

# Load symbols
python scripts/load_symbols.py

# Fetch some data
python scripts/fetch_prices.py --symbols AAPL
```

### "Database Locked"

**Cause:** Multiple connections or unfinished transactions

**Solution:**
1. Close all terminal scripts
2. Restart the GUI
3. Run VACUUM in Settings

### "Streamlit Not Found"

```bash
pip install streamlit
```

### Port Already in Use

```bash
# Use different port
streamlit run app.py --server.port 8502
```

---

## Advanced Usage

### Custom Themes

Create `.streamlit/config.toml`:
```toml
[theme]
primaryColor = "#00ff00"
backgroundColor = "#0e1117"
secondaryBackgroundColor = "#262730"
textColor = "#fafafa"
```

### Headless Mode

```bash
# Run without opening browser
streamlit run app.py --server.headless true
```

### Network Access

```bash
# Allow external connections
streamlit run app.py --server.address 0.0.0.0
```

---

## Integration Examples

### Export to Excel

```python
# In SQL Editor, run query then:
# Download CSV → Open in Excel
```

### API Integration

```python
# Access pipeline from external scripts
from src.pipeline import FinancePipeline

pipeline = FinancePipeline()
pipeline.connect()

# Your custom logic
data = pipeline.query("SELECT * FROM symbols WHERE sector = 'Technology'")
```

### Jupyter Notebooks

```python
# Import pipeline in Jupyter
import sys
sys.path.append('/path/to/FinancePipeline')

from src.pipeline import FinancePipeline
import pandas as pd

pipeline = FinancePipeline()
df = pd.DataFrame(pipeline.query("SELECT * FROM latest_prices"))
```

---

## Security Notes

**API Keys:**
- Never commit `config.json` to version control
- Use `.env.example` as template
- Mask keys in screenshots

**Database:**
- No external network access by default
- Data stored locally only
- Backup regularly

**Network Access:**
- GUI runs on localhost by default
- Only enable external access on trusted networks
- No authentication built-in (add reverse proxy if needed)

---

## Future Enhancements

**Planned Features:**
- [ ] Real-time WebSocket streaming
- [ ] Technical indicators (SMA, RSI, MACD)
- [ ] Alert system (price triggers)
- [ ] Portfolio tracking with P&L
- [ ] Backtesting interface
- [ ] News integration
- [ ] Correlation heatmaps
- [ ] Options data

---

## Support

**Documentation:**
- Main README: `README.md`
- Quick Start: `docs/QUICKSTART.md`
- PhiSHRI Door: 865

**Issues:**
- Check existing docs first
- Verify database initialized
- Confirm API keys configured
- Test with simple query

---

## License

MIT License - See LICENSE file

---

**Built with:**
- Streamlit (Web framework)
- Plotly (Interactive charts)
- SQLite (Local database)
- Pandas (Data processing)

**Philosophy:** Load once, query forever. 📊
