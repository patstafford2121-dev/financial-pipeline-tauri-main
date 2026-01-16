"""
Financial Data Pipeline - Main Class (YAHOO FINANCE VERSION)
Door 865: Financial Data Pipeline - SQLite Edition

✅ NEW: Uses Yahoo Finance (yfinance) instead of Alpha Vantage
Benefits:
- FREE and UNLIMITED (no API key needed!)
- Adjusted data by default
- Faster bulk downloads
- More reliable
"""

import sqlite3
import pandas as pd
import json
import os
from pathlib import Path
from datetime import datetime, timedelta
from typing import Optional, List, Dict, Any


class FinancePipeline:
    """Main pipeline class for financial data management"""

    def __init__(self, config_path: str = "config/config.json"):
        """Initialize pipeline with configuration"""
        self.config = self._load_config(config_path)
        self.db_path = self.config['database']['path']
        self.conn = None
        
    def _load_config(self, config_path: str) -> Dict[str, Any]:
        """Load configuration from JSON file"""
        if not os.path.exists(config_path):
            # Try to load example config
            example_path = config_path.replace('.json', '.example.json')
            if os.path.exists(example_path):
                print(f"Warning: {config_path} not found, using {example_path}")
                config_path = example_path
            else:
                raise FileNotFoundError(f"Config file not found: {config_path}")
                
        with open(config_path, 'r') as f:
            return json.load(f)
    
    def connect(self):
        """Establish database connection"""
        os.makedirs(os.path.dirname(self.db_path), exist_ok=True)
        self.conn = sqlite3.connect(self.db_path, check_same_thread=False)
        self.conn.row_factory = sqlite3.Row
        return self.conn
    
    def disconnect(self):
        """Close database connection"""
        if self.conn:
            self.conn.close()
            self.conn = None
    
    def query(self, sql: str, params: Optional[List] = None) -> pd.DataFrame:
        """
        Execute SQL query and return DataFrame
        
        Args:
            sql: SQL query string
            params: Optional parameters for parameterized queries
        """
        self.connect()
        
        if params:
            return pd.read_sql_query(sql, self.conn, params=params)
        else:
            return pd.read_sql_query(sql, self.conn)
    
    def log_api_call(self, source: str, endpoint: str = '', symbol: str = ''):
        """Log an API call for tracking"""
        self.connect()
        cursor = self.conn.cursor()
        
        cursor.execute("""
            INSERT INTO api_calls (source, endpoint, symbol, timestamp)
            VALUES (?, ?, ?, ?)
        """, [source, endpoint, symbol, datetime.now().isoformat()])
        
        self.conn.commit()
    
    def get_api_usage(self, source: str, hours: int = 24) -> int:
        """Get API call count for a source within time window"""
        self.connect()
        
        cutoff = (datetime.now() - timedelta(hours=hours)).isoformat()
        
        df = self.query("""
            SELECT COUNT(*) as count
            FROM api_calls
            WHERE source = ?
            AND timestamp >= ?
        """, [source, cutoff])
        
        return int(df['count'].iloc[0]) if not df.empty else 0
    
    def fetch_prices_yahoo(self, symbol: str, period: str = "1y"):
        """
        Fetch daily prices from Yahoo Finance using yfinance
        
        ✅ BENEFITS:
        - FREE and UNLIMITED (no API key needed)
        - Adjusted data by default
        - Fast and reliable
        
        Args:
            symbol: Stock symbol
            period: Time period - "1d", "5d", "1mo", "3mo", "6mo", "1y", "2y", "5y", "10y", "ytd", "max"
        """
        try:
            import yfinance as yf
        except ImportError:
            raise ImportError("yfinance not installed. Run: pip install yfinance")
        
        print(f"[FETCH] Fetching {symbol} from Yahoo Finance (period: {period})...")
        
        # Download data
        ticker = yf.Ticker(symbol)
        df = ticker.history(period=period)
        
        if df.empty:
            raise Exception(f"No data returned for {symbol}")
        
        # Log API call
        self.log_api_call('yahoo_finance', 'history', symbol)
        
        # Insert data into database
        conn = self.connect()
        cursor = conn.cursor()
        
        count = 0
        for date, row in df.iterrows():
            # Yahoo Finance returns adjusted data by default
            cursor.execute("""
                INSERT OR REPLACE INTO daily_prices 
                (symbol, timestamp, open, high, low, close, volume, source)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            """, [
                symbol,
                date.strftime('%Y-%m-%d'),
                float(row['Open']),
                float(row['High']),
                float(row['Low']),
                float(row['Close']),  # Already adjusted!
                int(row['Volume']),
                'yahoo_finance'
            ])
            count += 1
        
        conn.commit()
        print(f"[OK] Loaded {count} records for {symbol}")
        
        # No delay needed - Yahoo Finance has no rate limits!
    
    def fetch_prices_batch_yahoo(self, symbols: List[str], period: str = "1y"):
        """
        Fetch prices for multiple symbols at once (FAST!)
        
        This is MUCH faster than Alpha Vantage because:
        - No rate limits
        - Can download multiple symbols in parallel
        
        Args:
            symbols: List of stock symbols
            period: Time period for all symbols
        """
        try:
            import yfinance as yf
        except ImportError:
            raise ImportError("yfinance not installed. Run: pip install yfinance")
        
        print(f"[FETCH] Batch fetching {len(symbols)} symbols from Yahoo Finance...")
        print(f"Period: {period}")
        print("=" * 60)
        
        success_count = 0
        fail_count = 0
        
        for i, symbol in enumerate(symbols, 1):
            print(f"\n[{i}/{len(symbols)}] {symbol}...", end=" ")
            
            try:
                self.fetch_prices_yahoo(symbol, period)
                success_count += 1
                print("[OK]")
            except Exception as e:
                print(f"[FAIL] {e}")
                fail_count += 1
        
        print("\n" + "=" * 60)
        print(f"[OK] Batch fetch complete!")
        print(f"  Success: {success_count}/{len(symbols)}")
        print(f"  Failed: {fail_count}/{len(symbols)}")
    
    def fetch_fred(self, indicator: str):
        """
        Fetch macro data from FRED (Federal Reserve Economic Data)
        
        Args:
            indicator: FRED series ID (e.g., 'GDP', 'UNRATE', 'DFF')
        """
        import requests
        
        # FRED API (no key required for basic access)
        url = f"https://fred.stlouisfed.org/graph/fredgraph.csv?id={indicator}"
        
        response = requests.get(url)
        response.raise_for_status()
        
        # Parse CSV
        from io import StringIO
        df = pd.read_csv(StringIO(response.text))
        
        # Insert into database
        conn = self.connect()
        cursor = conn.cursor()
        
        count = 0
        for _, row in df.iterrows():
            if pd.notna(row.iloc[1]):  # Skip null values
                cursor.execute("""
                    INSERT OR REPLACE INTO macro_data 
                    (indicator, date, value, source)
                    VALUES (?, ?, ?, ?)
                """, [indicator, row.iloc[0], float(row.iloc[1]), 'FRED'])
                count += 1
        
        conn.commit()
        print(f"[OK] Loaded {count} records for {indicator}")
    
    def load_symbols(self, json_path: str):
        """Load symbols from FinanceDatabase JSON file"""
        with open(json_path, 'r') as f:
            data = json.load(f)
        
        conn = self.connect()
        cursor = conn.cursor()
        
        count = 0
        for symbol, info in data.items():
            cursor.execute("""
                INSERT OR REPLACE INTO symbols 
                (symbol, name, sector, industry, market_cap, country, exchange, 
                 currency, isin, asset_class)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            """, [
                symbol,
                info.get('name'),
                info.get('sector'),
                info.get('industry'),
                info.get('market_cap'),
                info.get('country'),
                info.get('exchange'),
                info.get('currency'),
                info.get('isin'),
                info.get('asset_class', 'equity')
            ])
            count += 1
        
        conn.commit()
        print(f"[OK] Loaded {count} symbols")
    
    def get_latest_price(self, symbol: str) -> Optional[float]:
        """Get most recent price for a symbol"""
        df = self.query("""
            SELECT close
            FROM daily_prices
            WHERE symbol = ?
            ORDER BY timestamp DESC
            LIMIT 1
        """, [symbol])
        
        return float(df['close'].iloc[0]) if not df.empty else None
    
    def create_watchlist(self, name: str, symbols: List[str]):
        """Create or update a watchlist"""
        conn = self.connect()
        cursor = conn.cursor()
        
        # Delete existing watchlist entries
        cursor.execute("DELETE FROM watchlists WHERE name = ?", [name])
        
        # Create watchlist
        cursor.execute("""
            INSERT INTO watchlists (name)
            VALUES (?)
        """, [name])
        
        watchlist_id = cursor.lastrowid
        
        # Add symbols
        for symbol in symbols:
            cursor.execute("""
                INSERT INTO watchlist_symbols (watchlist_id, symbol)
                VALUES (?, ?)
            """, [watchlist_id, symbol])
        
        conn.commit()
    
    def get_watchlist(self, name: str) -> List[str]:
        """Get symbols in a watchlist"""
        df = self.query("""
            SELECT ws.symbol
            FROM watchlists w
            JOIN watchlist_symbols ws ON w.id = ws.watchlist_id
            WHERE w.name = ?
        """, [name])
        
        return df['symbol'].tolist() if not df.empty else []
    
    def backup(self, backup_dir: str = "data/backups") -> str:
        """Create database backup"""
        os.makedirs(backup_dir, exist_ok=True)
        
        timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')
        backup_path = f"{backup_dir}/finance_{timestamp}.db"
        
        # Copy database
        import shutil
        shutil.copy2(self.db_path, backup_path)
        
        return backup_path
    
    def vacuum(self):
        """Optimize database (reclaim space, rebuild indexes)"""
        self.connect()
        cursor = self.conn.cursor()
        cursor.execute("VACUUM")
        cursor.execute("ANALYZE")
        self.conn.commit()
    
    def close(self):
        """Alias for disconnect()"""
        self.disconnect()
    
    def clear_symbol_prices(self, symbol: str):
        """Delete all price data for a symbol (useful before refetching)"""
        conn = self.connect()
        cursor = conn.cursor()
        cursor.execute("DELETE FROM daily_prices WHERE symbol = ?", [symbol])
        conn.commit()
        print(f"[OK] Cleared price data for {symbol}")
    
    def refetch_all_symbols_yahoo(self, symbols: List[str] = None, period: str = '1y'):
        """
        Refetch data for multiple symbols using Yahoo Finance
        
        Args:
            symbols: List of symbols to refetch (if None, refetches all symbols with existing data)
            period: Time period - "1y", "2y", "5y", "10y", "max"
        """
        if symbols is None:
            # Get all symbols that have price data
            df = self.query("SELECT DISTINCT symbol FROM daily_prices")
            symbols = df['symbol'].tolist()
        
        if not symbols:
            print("[WARN] No symbols to refetch")
            return
        
        print(f"[REFETCH] Refetching {len(symbols)} symbols from Yahoo Finance...")
        print(f"Period: {period}")
        print("=" * 60)
        
        success_count = 0
        fail_count = 0
        
        for i, symbol in enumerate(symbols, 1):
            print(f"\n[{i}/{len(symbols)}] {symbol}...")
            
            try:
                # Clear old data
                self.clear_symbol_prices(symbol)
                
                # Fetch new data from Yahoo Finance
                self.fetch_prices_yahoo(symbol, period=period)
                success_count += 1
                
            except Exception as e:
                print(f"  [FAIL] Failed: {e}")
                fail_count += 1
        
        print("\n" + "=" * 60)
        print(f"[OK] Refetch complete!")
        print(f"  Success: {success_count}")
        print(f"  Failed: {fail_count}")
        print(f"  Total: {len(symbols)}")
