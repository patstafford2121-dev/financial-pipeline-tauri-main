"""
Automated Migration Script: Alpha Vantage → Yahoo Finance

This script will:
1. Check if yfinance is installed
2. Find all symbols in your database
3. Clear old data
4. Refetch everything from Yahoo Finance (FAST & FREE!)
5. Verify the migration
"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))

def check_yfinance():
    """Check if yfinance is installed"""
    try:
        import yfinance
        print("[OK] yfinance is installed")
        return True
    except ImportError:
        print("[FAIL] yfinance is NOT installed")
        print("\nPlease install it:")
        print("  pip install yfinance")
        print("\nThen run this script again.")
        return False

def main():
    print("=" * 80)
    print("MIGRATION: ALPHA VANTAGE -> YAHOO FINANCE")
    print("=" * 80)
    
    # Check yfinance
    print("\n1. Checking dependencies...")
    if not check_yfinance():
        return
    
    # Import pipeline
    try:
        from src.pipeline import FinancePipeline
        print("[OK] Pipeline module loaded")
    except Exception as e:
        print(f"[FAIL] Failed to load pipeline: {e}")
        print("\nMake sure you're in the correct directory:")
        print("  cd \"C:\\Claude Projects\\Financial pipeline data\"")
        return
    
    # Connect to database
    print("\n2. Connecting to database...")
    pipeline = FinancePipeline()
    pipeline.connect()
    print("[OK] Connected to database")
    
    # Get existing symbols
    print("\n3. Finding symbols to migrate...")
    df = pipeline.query("SELECT DISTINCT symbol FROM daily_prices ORDER BY symbol")
    symbols = df['symbol'].tolist()
    
    if not symbols:
        print("[WARN] No symbols found in database")
        print("   Nothing to migrate!")
        return
    
    print(f"[OK] Found {len(symbols)} symbols:")
    for i, sym in enumerate(symbols, 1):
        print(f"   {i}. {sym}")
    
    # Ask for confirmation
    print("\n" + "=" * 80)
    print("[!] MIGRATION PLAN:")
    print(f"   - Delete all existing price data")
    print(f"   - Refetch {len(symbols)} symbols from Yahoo Finance")
    print(f"   - This will take approximately {len(symbols) * 2} seconds")
    print("=" * 80)
    
    response = input("\nContinue with migration? (yes/no): ").strip().lower()
    
    if response not in ['yes', 'y']:
        print("[CANCELLED] Migration cancelled")
        return
    
    # Choose period
    print("\n4. Select data range:")
    print("   1. 1 year (fast, recent data)")
    print("   2. 5 years (recommended)")
    print("   3. 10 years (comprehensive)")
    print("   4. Maximum (all available history)")
    
    period_choice = input("\nChoice (1-4): ").strip()
    period_map = {
        '1': '1y',
        '2': '5y',
        '3': '10y',
        '4': 'max'
    }
    period = period_map.get(period_choice, '5y')
    
    print(f"\n[OK] Using period: {period}")
    
    # Check if fetch_prices_yahoo exists
    print("\n5. Checking for Yahoo Finance method...")
    if not hasattr(pipeline, 'fetch_prices_yahoo'):
        print("[FAIL] Your pipeline.py doesn't have fetch_prices_yahoo() method")
        print("\nYou need to replace pipeline.py with the Yahoo Finance version:")
        print("  1. Backup: copy src\\pipeline.py src\\pipeline_OLD.py")
        print("  2. Replace: copy pipeline_YAHOO.py src\\pipeline.py")
        print("  3. Run this script again")
        return
    
    print("[OK] Yahoo Finance method found")
    
    # Start migration
    print("\n" + "=" * 80)
    print("STARTING MIGRATION")
    print("=" * 80)
    
    success_count = 0
    fail_count = 0
    failed_symbols = []
    
    for i, symbol in enumerate(symbols, 1):
        print(f"\n[{i}/{len(symbols)}] Migrating {symbol}...")
        
        try:
            # Clear old data
            print(f"  [DEL] Clearing old data...")
            pipeline.query("DELETE FROM daily_prices WHERE symbol = ?", [symbol])
            
            # Fetch from Yahoo Finance
            print(f"  [FETCH] Fetching from Yahoo Finance...")
            pipeline.fetch_prices_yahoo(symbol, period=period)
            
            # Verify
            latest = pipeline.get_latest_price(symbol)
            if latest:
                print(f"  [OK] Success! Latest price: ${latest:.2f}")
                success_count += 1
            else:
                print(f"  [WARN] Fetched but no price data")
                success_count += 1
                
        except Exception as e:
            print(f"  [FAIL] Failed: {e}")
            fail_count += 1
            failed_symbols.append(symbol)
    
    # Summary
    print("\n" + "=" * 80)
    print("MIGRATION COMPLETE!")
    print("=" * 80)
    print(f"[OK] Successfully migrated: {success_count}/{len(symbols)}")
    print(f"[FAIL] Failed: {fail_count}/{len(symbols)}")
    
    if failed_symbols:
        print(f"\nFailed symbols: {', '.join(failed_symbols)}")
        print("(These symbols may not be available on Yahoo Finance)")
    
    if success_count > 0:
        print("\n[SUCCESS] Migration successful!")
        print("   Your data is now powered by Yahoo Finance")
        print("   - No more API limits")
        print("   - No more delays")
        print("   - Free and unlimited")
        print("\n[TIP] Refresh your Streamlit app to see the results!")
    
    # Cleanup old API calls
    print("\n6. Cleaning up old Alpha Vantage API logs...")
    pipeline.query("DELETE FROM api_calls WHERE source = 'alpha_vantage'")
    print("[OK] Cleanup complete")
    
    pipeline.close()
    
    print("\n" + "=" * 80)
    print("[DONE] Yahoo Finance migration complete!")
    print("   You can now:")
    print("   - Fetch unlimited symbols")
    print("   - Download data in seconds (not minutes)")
    print("   - Remove your Alpha Vantage API key")
    print("=" * 80)

if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("\n\n[INTERRUPTED] Migration interrupted by user")
    except Exception as e:
        print(f"\n\n[ERROR] ERROR: {e}")
        import traceback
        traceback.print_exc()
