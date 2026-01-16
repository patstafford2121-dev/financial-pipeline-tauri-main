#!/usr/bin/env python3
"""
Fetch Price Data using Yahoo Finance
Door 865: Financial Data Pipeline - SQLite Edition

Uses yfinance - FREE and UNLIMITED (no API key needed!)
"""

import sys
import os
import argparse
from typing import List

# Add src to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))

from src.pipeline import FinancePipeline


def load_watchlist(path: str) -> List[str]:
    """Load symbols from watchlist file (one per line)"""
    with open(path, 'r') as f:
        return [line.strip() for line in f if line.strip() and not line.startswith('#')]


def main():
    parser = argparse.ArgumentParser(
        description='Fetch price data using Yahoo Finance (FREE & UNLIMITED)'
    )
    parser.add_argument(
        '--symbols',
        help='Comma-separated list of symbols (e.g., AAPL,MSFT,GOOGL)',
        default=None
    )
    parser.add_argument(
        '--watchlist',
        help='Path to watchlist file (one symbol per line)',
        default=None
    )
    parser.add_argument(
        '--watchlist-name',
        help='Load symbols from named watchlist in database',
        default=None
    )
    parser.add_argument(
        '--period',
        choices=['1d', '5d', '1mo', '3mo', '6mo', '1y', '2y', '5y', '10y', 'ytd', 'max'],
        default='1y',
        help='Time period (default: 1y)'
    )
    parser.add_argument(
        '--refetch',
        action='store_true',
        help='Clear existing data before fetching (fresh download)'
    )

    args = parser.parse_args()

    print("=" * 60)
    print("Financial Data Pipeline - Yahoo Finance Fetcher")
    print("FREE & UNLIMITED - No API key required!")
    print("=" * 60)

    # Initialize pipeline
    pipeline = FinancePipeline()

    # Get symbols to fetch
    symbols = []

    if args.symbols:
        symbols = [s.strip().upper() for s in args.symbols.split(',')]
    elif args.watchlist:
        symbols = load_watchlist(args.watchlist)
        print(f"\nLoaded {len(symbols)} symbols from {args.watchlist}")
    elif args.watchlist_name:
        symbols = pipeline.get_watchlist(args.watchlist_name)
        print(f"\nLoaded {len(symbols)} symbols from watchlist '{args.watchlist_name}'")
    else:
        # Use default watchlist from config
        default = pipeline.config.get('watchlists', {}).get('default', [])
        if default:
            symbols = default
            print(f"\nUsing default watchlist ({len(symbols)} symbols)")
        else:
            print("\nError: No symbols specified")
            print("Use --symbols, --watchlist, or --watchlist-name")
            return 1

    if not symbols:
        print("No symbols to fetch")
        return 1

    print(f"\nFetching {args.period} data for {len(symbols)} symbols...")
    print(f"Source: Yahoo Finance (yfinance)")
    print("-" * 60)

    # Fetch data
    if args.refetch:
        print("\n[REFETCH MODE] Clearing existing data before fetching...")
        pipeline.refetch_all_symbols_yahoo(symbols, period=args.period)
    else:
        pipeline.fetch_prices_batch_yahoo(symbols, period=args.period)

    # Show data summary
    total_records = pipeline.query("""
        SELECT COUNT(*) as count FROM daily_prices
    """)['count'].iloc[0]

    latest_date = pipeline.query("""
        SELECT MAX(timestamp) as latest FROM daily_prices
    """)['latest'].iloc[0]

    print("\n" + "=" * 60)
    print("Database Summary:")
    print(f"  Total price records: {total_records:,}")
    print(f"  Latest data: {latest_date}")

    print("\n" + "=" * 60)
    print("Next steps:")
    print("  python scripts/query.py --sector Technology")
    print("  python scripts/fetch_macro.py")
    print("=" * 60)

    return 0


if __name__ == "__main__":
    sys.exit(main())
