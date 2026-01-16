#!/usr/bin/env python3
"""
Example Workflow - Financial Data Pipeline
Door 865: Financial Data Pipeline - SQLite Edition

This script demonstrates a complete analysis workflow:
1. Load data
2. Query sector performance
3. Find correlations with macro indicators
4. Export results
"""

import sys
import os

# Add src to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))

from src.pipeline import FinancePipeline
import pandas as pd


def example_sector_analysis():
    """Analyze sector performance and export to CSV"""
    print("\n" + "=" * 60)
    print("Example 1: Sector Performance Analysis")
    print("=" * 60)
    
    pipeline = FinancePipeline()
    
    # Get sector performance
    sectors = pipeline.query("""
        SELECT 
            s.sector,
            COUNT(DISTINCT s.symbol) as num_stocks,
            AVG(p.close) as avg_price,
            MIN(p.close) as min_price,
            MAX(p.close) as max_price,
            SUM(p.volume) as total_volume
        FROM symbols s
        JOIN daily_prices p ON s.symbol = p.symbol
        WHERE p.timestamp >= date('now', '-30 days')
          AND s.sector IS NOT NULL
        GROUP BY s.sector
        ORDER BY avg_price DESC
    """)
    
    print(f"\nFound {len(sectors)} sectors")
    print("\nTop 5 by average price:")
    print(sectors.head().to_string(index=False))
    
    # Export to CSV
    output_path = "data/sector_analysis.csv"
    sectors.to_csv(output_path, index=False)
    print(f"\n✓ Exported to {output_path}")


def example_price_momentum():
    """Find stocks with strong recent momentum"""
    print("\n" + "=" * 60)
    print("Example 2: Price Momentum Analysis")
    print("=" * 60)
    
    pipeline = FinancePipeline()
    
    # Calculate 30-day momentum
    momentum = pipeline.query("""
        WITH price_comparison AS (
            SELECT 
                p1.symbol,
                s.name,
                s.sector,
                p1.close as current_price,
                p2.close as price_30d_ago,
                ((p1.close - p2.close) / p2.close * 100) as pct_change,
                p1.volume as current_volume
            FROM daily_prices p1
            JOIN daily_prices p2 ON p1.symbol = p2.symbol
            JOIN symbols s ON p1.symbol = s.symbol
            WHERE p1.timestamp = (SELECT MAX(timestamp) FROM daily_prices)
              AND p2.timestamp = (
                  SELECT MAX(timestamp) 
                  FROM daily_prices 
                  WHERE timestamp <= date('now', '-30 days')
              )
        )
        SELECT * FROM price_comparison
        WHERE pct_change IS NOT NULL
        ORDER BY pct_change DESC
        LIMIT 20
    """)
    
    print(f"\nTop 20 gainers (last 30 days):")
    for _, row in momentum.head(10).iterrows():
        print(f"{row['symbol']:6} {row['name'][:30]:30} "
              f"{row['pct_change']:+7.2f}% "
              f"(${row['price_30d_ago']:.2f} → ${row['current_price']:.2f})")


def example_macro_correlation():
    """Correlate market performance with unemployment rate"""
    print("\n" + "=" * 60)
    print("Example 3: Market vs Unemployment Correlation")
    print("=" * 60)
    
    pipeline = FinancePipeline()
    
    # Get market average and unemployment by month
    correlation = pipeline.query("""
        SELECT 
            strftime('%Y-%m', p.timestamp) as month,
            AVG(p.close) as market_avg,
            (SELECT value 
             FROM macro_data 
             WHERE indicator = 'UNRATE' 
             AND date <= p.timestamp 
             ORDER BY date DESC 
             LIMIT 1) as unemployment_rate
        FROM daily_prices p
        WHERE p.timestamp >= date('now', '-365 days')
        GROUP BY strftime('%Y-%m', p.timestamp)
        HAVING unemployment_rate IS NOT NULL
        ORDER BY month DESC
    """)
    
    if len(correlation) > 0:
        print(f"\nMonthly averages (last 12 months):")
        print(f"\n{'Month':10} {'Market Avg':>12} {'Unemployment':>12}")
        print("-" * 40)
        
        for _, row in correlation.head(12).iterrows():
            print(f"{row['month']:10} ${row['market_avg']:>11.2f} {row['unemployment_rate']:>11.1f}%")
        
        # Simple correlation
        if len(correlation) >= 2:
            corr = correlation[['market_avg', 'unemployment_rate']].corr().iloc[0, 1]
            print(f"\nCorrelation: {corr:.3f}")
            if abs(corr) > 0.5:
                direction = "negative" if corr < 0 else "positive"
                print(f"Strong {direction} correlation detected")
    else:
        print("No macro data available. Run: python scripts/fetch_macro.py")


