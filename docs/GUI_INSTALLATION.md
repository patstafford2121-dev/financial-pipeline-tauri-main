# GUI Installation & Testing Guide

Quick guide to get your Financial Data Pipeline GUI up and running.

---

## Prerequisites

- Python 3.8+
- pip (Python package manager)
- Internet connection (for package installation)

---

## Step 1: Install Dependencies

```bash
cd FinancePipeline
pip install -r requirements.txt
```

**This installs:**
- streamlit (web framework)
- plotly (interactive charts)
- pandas (data processing)
- requests (API calls)
- python-dotenv (config management)

---

## Step 2: Initialize Database

```bash
# Create database and schema
python scripts/init_db.py
```

**Creates:**
- `data/finance.db` - SQLite database
- All tables (symbols, daily_prices, fundamentals, etc.)
- Indexes for performance
- Views for common queries

---

## Step 3: Configure API Keys (Optional)

```bash
# Copy example config
cp config/config.example.json config/config.json

# Edit with your favorite editor
nano config/config.json  # or vim, code, etc.
```

**Add your API keys:**
```json
{
  "api_keys": {
    "alpha_vantage": "YOUR_KEY_HERE",
    "finnhub": "YOUR_KEY_HERE",
    "fmp": "YOUR_KEY_HERE"
  }
}
```

**Get Free API Keys:**
- Alpha Vantage: https://www.alphavantage.co/support/#api-key
- Finnhub: https://finnhub.io/register
- FMP: https://site.financialmodelingprep.com/developer/docs

**Note:** You can test the GUI without API keys - just won't be able to fetch data yet.

---

## Step 4: Launch GUI

```bash
# Easy launch
python launch_gui.py

# Or direct launch
streamlit run app.py
```

**Expected Output:**
```
  You can now view your Streamlit app in your browser.

  Local URL: http://localhost:8501
  Network URL: http://192.168.1.100:8501
```

**Browser should auto-open to:** `http://localhost:8501`

---

## Step 5: Test Core Features

### Test 1: Dashboard
✅ Navigate to "🏠 Dashboard"
- Should see 4 metric boxes
- Database status section
- Empty charts (no data yet)

### Test 2: Symbol Browser
✅ Navigate to "🔍 Symbol Browser"
- Leave search empty, click search
- Should show message "No symbols yet" or empty results
- This is normal - symbols not loaded yet

### Test 3: Settings
✅ Navigate to "⚙️ Settings"
- Check API Keys tab - should show your configured keys (masked)
- Check Database tab - should show database size
- Click "Vacuum" - should succeed

---

## Step 6: Load Sample Data

### Load a few symbols manually

```bash
# Option 1: Load from FinanceDatabase (300k+ symbols)
python scripts/load_symbols.py --download

# Option 2: Insert test symbols via SQL
python -c "
from src.pipeline import FinancePipeline
p = FinancePipeline()
p.connect()
p.query('''
INSERT INTO symbols (symbol, name, asset_class, sector) VALUES 
('AAPL', 'Apple Inc', 'equity', 'Technology'),
('MSFT', 'Microsoft Corp', 'equity', 'Technology'),
('GOOGL', 'Alphabet Inc', 'equity', 'Technology')
''')
print('✅ Test symbols loaded')
"
```

### Verify in GUI
1. Refresh dashboard (press R or refresh browser)
2. Go to Symbol Browser
3. Search for "Apple" - should find AAPL

### Fetch Price Data (requires API key)

```bash
# Using script
python scripts/fetch_prices.py --symbols AAPL

# Or use GUI Data Fetcher
# 1. Navigate to "📥 Data Fetcher"
# 2. Enter "AAPL" in Symbols field
# 3. Select "alpha_vantage" source
# 4. Select "compact" time range
# 5. Click "Fetch Prices"
```

### Verify Price Data
1. Go to Dashboard
2. Should see "Recent Price Updates" populated
3. Go to "Query & Analysis" → "Analysis" tab
4. Enter "AAPL" and click "Analyze"
5. Should see price chart

---

## Step 7: Advanced Testing

### Test SQL Editor
```sql
-- Go to "Query & Analysis" → "SQL Editor"
-- Run this query:

SELECT 
    s.symbol, 
    s.name, 
    COUNT(p.timestamp) as days_of_data
FROM symbols s
LEFT JOIN daily_prices p ON s.symbol = p.symbol
GROUP BY s.symbol
ORDER BY days_of_data DESC;
```

