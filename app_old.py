"""
Financial Data Pipeline - Web GUI
Door 865: Financial Data Pipeline - SQLite Edition

Launch with: streamlit run app.py
"""

import streamlit as st
import pandas as pd
import plotly.express as px
import plotly.graph_objects as go
from datetime import datetime, timedelta
import sys
from pathlib import Path

# Add src to path
sys.path.insert(0, str(Path(__file__).parent))

from src.pipeline import FinancePipeline

# Page config
st.set_page_config(
    page_title="Financial Data Pipeline",
    page_icon="📊",
    layout="wide",
    initial_sidebar_state="expanded"
)

# Initialize pipeline in session state
if 'pipeline' not in st.session_state:
    try:
        st.session_state.pipeline = FinancePipeline()
        st.session_state.pipeline.connect()
    except Exception as e:
        st.error(f"Failed to initialize pipeline: {e}")
        st.stop()

pipeline = st.session_state.pipeline

# Sidebar navigation
st.sidebar.title("📊 Finance Pipeline")
st.sidebar.markdown("---")

page = st.sidebar.radio(
    "Navigation",
    [
        "🏠 Dashboard",
        "🔍 Symbol Browser",
        "📥 Data Fetcher",
        "📊 Query & Analysis",
        "⭐ Watchlists",
        "⚙️ Settings"
    ]
)

st.sidebar.markdown("---")
st.sidebar.caption("Door 865 - PhiSHRI")

# === DASHBOARD PAGE ===
if page == "🏠 Dashboard":
    st.title("📊 Financial Data Pipeline Dashboard")
    st.markdown("**Local-first financial data system** - Load once, query forever")
    
    # Metrics row
    col1, col2, col3, col4 = st.columns(4)
    
    with col1:
        # Count symbols
        symbol_count = pipeline.query("SELECT COUNT(*) as count FROM symbols")
        count = symbol_count['count'].iloc[0] if not symbol_count.empty else 0
        st.metric("Total Symbols", f"{count:,}")
    
    with col2:
        # Count price records
        price_count = pipeline.query("SELECT COUNT(*) as count FROM daily_prices")
        count = price_count['count'].iloc[0] if not price_count.empty else 0
        st.metric("Price Records", f"{count:,}")
    
    with col3:
        # API calls today
        api_usage = pipeline.get_api_usage('alpha_vantage', hours=24)
        st.metric("API Calls (24h)", f"{api_usage}/25")
    
    with col4:
        # Latest update
        latest = pipeline.query("""
            SELECT MAX(timestamp) as latest 
            FROM daily_prices
        """)
        if not latest.empty and not pd.isna(latest['latest'].iloc[0]):
            latest_date = latest['latest'].iloc[0]
            st.metric("Latest Data", str(latest_date)[:10])
        else:
            st.metric("Latest Data", "No data")
    
    st.markdown("---")
    
    # Two column layout
    col_left, col_right = st.columns([1, 1])
    
    with col_left:
        st.subheader("📈 Recent Price Updates")
        
        recent_prices = pipeline.query("""
            SELECT s.symbol, s.name, p.close, p.timestamp
            FROM daily_prices p
            JOIN symbols s ON p.symbol = s.symbol
            ORDER BY p.timestamp DESC
            LIMIT 10
        """)
        
        if not recent_prices.empty:
            df = recent_prices.copy()
            df['close'] = df['close'].apply(lambda x: f"${x:.2f}")
            st.dataframe(df, use_container_width=True, hide_index=True)
        else:
            st.info("No price data loaded yet. Use Data Fetcher to load prices.")
    
    with col_right:
        st.subheader("🏢 Sector Breakdown")
        
        sectors = pipeline.query("""
            SELECT sector, COUNT(*) as count
            FROM symbols
            WHERE sector IS NOT NULL AND sector != ''
            GROUP BY sector
            ORDER BY count DESC
            LIMIT 10
        """)
        
        if not sectors.empty:
            df = sectors.copy()
            fig = px.bar(df, x='sector', y='count', 
                        title="Top Sectors by Symbol Count")
            st.plotly_chart(fig, use_container_width=True)
        else:
            st.info("No sector data available. Load symbols first.")
    
    # Database info
    st.markdown("---")
    st.subheader("💾 Database Status")
    
    col1, col2, col3 = st.columns(3)
    
    with col1:
        db_size = Path(pipeline.db_path).stat().st_size / (1024 * 1024)
        st.info(f"**Database Size:** {db_size:.2f} MB")
    
    with col2:
        st.info(f"**Location:** `{pipeline.db_path}`")
    
    with col3:
        if st.button("🔄 Backup Database"):
            try:
                backup_path = pipeline.backup()
                st.success(f"Backup created: {backup_path}")
            except Exception as e:
                st.error(f"Backup failed: {e}")