def example_watchlist_tracking():
    """Track a custom watchlist"""
    print("\n" + "=" * 60)
    print("Example 4: Custom Watchlist Tracking")
    print("=" * 60)
    
    pipeline = FinancePipeline()
    
    # Create or use existing watchlist
    watchlist_name = "tech_giants"
    symbols = ['AAPL', 'MSFT', 'GOOGL', 'AMZN', 'META']
    
    try:
        # Try to create (will fail if exists)
        pipeline.create_watchlist(watchlist_name, symbols, "Tech giants tracking")
    except:
        print(f"Using existing watchlist: {watchlist_name}")
    
    # Get current status
    status = pipeline.query("""
        SELECT 
            s.symbol,
            s.name,
            p.close as last_price,
            p.volume as last_volume,
            p.timestamp as last_update
        FROM watchlist_symbols ws
        JOIN watchlists w ON ws.watchlist_id = w.id
        JOIN symbols s ON ws.symbol = s.symbol
        LEFT JOIN (
            SELECT symbol, close, volume, timestamp
            FROM daily_prices
            WHERE (symbol, timestamp) IN (
                SELECT symbol, MAX(timestamp)
                FROM daily_prices
                GROUP BY symbol
            )
        ) p ON s.symbol = p.symbol
        WHERE w.name = ?
        ORDER BY s.symbol
    """, (watchlist_name,))
    
    print(f"\nWatchlist: {watchlist_name}")
    print(f"{'Symbol':6} {'Name':30} {'Price':>10} {'Volume':>15}")
    print("-" * 70)
    
    for _, row in status.iterrows():
        price = f"${row['last_price']:.2f}" if row['last_price'] else "N/A"
        volume = f"{row['last_volume']:,.0f}" if row['last_volume'] else "N/A"
        print(f"{row['symbol']:6} {row['name'][:30]:30} {price:>10} {volume:>15}")


def example_database_stats():
    """Show database statistics"""
    print("\n" + "=" * 60)
    print("Example 5: Database Statistics")
    print("=" * 60)
    
    pipeline = FinancePipeline()
    
    # Symbol count by asset class
    symbols = pipeline.query("""
        SELECT asset_class, COUNT(*) as count
        FROM symbols
        GROUP BY asset_class
    """)
    print("\nSymbols by asset class:")
    for _, row in symbols.iterrows():
        print(f"  {row['asset_class']:15} {row['count']:>8,}")
    
    # Price data summary
    prices = pipeline.query("""
        SELECT 
            COUNT(*) as total_records,
            COUNT(DISTINCT symbol) as symbols_with_data,
            MIN(timestamp) as earliest_date,
            MAX(timestamp) as latest_date
        FROM daily_prices
    """)
    
    if len(prices) > 0 and prices['total_records'].iloc[0] > 0:
        p = prices.iloc[0]
        print(f"\nPrice data:")
        print(f"  Total records: {p['total_records']:,}")
        print(f"  Symbols: {p['symbols_with_data']:,}")
        print(f"  Date range: {p['earliest_date']} to {p['latest_date']}")
    
    # Macro data summary
    macro = pipeline.query("""
        SELECT indicator, COUNT(*) as count
        FROM macro_data
        GROUP BY indicator
    """)
    
    if len(macro) > 0:
        print(f"\nMacro indicators ({len(macro)} total):")
        for _, row in macro.head(5).iterrows():
            print(f"  {row['indicator']:10} {row['count']:>6,} records")


def main():
    """Run all examples"""
    print("=" * 60)
    print("Financial Data Pipeline - Example Workflows")
    print("Door 865: Financial Data Pipeline - SQLite Edition")
    print("=" * 60)
    
    # Check if database exists
    if not os.path.exists("data/finance.db"):
        print("\n❌ Database not found!")
        print("Run setup first: python setup.py")
        return 1
    
    try:
        # Run examples
        example_database_stats()
        example_sector_analysis()
        example_price_momentum()
        example_watchlist_tracking()
        example_macro_correlation()
        
        print("\n" + "=" * 60)
        print("Examples complete!")
        print("\nNext steps:")
        print("  - Modify these examples for your analysis")
        print("  - See docs/QUERIES.md for more query patterns")
        print("  - Build backtesting strategies")
        print("=" * 60)
        
    except Exception as e:
        print(f"\n❌ Error: {e}")
        print("\nMake sure you have:")
        print("  1. Loaded symbols: python scripts/load_symbols.py")
        print("  2. Fetched prices: python scripts/fetch_prices.py")
        print("  3. Loaded macro data: python scripts/fetch_macro.py")
        return 1
    
    return 0


if __name__ == "__main__":
    sys.exit(main())
