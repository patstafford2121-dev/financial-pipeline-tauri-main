#!/usr/bin/env python3
"""
Sync Symbols - Ensure all symbols in daily_prices exist in symbols table
Door 865: Financial Data Pipeline - SQLite Edition
"""

import sys
import os

# Add src to path
sys.path.insert(0, os.path.dirname(__file__))

from src.pipeline import FinancePipeline


def main():
    print("=" * 60)
    print("Sync Symbols - Financial Data Pipeline")
    print("=" * 60)

    pipeline = FinancePipeline()
    pipeline.connect()

    # Find symbols in daily_prices that are missing from symbols table
    missing = pipeline.query("""
        SELECT DISTINCT symbol
        FROM daily_prices
        WHERE symbol NOT IN (SELECT symbol FROM symbols)
        ORDER BY symbol
    """)

    if missing.empty:
        print("\nAll symbols are already synced!")
        return 0

    missing_symbols = missing['symbol'].tolist()
    print(f"\nFound {len(missing_symbols)} symbols to add:")
    for sym in missing_symbols:
        print(f"  - {sym}")

    # Insert missing symbols
    cursor = pipeline.conn.cursor()
    count = 0

    for symbol in missing_symbols:
        cursor.execute("""
            INSERT OR IGNORE INTO symbols (symbol, name, asset_class)
            VALUES (?, ?, 'equity')
        """, [symbol, symbol])
        count += 1

    pipeline.conn.commit()

    print(f"\nAdded {count} symbols to the database.")
    print("=" * 60)

    return 0


if __name__ == "__main__":
    sys.exit(main())