# === SYMBOL BROWSER PAGE ===
elif page == "🔍 Symbol Browser":
    st.title("🔍 Symbol Browser")
    st.markdown("Browse and search 300k+ symbols from FinanceDatabase")
    
    # Search controls
    col1, col2, col3 = st.columns([2, 1, 1])
    
    with col1:
        search_term = st.text_input("🔎 Search symbols", placeholder="e.g., Apple, AAPL, Technology")
    
    with col2:
        asset_class = st.selectbox("Asset Class", 
                                   ["All", "equity", "etf", "crypto", "forex", "fund"])
    
    with col3:
        limit = st.number_input("Results", min_value=10, max_value=1000, value=100)
    
    # Build query
    query = "SELECT * FROM symbols WHERE 1=1"
    params = []
    
    if search_term:
        query += " AND (symbol LIKE ? OR name LIKE ? OR sector LIKE ?)"
        search_pattern = f"%{search_term}%"
        params.extend([search_pattern, search_pattern, search_pattern])
    
    if asset_class != "All":
        query += " AND asset_class = ?"
        params.append(asset_class)
    
    query += f" LIMIT {limit}"
    
    # Execute search
    results = pipeline.query(query, params) if params else pipeline.query(query)
    
    st.markdown(f"**Found {len(results)} symbols**")
    
    if not results.empty:
        df = results.copy()
        
        # Display options
        available_cols = [col for col in df.columns if col in ['symbol', 'name', 'sector', 'industry', 'market_cap', 'asset_class', 'exchange', 'country']]
        default_cols = [col for col in ['symbol', 'name', 'sector', 'industry', 'market_cap'] if col in available_cols]
        
        display_cols = st.multiselect(
            "Display Columns",
            available_cols,
            default=default_cols
        )
        
        if display_cols:
            st.dataframe(df[display_cols], use_container_width=True, height=600)
        
        # Export option
        st.download_button(
            "📥 Download as CSV",
            df.to_csv(index=False),
            "symbols.csv",
            "text/csv"
        )

# === DATA FETCHER PAGE ===
elif page == "📥 Data Fetcher":
    st.title("📥 Data Fetcher")
    st.markdown("Fetch price data, fundamentals, and macro indicators")
    
    # Tab layout
    tab1, tab2, tab3 = st.tabs(["📈 Prices", "📊 Fundamentals", "🌍 Macro Data"])
    
    # === PRICE FETCHER ===
    with tab1:
        st.subheader("📈 Fetch Price Data")
        
        col1, col2 = st.columns([2, 1])
        
        with col1:
            symbols_input = st.text_area(
                "Symbols (comma-separated)",
                placeholder="AAPL, MSFT, GOOGL",
                height=100
            )
        
        with col2:
            source = st.selectbox("Data Source", ["alpha_vantage", "finnhub"])
            outputsize = st.selectbox("Time Range", ["compact", "full"])
            
            st.caption("**Compact:** Last 100 days")
            st.caption("**Full:** 20+ years")
        
        # API usage warning
        if symbols_input:
            symbol_list = [s.strip() for s in symbols_input.split(',') if s.strip()]
            st.warning(f"⚠️ This will use {len(symbol_list)} API calls")
            
            usage = pipeline.get_api_usage('alpha_vantage', hours=24)
            remaining = 25 - usage
            st.info(f"Remaining today: {remaining}/25 calls")
        
        if st.button("🚀 Fetch Prices", type="primary"):
            if not symbols_input:
                st.error("Please enter symbols")
            else:
                symbol_list = [s.strip() for s in symbols_input.split(',') if s.strip()]
                
                progress_bar = st.progress(0)
                status_text = st.empty()
                
                for i, symbol in enumerate(symbol_list):
                    status_text.text(f"Fetching {symbol}... ({i+1}/{len(symbol_list)})")
                    
                    try:
                        if source == "alpha_vantage":
                            pipeline.fetch_prices_alpha(symbol, outputsize=outputsize)
                        else:
                            pipeline.fetch_prices_finnhub(symbol)
                        
                        st.success(f"✅ {symbol} loaded successfully")
                    except Exception as e:
                        st.error(f"❌ {symbol} failed: {e}")
                    
                    progress_bar.progress((i + 1) / len(symbol_list))
                
                status_text.text("✅ Complete!")
                st.balloons()
    
    # === FUNDAMENTALS FETCHER ===
    with tab2:
        st.subheader("📊 Fetch Fundamentals")
        st.info("🚧 Fundamentals fetching coming soon...")
        st.markdown("""
        **Planned features:**
        - Income statements
        - Balance sheets
        - Cash flow statements
        - Financial ratios
        - Analyst estimates
        """)
    
    # === MACRO DATA FETCHER ===
    with tab3:
        st.subheader("🌍 Fetch Macro Indicators (FRED)")
        
        col1, col2 = st.columns([2, 1])
        
        with col1:
            indicator = st.selectbox(
                "Select Indicator",
                ["GDP", "UNRATE", "DFF", "CPIAUCSL", "DEXUSEU", "DGS10"]
            )
            
            st.caption({
                "GDP": "Gross Domestic Product",
                "UNRATE": "Unemployment Rate",
                "DFF": "Federal Funds Rate",
                "CPIAUCSL": "Consumer Price Index",
                "DEXUSEU": "USD/EUR Exchange Rate",
                "DGS10": "10-Year Treasury Rate"
            }.get(indicator, ""))
        
        with col2:
            st.info("FRED API has no rate limits")
        
        if st.button("🚀 Fetch Indicator", type="primary"):
            try:
                with st.spinner(f"Fetching {indicator}..."):
                    pipeline.fetch_fred(indicator)
                st.success(f"✅ {indicator} loaded successfully")
            except Exception as e:
                st.error(f"❌ Failed: {e}")

