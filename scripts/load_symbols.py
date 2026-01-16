#!/usr/bin/env python3
"""
Load Symbol Catalog from FinanceDatabase
Door 865: Financial Data Pipeline - SQLite Edition
"""

import sys
import os
import argparse
import json
from pathlib import Path

# Add src to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))

from src.pipeline import FinancePipeline


def download_finance_database():
    """Clone or update FinanceDatabase repository"""
    import subprocess
    
    repo_url = "https://github.com/JerBouma/FinanceDatabase"
    local_path = "data/raw/FinanceDatabase"
    
    if os.path.exists(local_path):
        print(f"Updating FinanceDatabase at {local_path}...")
        subprocess.run(["git", "pull"], cwd=local_path, check=True)
    else:
        print(f"Cloning FinanceDatabase to {local_path}...")
        os.makedirs("data/raw", exist_ok=True)
        subprocess.run(["git", "clone", repo_url, local_path], check=True)
    
    return local_path


def find_json_files(base_path: str) -> dict:
    """Find all JSON files in FinanceDatabase structure"""
    files = {}
    
    # FinanceDatabase structure
    categories = {
        'equities': 'equity',
        'etfs': 'etf',
        'cryptocurrencies': 'crypto',
        'currencies': 'forex',
        'indices': 'index',
        'moneymarkets': 'money_market',
        'commodities': 'commodity'
    }
    
    for category, asset_class in categories.items():
        json_path = os.path.join(base_path, category + '.json')
        if os.path.exists(json_path):
            files[asset_class] = json_path
    
    return files


def main():
    parser = argparse.ArgumentParser(
        description='Load symbol catalog into database'
    )
    parser.add_argument(
        '--path',
        help='Path to FinanceDatabase JSON file or directory',
        default=None
    )
    parser.add_argument(
        '--download',
        action='store_true',
        help='Download/update FinanceDatabase from GitHub'
    )
    parser.add_argument(
        '--asset-class',
        help='Asset class: equity, etf, crypto, forex, index',
        default=None
    )
    parser.add_argument(
        '--all',
        action='store_true',
        help='Load all available asset classes'
    )
    
    args = parser.parse_args()
    
    print("=" * 60)
    print("Financial Data Pipeline - Symbol Catalog Loader")
    print("=" * 60)
    
    # Initialize pipeline
    pipeline = FinancePipeline()
    
    # Download if requested
    if args.download:
        base_path = download_finance_database()
    elif args.path:
        base_path = args.path
    else:
        # Try default location
        base_path = "data/raw/FinanceDatabase"
        if not os.path.exists(base_path):
            print("\nError: FinanceDatabase not found")
            print("Options:")
            print("  1. Run with --download to clone repository")
            print("  2. Specify --path to existing JSON file/directory")
            return 1
    
    # Find JSON files
    if os.path.isfile(base_path):
        # Single file
        files = {args.asset_class or 'equity': base_path}
    else:
        # Directory - find all JSON files
        files = find_json_files(base_path)
    
    if not files:
        print(f"\nNo JSON files found in {base_path}")
        return 1
    
    # Load symbols
    total_loaded = 0
    
    if args.all:
        # Load all available
        for asset_class, json_path in files.items():
            print(f"\nLoading {asset_class} from {json_path}...")
            try:
                count = pipeline.load_symbols(json_path, asset_class)
                total_loaded += count
            except Exception as e:
                print(f"Error loading {asset_class}: {e}")
    else:
        # Load specific asset class
        asset_class = args.asset_class or 'equity'
        if asset_class not in files:
            print(f"\nAsset class '{asset_class}' not found")
            print(f"Available: {', '.join(files.keys())}")
            return 1
        
        json_path = files[asset_class]
        print(f"\nLoading {asset_class} from {json_path}...")
        count = pipeline.load_symbols(json_path, asset_class)
        total_loaded = count
    
    # Show summary
    print("\n" + "=" * 60)
    print(f"✓ Loaded {total_loaded:,} symbols")
    
    # Show breakdown by asset class
    summary = pipeline.query("""
        SELECT asset_class, COUNT(*) as count
        FROM symbols
        GROUP BY asset_class
        ORDER BY count DESC
    """)
    
    print("\nBreakdown by asset class:")
    for _, row in summary.iterrows():
        print(f"  {row['asset_class']:15} {row['count']:>8,} symbols")
    
    # Show top sectors (for equities)
    sectors = pipeline.query("""
        SELECT sector, COUNT(*) as count
        FROM symbols
        WHERE asset_class = 'equity' AND sector IS NOT NULL
        GROUP BY sector
        ORDER BY count DESC
        LIMIT 10
    """)
    
    if len(sectors) > 0:
        print("\nTop 10 sectors:")
        for _, row in sectors.iterrows():
            print(f"  {row['sector']:30} {row['count']:>6,} stocks")
    
    print("\n" + "=" * 60)
    print("Next steps:")
    print("  python scripts/fetch_prices.py --symbols AAPL,MSFT,GOOGL")
    print("=" * 60)
    
    return 0


if __name__ == "__main__":
    sys.exit(main())
