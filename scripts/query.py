#!/usr/bin/env python3
"""
Query Examples - Financial Data Pipeline
Door 865: Financial Data Pipeline - SQLite Edition
"""

import sys
import os
import argparse

# Add src to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))

from src.pipeline import FinancePipeline


def query_sector_performance(pipeline: FinancePipeline, limit: int = 10):
    """Show sector performance"""
    print("\n" + "=" * 60)
    print("Sector Performance (Latest)")
    print("=" * 60)
    
    result = pipeline.get_sector_performance(limit)
    
    for _, row in result.iterrows():
        print(f"{row['sector']:30} ${row['avg_price']:>10.2f}  "
              f"({row['num_stocks']:>4} stocks, {row['total_volume']:>15,.0f} vol)")


def query_top_movers(pipeline: FinancePipeline, days: int = 30, limit: int = 10):
    """Show biggest price changes"""
    print("\n" + "=" * 60)
    print(f"Top Movers (Last {days} Days)")
    print("=" * 60)
    
    result = pipeline.query(f"""
        WITH price_change AS (
            SELECT 
                p1.symbol,
                s.name,
                s.sector,
                p1.close as current_price,
                p2.close as past_price,
                ((p1.close - p2.close) / p2.close * 100) as pct_change
            FROM daily_prices p1
            JOIN daily_prices p2 ON p1.symbol = p2.symbol
            JOIN symbols s ON p1.symbol = s.symbol
            WHERE p1.timestamp = (SELECT MAX(timestamp) FROM daily_prices)
              AND p2.timestamp = (SELECT MAX(timestamp) FROM daily_prices WHERE timestamp < date('now', '-{days} days'))
        )
        SELECT * FROM price_change
        ORDER BY pct_change DESC
        LIMIT {limit}
    """)
    
    for _, row in result.iterrows():
        change_str = f"{row['pct_change']:+.2f}%"
        print(f"{row['symbol']:6} {row['name'][:30]:30} {change_str:>10}  "
              f"(${row['past_price']:.2f} → ${row['current_price']:.2f})")


def query_watchlist_summary(pipeline: FinancePipeline, watchlist_name: str):
    """Show watchlist with latest prices"""
    print("\n" + "=" * 60)
    print(f"Watchlist: {watchlist_name}")
    print("=" * 60)
    
    result = pipeline.query("""
        SELECT 
            s.symbol,
            s.name,
            s.sector,
            p.close as last_price,
            p.volume as last_volume,
            p.timestamp as last_updated
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
    
    for _, row in result.iterrows():
        price = f"${row['last_price']:.2f}" if row['last_price'] else "N/A"
        print(f"{row['symbol']:6} {row['name'][:30]:30} {price:>10}  "
              f"{row['sector'] or 'N/A':20} {row['last_updated'] or 'N/A'}")


def query_macro_correlation(pipeline: FinancePipeline):
    """Show market vs macro indicators"""
    print("\n" + "=" * 60)
    print("Market Performance vs Macro Indicators")
    print("=" * 60)
    
    # Get average market price by date
    result = pipeline.query("""
        SELECT 
            date(p.timestamp) as date,
            AVG(p.close) as market_avg,
            (SELECT value FROM macro_data 
             WHERE indicator = 'GDP' 
             AND date <= p.timestamp 
             ORDER BY date DESC LIMIT 1) as gdp,
            (SELECT value FROM macro_data 
             WHERE indicator = 'UNRATE' 
             AND date <= p.timestamp 
             ORDER BY date DESC LIMIT 1) as unemployment
        FROM daily_prices p
        WHERE p.timestamp >= date('now', '-90 days')
        GROUP BY date(p.timestamp)
        ORDER BY date DESC
        LIMIT 20
    """)
    
    print(f"\n{'Date':12} {'Market Avg':>12} {'GDP':>12} {'Unemployment':>12}")
    print("-" * 60)
    for _, row in result.iterrows():
        gdp_str = f"{row['gdp']:.1f}" if row['gdp'] else "N/A"
        unemp_str = f"{row['unemployment']:.1f}%" if row['unemployment'] else "N/A"
        print(f"{row['date']:12} ${row['market_avg']:>11.2f} {gdp_str:>12} {unemp_str:>12}")


def query_custom(pipeline: FinancePipeline, sql: str):
    """Execute custom SQL query"""
    print("\n" + "=" * 60)
    print("Custom Query Results")
    print("=" * 60)
    
    result = pipeline.query(sql)
    print(result.to_string())


def main():
    parser = argparse.ArgumentParser(
        description='Query financial data with pre-built examples'
    )
    parser.add_argument(
        '--sector',
        help='Show sector performance',
        action='store_true'
    )
    parser.add_argument(
        '--movers',
        help='Show top price movers',
        action='store_true'
    )
    parser.add_argument(
        '--watchlist',
        help='Show watchlist summary (specify name)',
        default=None
    )
    parser.add_argument(
        '--macro-correlation',
        help='Show market vs macro indicators',
        action='store_true'
    )
    parser.add_argument(
        '--sql',
        help='Execute custom SQL query',
        default=None
    )
    parser.add_argument(
        '--limit',
        type=int,
        default=10,
        help='Result limit (default: 10)'
    )
    parser.add_argument(
        '--days',
        type=int,
        default=30,
        help='Days for price change calculation (default: 30)'
    )
    
    args = parser.parse_args()
    
    # Initialize pipeline
    pipeline = FinancePipeline()
    
    print("=" * 60)
    print("Financial Data Pipeline - Query Tool")
    print("=" * 60)
    
    # Execute queries based on args
    if args.sector:
        query_sector_performance(pipeline, args.limit)
    
    if args.movers:
        query_top_movers(pipeline, args.days, args.limit)
    
    if args.watchlist:
        query_watchlist_summary(pipeline, args.watchlist)
    
    if args.macro_correlation:
        query_macro_correlation(pipeline)
    
    if args.sql:
        query_custom(pipeline, args.sql)
    
    # If no specific query, show available options
    if not any([args.sector, args.movers, args.watchlist, 
                args.macro_correlation, args.sql]):
        print("\nAvailable queries:")
        print("  --sector              Sector performance summary")
        print("  --movers              Top price movers")
        print("  --watchlist NAME      Watchlist with latest prices")
        print("  --macro-correlation   Market vs macro indicators")
        print("  --sql 'QUERY'         Custom SQL query")
        print("\nExamples:")
        print("  python scripts/query.py --sector --limit 20")
        print("  python scripts/query.py --movers --days 7")
        print("  python scripts/query.py --watchlist default")
        print("  python scripts/query.py --sql 'SELECT * FROM symbols LIMIT 5'")
    
    print("\n" + "=" * 60)
    
    return 0


if __name__ == "__main__":
    sys.exit(main())