# === QUERY & ANALYSIS PAGE ===
elif page == "📊 Query & Analysis":
    st.title("📊 Query & Analysis")
    
    tab1, tab2, tab3 = st.tabs(["🔍 Quick Queries", "💻 SQL Editor", "📈 Analysis"])
    
    # === QUICK QUERIES ===
    with tab1:
        st.subheader("🔍 Quick Queries")
        
        query_type = st.selectbox(
            "Select Query Type",
            [
                "Latest Prices",
                "Sector Performance",
                "Top Movers (30 days)",
                "Price History"
            ]
        )
        
        if query_type == "Latest Prices":
            limit = st.number_input("Limit", min_value=10, max_value=500, value=50)
            
            if st.button("Run Query"):
                results = pipeline.query(f"""
                    SELECT s.symbol, s.name, p.close, p.timestamp
                    FROM daily_prices p
                    JOIN symbols s ON p.symbol = s.symbol
                    ORDER BY p.timestamp DESC
                    LIMIT {limit}
                """)
                
                if not results.empty:
                    st.dataframe(results, use_container_width=True)
                else:
                    st.info("No data available")
        
        elif query_type == "Sector Performance":
            if st.button("Run Query"):
                results = pipeline.query("""
                    SELECT 
                        s.sector,
                        COUNT(*) as num_symbols,
                        ROUND(AVG(p.close), 2) as avg_price,
                        ROUND(MIN(p.close), 2) as min_price,
                        ROUND(MAX(p.close), 2) as max_price
                    FROM symbols s
                    JOIN daily_prices p ON s.symbol = p.symbol
                    WHERE s.sector IS NOT NULL
                    AND p.timestamp >= date('now', '-7 days')
                    GROUP BY s.sector
                    ORDER BY avg_price DESC
                """)
                
                if not results.empty:
                    st.dataframe(results, use_container_width=True)
                    
                    fig = px.bar(results, x='sector', y='avg_price',
                               title="Average Price by Sector")
                    st.plotly_chart(fig, use_container_width=True)
                else:
                    st.info("No data available")
        
        elif query_type == "Top Movers (30 days)":
            limit = st.number_input("Top N", min_value=5, max_value=100, value=20)
            
            if st.button("Run Query"):
                results = pipeline.query(f"""
                    SELECT 
                        s.symbol,
                        s.name,
                        s.sector,
                        p1.close as latest_price,
                        p2.close as price_30d_ago,
                        ROUND(((p1.close - p2.close) / p2.close * 100), 2) as pct_change
                    FROM symbols s
                    JOIN daily_prices p1 ON s.symbol = p1.symbol
                    JOIN daily_prices p2 ON s.symbol = p2.symbol
                    WHERE p1.timestamp = (SELECT MAX(timestamp) FROM daily_prices WHERE symbol = s.symbol)
                    AND p2.timestamp = (SELECT MIN(timestamp) FROM daily_prices WHERE symbol = s.symbol 
                                       AND timestamp >= date('now', '-30 days'))
                    ORDER BY ABS(pct_change) DESC
                    LIMIT {limit}
                """)
                
                if not results.empty:
                    df = results.copy()
                    df['pct_change'] = df['pct_change'].apply(lambda x: f"{x:.2f}%")
                    st.dataframe(df, use_container_width=True)
                else:
                    st.info("No data available")
    
    # === SQL EDITOR ===
    with tab2:
        st.subheader("💻 SQL Editor")
        
        # Example queries
        examples = {
            "Select all symbols": "SELECT * FROM symbols LIMIT 100",
            "Price history": "SELECT * FROM daily_prices WHERE symbol = 'AAPL' ORDER BY timestamp DESC LIMIT 100",
            "Join symbols + prices": """SELECT s.symbol, s.name, p.close, p.timestamp 
FROM symbols s 
JOIN daily_prices p ON s.symbol = p.symbol 
LIMIT 100""",
        }
        
        example = st.selectbox("Load Example", ["Custom"] + list(examples.keys()))
        
        if example != "Custom":
            default_sql = examples[example]
        else:
            default_sql = "SELECT * FROM symbols LIMIT 100"
        
        sql_query = st.text_area(
            "SQL Query",
            value=default_sql,
            height=200
        )
        
        col1, col2 = st.columns([1, 4])
        with col1:
            if st.button("▶️ Execute", type="primary"):
                try:
                    results = pipeline.query(sql_query)
                    st.success(f"✅ Returned {len(results)} rows")
                    
                    if not results.empty:
                        st.dataframe(results, use_container_width=True, height=400)
                        
                        st.download_button(
                            "📥 Download CSV",
                            results.to_csv(index=False),
                            "query_results.csv",
                            "text/csv"
                        )
                except Exception as e:
                    st.error(f"❌ Query failed: {e}")
    
    # === ANALYSIS ===
    with tab3:
        st.subheader("📈 Price Analysis")
        
        symbol = st.text_input("Symbol", "AAPL")
        
        if st.button("📊 Analyze"):
            # Get price history
            results = pipeline.query("""
                SELECT timestamp, close, volume
                FROM daily_prices
                WHERE symbol = ?
                ORDER BY timestamp ASC
            """, [symbol])
            
            if not results.empty:
                df = results.copy()
                df['timestamp'] = pd.to_datetime(df['timestamp'])
                
                # Price chart
                fig = go.Figure()
                fig.add_trace(go.Scatter(
                    x=df['timestamp'], 
                    y=df['close'],
                    mode='lines',
                    name='Close Price'
                ))
                fig.update_layout(
                    title=f"{symbol} Price History",
                    xaxis_title="Date",
                    yaxis_title="Price ($)"
                )
                st.plotly_chart(fig, use_container_width=True)
                
                # Stats
                col1, col2, col3, col4 = st.columns(4)
                with col1:
                    st.metric("Current", f"${df['close'].iloc[-1]:.2f}")
                with col2:
                    st.metric("High", f"${df['close'].max():.2f}")
                with col3:
                    st.metric("Low", f"${df['close'].min():.2f}")
                with col4:
                    change = ((df['close'].iloc[-1] - df['close'].iloc[0]) / df['close'].iloc[0] * 100)
                    st.metric("Change", f"{change:.2f}%")
            else:
                st.info(f"No data for {symbol}")