### Test Watchlist
1. Go to "⭐ Watchlists"
2. Create new watchlist:
   - Name: "Test Portfolio"
   - Symbols: "AAPL, MSFT, GOOGL"
3. Click "Create"
4. Select from dropdown
5. Should show table with symbols

---

## Troubleshooting

### "Module not found: streamlit"
```bash
pip install streamlit
```

### "Database not found"
```bash
python scripts/init_db.py
```

### "Port 8501 already in use"
```bash
# Kill existing streamlit
pkill -f streamlit

# Or use different port
streamlit run app.py --server.port 8502
```

### "API rate limit exceeded"
- Go to Settings → Rate Limits
- Click "Clear API Call History"
- Or wait 24 hours for Alpha Vantage reset

### GUI won't load / blank page
1. Check terminal for errors
2. Try clearing browser cache
3. Try incognito/private window
4. Restart streamlit

### Performance issues
```bash
# Optimize database
python -c "from src.pipeline import FinancePipeline; p = FinancePipeline(); p.connect(); p.vacuum()"
```

---

## Production Deployment

### Local Network Access

```bash
# Allow connections from other devices on your network
streamlit run app.py --server.address 0.0.0.0
```

Access from other devices: `http://YOUR_IP:8501`

### Run in Background

```bash
# Using nohup
nohup streamlit run app.py > gui.log 2>&1 &

# Using screen
screen -S finance-gui
streamlit run app.py
# Press Ctrl+A then D to detach
```

### Custom Domain (with reverse proxy)

```nginx
# nginx config
server {
    listen 80;
    server_name finance.yourdomain.com;
    
    location / {
        proxy_pass http://localhost:8501;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
    }
}
```

---

## Performance Tips

**For Large Datasets (>1M rows):**

1. **Use indexes** (already included in schema)
2. **Limit results** in queries
3. **Filter by date** when possible
4. **Run VACUUM** monthly
5. **Use views** for complex queries

**Example Optimized Query:**
```sql
-- Bad: Scans entire table
SELECT * FROM daily_prices WHERE symbol = 'AAPL';

-- Good: Uses index + limit
SELECT * FROM daily_prices 
WHERE symbol = 'AAPL' 
AND timestamp >= date('now', '-90 days')
ORDER BY timestamp DESC 
LIMIT 100;
```

---

## Next Steps

✅ **Loaded sample data** → Load full symbol catalog  
✅ **Tested GUI** → Create real watchlists  
✅ **Ran queries** → Build custom analysis  

**Recommended Workflow:**

1. **Daily:** Update watchlist prices via GUI
2. **Weekly:** Load new symbols if needed
3. **Monthly:** Run database vacuum/optimization
4. **As needed:** Use SQL editor for custom research

---

## Documentation

- **GUI Guide:** `docs/GUI_GUIDE.md` (comprehensive)
- **Quick Start:** `docs/QUICKSTART.md` (CLI focus)
- **Main README:** `README.md` (project overview)
- **API Sources:** `docs/API_SOURCES.md` (not yet created)

---

## Support

**Common Questions:**

**Q: Can I use this without API keys?**  
A: Yes! You can browse loaded data and run queries. You just can't fetch new data.

**Q: Is this production-ready?**  
A: For personal use, yes. For multi-user, add authentication.

**Q: Can I deploy to cloud?**  
A: Yes! Works on AWS, GCP, Azure, Heroku, etc.

**Q: Mobile friendly?**  
A: Streamlit is responsive, works on tablets/phones.

---

## Success Checklist

- [ ] Python 3.8+ installed
- [ ] Dependencies installed (`pip install -r requirements.txt`)
- [ ] Database initialized (`python scripts/init_db.py`)
- [ ] Config created (`cp config/config.example.json config/config.json`)
- [ ] GUI launches (`python launch_gui.py`)
- [ ] Dashboard loads in browser
- [ ] Sample symbols loaded
- [ ] Price data fetched (optional, requires API key)
- [ ] SQL query tested
- [ ] Watchlist created

**All checked?** You're ready to go! 🚀

---

**Built for:** Local-first financial analysis  
**Philosophy:** Load once, query forever  
**License:** MIT
