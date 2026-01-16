-- Financial Data Pipeline Schema
-- Door 865: Financial Data Pipeline - SQLite Edition

-- Symbol master table (from FinanceDatabase)
CREATE TABLE IF NOT EXISTS symbols (
    symbol TEXT PRIMARY KEY,
    name TEXT,
    sector TEXT,
    industry TEXT,
    market_cap REAL,
    country TEXT,
    exchange TEXT,
    currency TEXT,
    isin TEXT,
    asset_class TEXT,  -- 'equity', 'etf', 'crypto', 'forex'
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Daily price data (from APIs)
CREATE TABLE IF NOT EXISTS daily_prices (
    symbol TEXT,
    timestamp DATE,
    open REAL,
    high REAL,
    low REAL,
    close REAL,
    volume INTEGER,
    adjusted_close REAL,
    source TEXT,  -- 'alpha_vantage', 'finnhub', 'fmp', etc.
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (symbol, timestamp),
    FOREIGN KEY (symbol) REFERENCES symbols(symbol)
);

-- Intraday price data (optional, for higher frequency analysis)
CREATE TABLE IF NOT EXISTS intraday_prices (
    symbol TEXT,
    timestamp DATETIME,
    open REAL,
    high REAL,
    low REAL,
    close REAL,
    volume INTEGER,
    source TEXT,
    PRIMARY KEY (symbol, timestamp),
    FOREIGN KEY (symbol) REFERENCES symbols(symbol)
);

-- Fundamental data (quarterly/annual)
CREATE TABLE IF NOT EXISTS fundamentals (
    symbol TEXT,
    period_end DATE,
    period_type TEXT,  -- 'quarterly', 'annual'
    revenue REAL,
    net_income REAL,
    earnings REAL,
    eps REAL,
    eps_diluted REAL,
    pe_ratio REAL,
    pb_ratio REAL,
    div_yield REAL,
    total_assets REAL,
    total_liabilities REAL,
    shareholders_equity REAL,
    operating_cash_flow REAL,
    source TEXT,
    PRIMARY KEY (symbol, period_end, period_type),
    FOREIGN KEY (symbol) REFERENCES symbols(symbol)
);

-- Macro economic indicators (from FRED)
CREATE TABLE IF NOT EXISTS macro_data (
    indicator TEXT,  -- 'GDP', 'UNRATE', 'DFF', 'CPI', etc.
    date DATE,
    value REAL,
    frequency TEXT,  -- 'daily', 'monthly', 'quarterly', 'annual'
    source TEXT DEFAULT 'FRED',
    PRIMARY KEY (indicator, date)
);

-- News/events (from Finnhub or similar)
CREATE TABLE IF NOT EXISTS news (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol TEXT,
    headline TEXT,
    summary TEXT,
    url TEXT,
    source TEXT,
    published_at DATETIME,
    sentiment REAL,  -- -1 to 1, if available
    FOREIGN KEY (symbol) REFERENCES symbols(symbol)
);

-- Watchlists (user-defined)
CREATE TABLE IF NOT EXISTS watchlists (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE,
    description TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS watchlist_symbols (
    watchlist_id INTEGER,
    symbol TEXT,
    added_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    notes TEXT,
    PRIMARY KEY (watchlist_id, symbol),
    FOREIGN KEY (watchlist_id) REFERENCES watchlists(id),
    FOREIGN KEY (symbol) REFERENCES symbols(symbol)
);

-- API call tracking (rate limit management)
CREATE TABLE IF NOT EXISTS api_calls (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source TEXT,  -- 'alpha_vantage', 'finnhub', etc.
    endpoint TEXT,
    symbol TEXT,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    success BOOLEAN,
    error_message TEXT
);

-- Backtesting results
CREATE TABLE IF NOT EXISTS backtest_results (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    strategy_name TEXT,
    symbol TEXT,
    start_date DATE,
    end_date DATE,
    initial_capital REAL,
    final_capital REAL,
    total_return REAL,
    sharpe_ratio REAL,
    max_drawdown REAL,
    num_trades INTEGER,
    win_rate REAL,
    config JSON,  -- Store strategy parameters
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_prices_symbol ON daily_prices(symbol);
CREATE INDEX IF NOT EXISTS idx_prices_timestamp ON daily_prices(timestamp);
CREATE INDEX IF NOT EXISTS idx_prices_source ON daily_prices(source);
CREATE INDEX IF NOT EXISTS idx_symbols_sector ON symbols(sector);
CREATE INDEX IF NOT EXISTS idx_symbols_exchange ON symbols(exchange);
CREATE INDEX IF NOT EXISTS idx_symbols_country ON symbols(country);
CREATE INDEX IF NOT EXISTS idx_macro_indicator ON macro_data(indicator);
CREATE INDEX IF NOT EXISTS idx_macro_date ON macro_data(date);
CREATE INDEX IF NOT EXISTS idx_news_symbol ON news(symbol);
CREATE INDEX IF NOT EXISTS idx_news_published ON news(published_at);
CREATE INDEX IF NOT EXISTS idx_api_calls_source ON api_calls(source);
CREATE INDEX IF NOT EXISTS idx_api_calls_timestamp ON api_calls(timestamp);
CREATE INDEX IF NOT EXISTS idx_fundamentals_symbol ON fundamentals(symbol);
CREATE INDEX IF NOT EXISTS idx_fundamentals_period ON fundamentals(period_end);

-- Views for common queries
CREATE VIEW IF NOT EXISTS latest_prices AS
SELECT p.* 
FROM daily_prices p
INNER JOIN (
    SELECT symbol, MAX(timestamp) as max_date
    FROM daily_prices
    GROUP BY symbol
) latest ON p.symbol = latest.symbol AND p.timestamp = latest.max_date;

CREATE VIEW IF NOT EXISTS symbol_summary AS
SELECT 
    s.symbol,
    s.name,
    s.sector,
    s.industry,
    s.exchange,
    lp.close as last_price,
    lp.volume as last_volume,
    lp.timestamp as last_updated
FROM symbols s
LEFT JOIN latest_prices lp ON s.symbol = lp.symbol;

CREATE VIEW IF NOT EXISTS api_rate_limits AS
SELECT 
    source,
    COUNT(*) as calls_today,
    MAX(timestamp) as last_call
FROM api_calls
WHERE DATE(timestamp) = DATE('now')
GROUP BY source;