# === WATCHLISTS PAGE ===
elif page == "⭐ Watchlists":
    st.title("⭐ Watchlist Manager")
    
    col1, col2 = st.columns([1, 2])
    
    with col1:
        st.subheader("My Watchlists")
        
        # Get existing watchlists
        watchlists = pipeline.query("SELECT DISTINCT name FROM watchlists ORDER BY name")
        watchlist_names = watchlists['name'].tolist() if not watchlists.empty else []
        
        if watchlist_names:
            selected = st.selectbox("Select Watchlist", watchlist_names)
        else:
            st.info("No watchlists yet")
            selected = None
        
        st.markdown("---")
        
        # Create new watchlist
        st.subheader("Create New")
        new_name = st.text_input("Watchlist Name")
        new_symbols = st.text_area("Symbols (comma-separated)", placeholder="AAPL, MSFT, GOOGL")
        
        if st.button("➕ Create"):
            if new_name and new_symbols:
                symbols = [s.strip() for s in new_symbols.split(',')]
                try:
                    pipeline.create_watchlist(new_name, symbols)
                    st.success(f"✅ Created watchlist: {new_name}")
                    st.rerun()
                except Exception as e:
                    st.error(f"Failed: {e}")
    
    with col2:
        if selected:
            st.subheader(f"📋 {selected}")
            
            # Get watchlist symbols
            symbols = pipeline.get_watchlist(selected)
            
            if symbols:
                # Get latest prices
                symbol_list = "','".join(symbols)
                results = pipeline.query(f"""
                    SELECT 
                        s.symbol,
                        s.name,
                        s.sector,
                        p.close as price,
                        p.timestamp
                    FROM symbols s
                    LEFT JOIN daily_prices p ON s.symbol = p.symbol
                    WHERE s.symbol IN ('{symbol_list}')
                    AND (p.timestamp = (SELECT MAX(timestamp) FROM daily_prices WHERE symbol = s.symbol) 
                         OR p.timestamp IS NULL)
                    ORDER BY s.symbol
                """)
                
                if not results.empty:
                    st.dataframe(results, use_container_width=True, height=400)
                    
                    # Fetch all button
                    if st.button(f"📥 Fetch Prices for All ({len(symbols)} symbols)"):
                        st.warning(f"This will use {len(symbols)} API calls")
                        # TODO: Implement batch fetch
                else:
                    st.info("No data loaded for these symbols yet")

