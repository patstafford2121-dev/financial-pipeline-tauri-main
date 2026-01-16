import sqlite3
from datetime import datetime, timedelta
import random

# Connect to database
conn = sqlite3.connect('data/finance.db')
cursor = conn.cursor()

# Insert test symbols
print("Loading test symbols...")
test_symbols = [
    ('AAPL', 'Apple Inc', 'equity', 'Technology', 'Consumer Electronics', '2900000000000'),
    ('MSFT', 'Microsoft Corporation', 'equity', 'Technology', 'Software', '2800000000000'),
    ('GOOGL', 'Alphabet Inc', 'equity', 'Technology', 'Internet', '1700000000000'),
    ('AMZN', 'Amazon.com Inc', 'equity', 'Consumer Cyclical', 'Internet Retail', '1500000000000'),
    ('NVDA', 'NVIDIA Corporation', 'equity', 'Technology', 'Semiconductors', '1200000000000'),
    ('TSLA', 'Tesla Inc', 'equity', 'Consumer Cyclical', 'Auto Manufacturers', '800000000000'),
    ('META', 'Meta Platforms Inc', 'equity', 'Technology', 'Internet', '900000000000'),
    ('JPM', 'JPMorgan Chase & Co', 'equity', 'Financial Services', 'Banks', '500000000000'),
    ('V', 'Visa Inc', 'equity', 'Financial Services', 'Credit Services', '520000000000'),
    ('WMT', 'Walmart Inc', 'equity', 'Consumer Defensive', 'Discount Stores', '400000000000'),
]

for symbol, name, asset_class, sector, industry, market_cap in test_symbols:
    cursor.execute("""
        INSERT OR REPLACE INTO symbols 
        (symbol, name, asset_class, sector, industry, market_cap)
        VALUES (?, ?, ?, ?, ?, ?)
    """, [symbol, name, asset_class, sector, industry, market_cap])

conn.commit()
print(f"✓ Loaded {len(test_symbols)} test symbols")

# Generate sample price data
print("Generating price data...")
base_prices = {
    'AAPL': 180, 'MSFT': 375, 'GOOGL': 142, 'AMZN': 155,
    'NVDA': 520, 'TSLA': 245, 'META': 390, 'JPM': 160,
    'V': 250, 'WMT': 170
}

records = 0
for days_ago in range(90, -1, -1):
    date = (datetime.now() - timedelta(days=days_ago)).strftime('%Y-%m-%d')
    
    for symbol, base_price in base_prices.items():
        variation = random.uniform(-0.05, 0.05)
        price = base_price * (1 + variation)
        
        open_price = price * random.uniform(0.98, 1.02)
        high_price = max(open_price, price) * random.uniform(1.0, 1.03)
        low_price = min(open_price, price) * random.uniform(0.97, 1.0)
        volume = int(random.uniform(50000000, 150000000))
        
        cursor.execute("""
            INSERT OR REPLACE INTO daily_prices 
            (symbol, timestamp, open, high, low, close, volume)
            VALUES (?, ?, ?, ?, ?, ?, ?)
        """, [symbol, date, open_price, high_price, low_price, price, volume])
        
        records += 1

conn.commit()
print(f"✓ Generated {records} price records")

# Add macro data
print("Loading macro indicators...")
macro_data = [
    ('GDP', '2024-10-01', 28000.0, 'quarterly'),
    ('UNRATE', '2024-12-01', 3.7, 'monthly'),
    ('DFF', '2025-01-01', 4.5, 'daily'),
    ('CPIAUCSL', '2024-12-01', 308.5, 'monthly'),
]

for indicator, date, value, frequency in macro_data:
    cursor.execute("""
        INSERT OR REPLACE INTO macro_data 
        (indicator, date, value, frequency)
        VALUES (?, ?, ?, ?)
    """, [indicator, date, value, frequency])

conn.commit()
print(f"✓ Loaded {len(macro_data)} macro indicators")

# Verify
symbol_count = cursor.execute("SELECT COUNT(*) FROM symbols").fetchone()[0]
price_count = cursor.execute("SELECT COUNT(*) FROM daily_prices").fetchone()[0]
macro_count = cursor.execute("SELECT COUNT(*) FROM macro_data").fetchone()[0]

print("\n" + "="*50)
print("✅ TEST DATA LOADED SUCCESSFULLY!")
print("="*50)
print(f"  Symbols:         {symbol_count}")
print(f"  Price Records:   {price_count}")
print(f"  Macro Indicators: {macro_count}")
print("="*50)
print("\nNow restart the GUI:")
print("  python -m streamlit run app.py")

conn.close()