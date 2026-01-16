#!/usr/bin/env python3
"""
Initialize Financial Pipeline Database
Door 865: Financial Data Pipeline - SQLite Edition
"""

import sys
import os

# Add src to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))

from src.pipeline import FinancePipeline


def main():
    """Initialize database with schema"""
    print("=" * 60)
    print("Financial Data Pipeline - Database Initialization")
    print("=" * 60)
    
    # Create pipeline instance
    pipeline = FinancePipeline()
    
    # Initialize schema
    print("\n1. Creating database schema...")
    pipeline.init_schema()
    
    # Create default watchlists from config
    print("\n2. Creating default watchlists...")
    watchlists = pipeline.config.get('watchlists', {})
    
    for name, symbols in watchlists.items():
        try:
            pipeline.create_watchlist(
                name=name,
                symbols=symbols,
                description=f"Default {name} watchlist"
            )
        except Exception as e:
            print(f"Watchlist '{name}' may already exist: {e}")
    
    # Verify setup
    print("\n3. Verifying setup...")
    tables = pipeline.query("""
        SELECT name FROM sqlite_master 
        WHERE type='table' 
        ORDER BY name
    """)
    
    print(f"\nCreated {len(tables)} tables:")
    for table in tables['name']:
        print(f"  - {table}")
    
    # Show database path
    print(f"\n✓ Database initialized at: {pipeline.db_path}")
    print(f"✓ Schema version: 1.0")
    print(f"✓ Ready for data loading")
    
    print("\n" + "=" * 60)
    print("Next steps:")
    print("  1. Configure API keys in config/config.json")
    print("  2. Run: python scripts/load_symbols.py")
    print("  3. Run: python scripts/fetch_prices.py --symbols AAPL,MSFT")
    print("=" * 60)


if __name__ == "__main__":
    main()
