"""
Refetch Script - Reload all price data with ADJUSTED prices
Uses Yahoo Finance (free, provides split-adjusted prices)

This will:
1. Clear old unadjusted data
2. Refetch with split-adjusted prices from Yahoo Finance
3. Give you smooth charts that match Yahoo Finance
"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))

from src.pipeline import FinancePipeline
from datetime import datetime
import time


def main():
    print("=" * 60)
    print("REFETCH DATA WITH ADJUSTED PRICES (Yahoo Finance)")
    print("=" * 60)

    pipeline = FinancePipeline()
    pipeline.connect()

    # Get all symbols that currently have price data
    print("\nFinding symbols to refetch...")
    df = pipeline.query("SELECT DISTINCT symbol FROM daily_prices")
    symbols = df['symbol'].tolist()

    if not symbols:
        print("[!] No symbols found in database")
        print("    Use the Data Fetcher in the GUI to add symbols first")
        return

    print(f"Found {len(symbols)} symbols: {', '.join(symbols)}")

    # Choose data range
    print("\nSelect data range:")
    print("  1. 1 year (default)")
    print("  2. 2 years")
    print("  3. 5 years")
    print("  4. Max available")

    range_choice = input("Choice (1-4): ").strip()
    period_map = {'1': '1y', '2': '2y', '3': '5y', '4': 'max'}
    period = period_map.get(range_choice, '1y')

    print(f"\nUsing '{period}' data range")

    # Confirm
    print("\n" + "=" * 60)
    print("This will:")
    print("  1. Delete existing price data for all symbols")
    print("  2. Refetch with adjusted prices from Yahoo Finance")
    print("  3. No API limits (Yahoo Finance is free)")
    print("=" * 60)

    response = input("\nContinue? (yes/no): ").strip().lower()
    if response not in ['yes', 'y']:
        print("Cancelled")
        return

    # Refetch all symbols
    print("\n" + "=" * 60)
    print("STARTING REFETCH")
    print("=" * 60)

    success_count = 0
    failed_symbols = []

    for i, symbol in enumerate(symbols, 1):
        print(f"\n[{i}/{len(symbols)}] {symbol}...")

        try:
            pipeline.clear_symbol_prices(symbol)
            pipeline.fetch_prices_yahoo(symbol, period=period)
            success_count += 1
            time.sleep(0.5)  # Be nice to Yahoo

        except Exception as e:
            print(f"    FAILED: {e}")
            failed_symbols.append((symbol, str(e)))

    # Summary
    print("\n" + "=" * 60)
    print("REFETCH COMPLETE!")
    print("=" * 60)
    print(f"Success: {success_count}/{len(symbols)}")
    print(f"Failed: {len(failed_symbols)}/{len(symbols)}")

    if success_count > 0:
        print("\nYour charts should now be smooth and match Yahoo Finance!")
        print("Refresh your Streamlit app to see the corrected data.")

    if failed_symbols:
        print("\nFailed symbols:")
        for sym, err in failed_symbols:
            print(f"  {sym}: {err}")

    pipeline.close()


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("\n\nInterrupted by user")
    except Exception as e:
        print(f"\n\nERROR: {e}")
        import traceback
        traceback.print_exc()
