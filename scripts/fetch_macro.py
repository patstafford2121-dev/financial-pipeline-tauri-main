#!/usr/bin/env python3
"""
Fetch Macro Economic Data from FRED
Door 865: Financial Data Pipeline - SQLite Edition
"""

import sys
import os
import argparse

# Add src to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))

from src.pipeline import FinancePipeline


DEFAULT_INDICATORS = [
    'GDP',        # Gross Domestic Product
    'UNRATE',     # Unemployment Rate
    'DFF',        # Federal Funds Rate
    'CPIAUCSL',   # Consumer Price Index
    'PCE',        # Personal Consumption Expenditures
    'FEDFUNDS',   # Effective Federal Funds Rate
    'DEXUSEU',    # USD/EUR Exchange Rate
    'DTWEXBGS',   # Trade Weighted USD Index
    'T10Y2Y',     # 10-Year Treasury Minus 2-Year
    'VIXCLS',     # VIX Volatility Index
]


def main():
    parser = argparse.ArgumentParser(
        description='Fetch macro economic data from FRED'
    )
    parser.add_argument(
        '--indicators',
        help='Comma-separated FRED series IDs (default: common indicators)',
        default=None
    )
    parser.add_argument(
        '--all',
        action='store_true',
        help='Fetch all indicators from config'
    )
    parser.add_argument(
        '--list',
        action='store_true',
        help='List available indicators and exit'
    )
    
    args = parser.parse_args()
    
    # Initialize pipeline
    pipeline = FinancePipeline()
    
    if args.list:
        print("=" * 60)
        print("Common FRED Economic Indicators")
        print("=" * 60)
        print("\nMacro:")
        print("  GDP       - Gross Domestic Product")
        print("  UNRATE    - Unemployment Rate")
        print("  CPIAUCSL  - Consumer Price Index")
        print("  PCE       - Personal Consumption Expenditures")
        print("\nMonetary Policy:")
        print("  DFF       - Federal Funds Rate")
        print("  FEDFUNDS  - Effective Federal Funds Rate")
        print("  T10Y2Y    - 10Y-2Y Treasury Spread (recession indicator)")
        print("\nMarkets:")
        print("  VIXCLS    - VIX Volatility Index")
        print("  DEXUSEU   - USD/EUR Exchange Rate")
        print("  DTWEXBGS  - Trade Weighted USD Index")
        print("\nMore at: https://fred.stlouisfed.org/")
        print("=" * 60)
        return 0
    
    print("=" * 60)
    print("Financial Data Pipeline - FRED Data Fetcher")
    print("=" * 60)
    
    # Get indicators to fetch
    if args.indicators:
        indicators = [i.strip() for i in args.indicators.split(',')]
    elif args.all:
        indicators = pipeline.config.get('data_sources', {}).get('fred', {}).get('indicators', DEFAULT_INDICATORS)
    else:
        indicators = DEFAULT_INDICATORS
    
    print(f"\nFetching {len(indicators)} indicators from FRED...")
    
    # Fetch data
    results = []
    for indicator in indicators:
        try:
            print(f"\n{indicator}...", end=' ')
            count = pipeline.fetch_fred(indicator)
            results.append({'indicator': indicator, 'count': count, 'success': True})
            print(f"✓ {count} records")
        except Exception as e:
            print(f"✗ Error: {e}")
            results.append({'indicator': indicator, 'error': str(e), 'success': False})
    
    # Show summary
    print("\n" + "=" * 60)
    successful = sum(1 for r in results if r['success'])
    total_records = sum(r.get('count', 0) for r in results if r['success'])
    
    print(f"✓ Successfully loaded: {successful}/{len(results)} indicators")
    print(f"✓ Total records: {total_records:,}")
    
    failed = [r for r in results if not r['success']]
    if failed:
        print(f"\n✗ Failed: {len(failed)} indicators")
        for r in failed:
            print(f"  {r['indicator']}: {r['error']}")
    
    # Show what's in database
    summary = pipeline.query("""
        SELECT 
            indicator,
            frequency,
            COUNT(*) as records,
            MIN(date) as earliest,
            MAX(date) as latest
        FROM macro_data
        GROUP BY indicator, frequency
        ORDER BY indicator
    """)
    
    print("\nMacro data in database:")
    for _, row in summary.iterrows():
        print(f"  {row['indicator']:12} ({row['frequency']:9}) "
              f"{row['records']:>6,} records  "
              f"{row['earliest']} to {row['latest']}")
    
    print("\n" + "=" * 60)
    print("Next steps:")
    print("  # Query macro data")
    print("  python -c \"from src.pipeline import FinancePipeline; \\")
    print("    p = FinancePipeline(); \\")
    print("    print(p.query('SELECT * FROM macro_data WHERE indicator=\\\"GDP\\\" ORDER BY date DESC LIMIT 5'))\"")
    print("\n  # Correlate with market data")
    print("  python scripts/query.py --macro-correlation")
    print("=" * 60)
    
    return 0


if __name__ == "__main__":
    sys.exit(main())
