"""
Financial Data Pipeline - Main Class
Door 865: Financial Data Pipeline - SQLite Edition
"""

import sqlite3
import pandas as pd
import json
import os
from pathlib import Path
from datetime import datetime, timedelta
from typing import Optional, List, Dict, Any
import time


class FinancePipeline:
    """Main pipeline class for financial data management"""
    
    def __init__(self, config_path: str = "config/config.json"):
        """Initialize pipeline with configuration"""
        self.config = self._load_config(config_path)
        self.db_path = self.config['database']['path']
        self.conn = None
        self.api_keys = self.config.get('api_keys', {})
        
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
        """Log an API call for rate limiting"""
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
    
    def fetch_prices_alpha(self, symbol: str, outputsize: str = "compact"):
        """
        Fetch daily prices from Alpha Vantage
        
        Args:
            symbol: Stock symbol
            outputsize: 'compact' (100 days) or 'full' (20+ years)
        """
        if not self.api_keys.get('alpha_vantage'):
            raise ValueError("Alpha Vantage API key not configured")
        
        import requests
        
        # Check rate limits
        usage = self.get_api_usage('alpha_vantage', hours=24)
        if usage >= 25:
            raise Exception("Alpha Vantage rate limit exceeded (25/day)")
        
        # API call
        url = "https://www.alphavantage.co/query"
        params = {
            'function': 'TIME_SERIES_DAILY',
            'symbol': symbol,
            'outputsize': outputsize,
            'apikey': self.api_keys['alpha_vantage']
        }
        
        response = requests.get(url, params=params)
        data = response.json()
        
        # Log API call
        self.log_api_call('alpha_vantage', 'TIME_SERIES_DAILY', symbol)
        
        # Check for errors
        if 'Error Message' in data:
            raise Exception(f"API Error: {data['Error Message']}")
        
        if 'Note' in data:
            raise Exception("API rate limit exceeded")
        
        if 'Time Series (Daily)' not in data:
            raise Exception(f"No data returned for {symbol}")
        
        # Parse and insert data
        time_series = data['Time Series (Daily)']
        
        conn = self.connect()
        cursor = conn.cursor()
        
        count = 0
        for date, values in time_series.items():
            cursor.execute("""
                INSERT OR REPLACE INTO daily_prices 
                (symbol, timestamp, open, high, low, close, volume, source)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            """, [
                symbol,
                date,
                float(values['1. open']),
                float(values['2. high']),
                float(values['3. low']),
                float(values['4. close']),
                int(values['5. volume']),
                'alpha_vantage'
            ])
            count += 1
        
        conn.commit()

        print(f"✓ Loaded {count} records for {symbol}")

        # Delay to respect rate limits (5 calls per minute for free tier)
        time.sleep(12)
    
    def fetch_prices_finnhub(self, symbol: str, days: int = 365):
        """
        Fetch daily prices from Finnhub
        
        Args:
            symbol: Stock symbol
            days: Number of days of historical data (default 365)
        """
        if not self.api_keys.get('finnhub'):
            raise ValueError("Finnhub API key not configured")
        
        import requests
        
        # Calculate date range
        end_date = datetime.now()
        start_date = end_date - timedelta(days=days)
        
        # Convert to Unix timestamps
        end_ts = int(end_date.timestamp())
        start_ts = int(start_date.timestamp())
        
        # API call
        url = "https://finnhub.io/api/v1/stock/candle"
        params = {
            'symbol': symbol,
            'resolution': 'D',  # Daily
            'from': start_ts,
            'to': end_ts,
            'token': self.api_keys['finnhub']
        }
        
        response = requests.get(url, params=params)
        data = response.json()
        
        # Log API call
        self.log_api_call('finnhub', 'stock/candle', symbol)
        
        # Check for errors
        if data.get('s') == 'no_data':
            raise Exception(f"No data returned for {symbol}")
        
        if 'error' in data:
            raise Exception(f"API Error: {data['error']}")
        
        # Parse and insert data
        conn = self.connect()
        cursor = conn.cursor()
        
        count = 0
        timestamps = data.get('t', [])
        opens = data.get('o', [])
        highs = data.get('h', [])
        lows = data.get('l', [])
        closes = data.get('c', [])
        volumes = data.get('v', [])
        
        for i in range(len(timestamps)):
            # Convert Unix timestamp to date
            date = datetime.fromtimestamp(timestamps[i]).strftime('%Y-%m-%d')
            
            cursor.execute("""
                INSERT OR REPLACE INTO daily_prices 
                (symbol, timestamp, open, high, low, close, volume, source)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            """, [
                symbol,
                date,
                float(opens[i]),
                float(highs[i]),
                float(lows[i]),
                float(closes[i]),
                int(volumes[i]),
                'finnhub'
            ])
            count += 1
        
        conn.commit()
        
        print(f"✓ Loaded {count} records for {symbol}")
        
        # Small delay to respect rate limits (60 calls/minute)
        time.sleep(1)
    
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
        
        print(f"✓ Loaded {count} records for {indicator}")
    
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
        print(f"✓ Loaded {count} symbols")
    
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
        
        # Create watchlist
        cursor.execute("""
            INSERT OR REPLACE INTO watchlists (name)
            VALUES (?)
        """, [name])
        
        watchlist_id = cursor.lastrowid
        
        # Add symbols
        for symbol in symbols:
            cursor.execute("""
                INSERT OR REPLACE INTO watchlist_symbols (watchlist_id, symbol)
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