# === SETTINGS PAGE ===
elif page == "⚙️ Settings":
    st.title("⚙️ Settings")
    
    tab1, tab2, tab3 = st.tabs(["🔑 API Keys", "⏱️ Rate Limits", "💾 Database"])
    
    with tab1:
        st.subheader("🔑 API Key Configuration")
        
        st.info("Edit `config/config.json` to update API keys")
        
        # Display current config (masked)
        api_keys = pipeline.api_keys
        
        for source, key in api_keys.items():
            if key:
                masked = key[:4] + "*" * (len(key) - 8) + key[-4:] if len(key) > 8 else "***"
                st.text_input(f"{source.replace('_', ' ').title()}", masked, disabled=True)
            else:
                st.text_input(f"{source.replace('_', ' ').title()}", "Not configured", disabled=True)
    
    with tab2:
        st.subheader("⏱️ API Rate Limit Status")
        
        col1, col2 = st.columns(2)
        
        with col1:
            st.markdown("**Alpha Vantage (25/day)**")
            usage_24h = pipeline.get_api_usage('alpha_vantage', hours=24)
            st.progress(usage_24h / 25)
            st.caption(f"{usage_24h} / 25 calls used (last 24h)")
        
        with col2:
            st.markdown("**Finnhub (60/min)**")
            usage_1h = pipeline.get_api_usage('finnhub', hours=1)
            st.progress(min(usage_1h / 60, 1.0))
            st.caption(f"{usage_1h} calls (last hour)")
        
        # Reset button
        if st.button("🔄 Clear API Call History"):
            pipeline.query("DELETE FROM api_calls")
            st.success("API call history cleared")
            st.rerun()
    
    with tab3:
        st.subheader("💾 Database Management")
        
        db_path = Path(pipeline.db_path)
        if db_path.exists():
            size_mb = db_path.stat().st_size / (1024 * 1024)
            st.info(f"**Size:** {size_mb:.2f} MB")
            st.info(f"**Location:** `{pipeline.db_path}`")
        
        col1, col2, col3 = st.columns(3)
        
        with col1:
            if st.button("🗜️ Vacuum (Optimize)"):
                try:
                    pipeline.vacuum()
                    st.success("✅ Database optimized")
                except Exception as e:
                    st.error(f"Failed: {e}")
        
        with col2:
            if st.button("💾 Backup"):
                try:
                    backup_path = pipeline.backup()
                    st.success(f"✅ Backup: {backup_path}")
                except Exception as e:
                    st.error(f"Failed: {e}")
        
        with col3:
            if st.button("📊 Analyze"):
                try:
                    pipeline.query("ANALYZE")
                    st.success("✅ Database analyzed")
                except Exception as e:
                    st.error(f"Failed: {e}")

# Footer
st.sidebar.markdown("---")
st.sidebar.caption("💡 **Tip:** Use the SQL Editor for custom queries")
st.sidebar.caption("📖 [Documentation](docs/QUICKSTART.md)")
