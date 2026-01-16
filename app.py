"""
Financial Data Pipeline - Enhanced Web GUI
Door 865: Financial Data Pipeline - SQLite Edition

Launch with: streamlit run app_enhanced.py
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

# ============================================================================
# CUSTOM CSS STYLING
# ============================================================================

def load_custom_css():
    st.markdown("""
    <style>
    /* Main theme colors */
    :root {
        --primary-color: #1f77b4;
        --secondary-color: #ff7f0e;
        --success-color: #2ecc71;
        --danger-color: #e74c3c;
        --warning-color: #f39c12;
        --info-color: #3498db;
        --dark-bg: #0e1117;
        --card-bg: #1e2130;
        --text-primary: #fafafa;
        --text-secondary: #b0b0b0;
    }
    
    /* Global font improvements */
    .main {
        font-family: 'Inter', 'Segoe UI', sans-serif;
    }
    
    /* Enhanced metrics */
    [data-testid="stMetricValue"] {
        font-size: 2rem;
        font-weight: 700;
        background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
        -webkit-background-clip: text;
        -webkit-text-fill-color: transparent;
        background-clip: text;
    }
    
    [data-testid="stMetricLabel"] {
        font-size: 0.9rem;
        font-weight: 600;
        color: var(--text-secondary);
        text-transform: uppercase;
        letter-spacing: 0.5px;
    }
    
    /* Card styling */
    div[data-testid="stVerticalBlock"] > div {
        background-color: var(--card-bg);
        border-radius: 12px;
        padding: 1.5rem;
        box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
        border: 1px solid rgba(255, 255, 255, 0.05);
    }
    
    /* Enhanced dataframe */
    [data-testid="stDataFrame"] {
        border-radius: 8px;
        overflow: hidden;
        box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
    }
    
    /* Button improvements */
    .stButton > button {
        border-radius: 8px;
        font-weight: 600;
        padding: 0.5rem 1.5rem;
        transition: all 0.3s ease;
        border: none;
        box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    }
    
    .stButton > button:hover {
        transform: translateY(-2px);
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
    }
    
    /* Primary button */
    .stButton > button[kind="primary"] {
        background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    }
    
    /* Sidebar styling */
    [data-testid="stSidebar"] {
        background: linear-gradient(180deg, #1e2130 0%, #0e1117 100%);
    }
    
    [data-testid="stSidebar"] .stRadio > label {
        font-size: 1rem;
        font-weight: 600;
        padding: 0.5rem 0;
    }
    
    /* Tab styling */
    .stTabs [data-baseweb="tab-list"] {
        gap: 8px;
        background-color: transparent;
    }
    
    .stTabs [data-baseweb="tab"] {
        border-radius: 8px;
        padding: 0.5rem 1.5rem;
        font-weight: 600;
        background-color: rgba(255, 255, 255, 0.05);
    }
    
    .stTabs [aria-selected="true"] {
        background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    }
    
    /* Input fields */
    .stTextInput > div > div > input,
    .stTextArea > div > div > textarea {
        border-radius: 8px;
        border: 1px solid rgba(255, 255, 255, 0.1);
        background-color: rgba(255, 255, 255, 0.05);
    }
    
    /* Info boxes */
    .stAlert {
        border-radius: 8px;
        border-left: 4px solid var(--info-color);
    }
    
    /* Success boxes */
    .stSuccess {
        border-left-color: var(--success-color) !important;
    }
    
    /* Error boxes */
    .stError {
        border-left-color: var(--danger-color) !important;
    }
    
    /* Warning boxes */
    .stWarning {
        border-left-color: var(--warning-color) !important;
    }
    
    /* Progress bar */
    .stProgress > div > div > div {
        background: linear-gradient(90deg, #667eea 0%, #764ba2 100%);
        border-radius: 10px;
    }
    
    /* Expander */
    .streamlit-expanderHeader {
        border-radius: 8px;
        background-color: rgba(255, 255, 255, 0.05);
        font-weight: 600;
    }
    
    /* Custom metric card */
    .metric-card {
        background: linear-gradient(135deg, rgba(102, 126, 234, 0.1) 0%, rgba(118, 75, 162, 0.1) 100%);
        border-radius: 12px;
        padding: 1.5rem;
        border: 1px solid rgba(102, 126, 234, 0.2);
        box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
    }
    
    /* Scrollbar styling */
    ::-webkit-scrollbar {
        width: 8px;
        height: 8px;
    }
    
    ::-webkit-scrollbar-track {
        background: rgba(255, 255, 255, 0.05);
    }
    
    ::-webkit-scrollbar-thumb {
        background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
        border-radius: 4px;
    }
    
    /* Title styling */
    h1 {
        font-weight: 700;
        background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
        -webkit-background-clip: text;
        -webkit-text-fill-color: transparent;
        background-clip: text;
        margin-bottom: 1rem;
    }
    
    h2, h3 {
        font-weight: 600;
        color: var(--text-primary);
    }
    
    /* Divider */
    hr {
        border: none;
        height: 2px;
        background: linear-gradient(90deg, transparent, rgba(102, 126, 234, 0.5), transparent);
        margin: 2rem 0;
    }
    
    /* Plotly chart enhancements */
    .js-plotly-plot {
        border-radius: 12px;
        overflow: hidden;
    }
    </style>
    """, unsafe_allow_html=True)

# ============================================================================
# HELPER FUNCTIONS
# ============================================================================

def create_metric_card(label, value, delta=None, icon="📊"):
    """Create a styled metric card"""
    delta_html = f'<div style="color: {"#2ecc71" if delta and delta > 0 else "#e74c3c"}; font-size: 0.9rem; margin-top: 0.5rem;">{"↑" if delta and delta > 0 else "↓"} {abs(delta):.2f}%</div>' if delta else ""
    
    st.markdown(f"""
    <div class="metric-card">
        <div style="font-size: 2rem; margin-bottom: 0.5rem;">{icon}</div>
        <div style="color: #b0b0b0; font-size: 0.85rem; text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 0.5rem;">{label}</div>
        <div style="font-size: 2rem; font-weight: 700; color: #fafafa;">{value}</div>
        {delta_html}
    </div>
    """, unsafe_allow_html=True)

def format_number(num):
    """Format large numbers with K, M, B suffixes"""
    if num >= 1_000_000_000:
        return f"{num/1_000_000_000:.2f}B"
    elif num >= 1_000_000:
        return f"{num/1_000_000:.2f}M"
    elif num >= 1_000:
        return f"{num/1_000:.2f}K"
    return str(num)

# ============================================================================
# PAGE CONFIGURATION
# ============================================================================

st.set_page_config(
    page_title="Financial Data Pipeline Pro",
    page_icon="📊",
    layout="wide",
    initial_sidebar_state="expanded"
)

# Load custom CSS
load_custom_css()

# ============================================================================
# INITIALIZE PIPELINE
# ============================================================================

if 'pipeline' not in st.session_state:
    try:
        st.session_state.pipeline = FinancePipeline()
        st.session_state.pipeline.connect()
    except Exception as e:
        st.error(f"❌ Failed to initialize pipeline: {e}")
        st.stop()

pipeline = st.session_state.pipeline

# ============================================================================
# SIDEBAR NAVIGATION
# ============================================================================

with st.sidebar:
    st.markdown("""
    <div style="text-align: center; padding: 1rem 0 2rem 0;">
        <h1 style="font-size: 1.8rem; margin: 0;">📊</h1>
        <h2 style="font-size: 1.3rem; margin: 0.5rem 0 0 0;">Finance Pipeline</h2>
        <p style="color: #b0b0b0; font-size: 0.85rem; margin: 0.5rem 0 0 0;">Professional Edition</p>
    </div>
    """, unsafe_allow_html=True)
    
    st.markdown("---")
    
    page = st.radio(
        "Navigate",
        [
            "🏠 Dashboard",
            "🔍 Symbol Browser",
            "📥 Data Fetcher",
            "📊 Query & Analysis",
            "⭐ Watchlists",
            "⚙️ Settings"
        ],
        label_visibility="collapsed"
    )
    
    st.markdown("---")
    
    # Quick stats in sidebar
    st.markdown("### 📈 Quick Stats")
    
    try:
        symbol_count = pipeline.query("SELECT COUNT(*) as count FROM symbols")['count'].iloc[0]
        st.metric("Symbols", format_number(symbol_count), label_visibility="visible")
    except:
        st.metric("Symbols", "0", label_visibility="visible")
    
    try:
        price_count = pipeline.query("SELECT COUNT(*) as count FROM daily_prices")['count'].iloc[0]
        st.metric("Price Records", format_number(price_count), label_visibility="visible")
    except:
        st.metric("Price Records", "0", label_visibility="visible")
    
    st.markdown("---")
    st.caption("🔧 Door 865 - PhiSHRI")
    st.caption("v2.0.0 Enhanced Edition")

# ============================================================================
# DASHBOARD PAGE
# ============================================================================

if page == "🏠 Dashboard":
    st.title("📊 Financial Data Pipeline Dashboard")
    st.markdown("**Local-first financial data system** - Load once, query forever")
    
    st.markdown("---")
    
    # Top metrics row with enhanced cards
    col1, col2, col3, col4 = st.columns(4)
    
    with col1:
        try:
            symbol_count = pipeline.query("SELECT COUNT(*) as count FROM symbols")
            count = symbol_count['count'].iloc[0] if not symbol_count.empty else 0
            st.metric("💼 Total Symbols", f"{count:,}", help="Total number of symbols in database")
        except:
            st.metric("💼 Total Symbols", "0")
    
    with col2:
        try:
            price_count = pipeline.query("SELECT COUNT(*) as count FROM daily_prices")
            count = price_count['count'].iloc[0] if not price_count.empty else 0
            st.metric("📈 Price Records", f"{count:,}", help="Total price data points")
        except:
            st.metric("📈 Price Records", "0")
    
    with col3:
        try:
            api_usage = pipeline.get_api_usage('yahoo_finance', hours=24)
            st.metric("🔌 Yahoo Finance Calls (24h)", f"{api_usage}",
                     help="Yahoo Finance has no rate limits!")
        except:
            st.metric("🔌 Yahoo Finance Calls (24h)", "0")
    
    with col4:
        try:
            latest = pipeline.query("SELECT MAX(timestamp) as latest FROM daily_prices")
            if not latest.empty and not pd.isna(latest['latest'].iloc[0]):
                latest_date = latest['latest'].iloc[0]
                st.metric("📅 Latest Data", str(latest_date)[:10])
            else:
                st.metric("📅 Latest Data", "No data")
        except:
            st.metric("📅 Latest Data", "No data")
    
    st.markdown("---")
    
    # Two column layout for main content
    col_left, col_right = st.columns([3, 2])
    
    with col_left:
        st.subheader("📈 Recent Price Updates")
        
        try:
            recent_prices = pipeline.query("""
                SELECT s.symbol, s.name, p.close, p.volume, p.timestamp
                FROM daily_prices p
                JOIN symbols s ON p.symbol = s.symbol
                ORDER BY p.timestamp DESC
                LIMIT 15
            """)
            
            if not recent_prices.empty:
                df = recent_prices.copy()
                df['close'] = df['close'].apply(lambda x: f"${x:.2f}")
                df['volume'] = df['volume'].apply(lambda x: format_number(x) if pd.notna(x) else "N/A")
                df['timestamp'] = pd.to_datetime(df['timestamp']).dt.strftime('%Y-%m-%d')
                
                st.dataframe(
                    df,
                    use_container_width=True,
                    height=400,
                    hide_index=True,
                    column_config={
                        "symbol": st.column_config.TextColumn("Symbol", width="small"),
                        "name": st.column_config.TextColumn("Name", width="medium"),
                        "close": st.column_config.TextColumn("Price", width="small"),
                        "volume": st.column_config.TextColumn("Volume", width="small"),
                        "timestamp": st.column_config.TextColumn("Date", width="small")
                    }
                )
            else:
                st.info("💡 No price data loaded yet. Use **Data Fetcher** to load prices.")
        except Exception as e:
            st.error(f"Error loading recent prices: {e}")
    
    with col_right:
        st.subheader("🏢 Top Sectors")
        
        try:
            sectors = pipeline.query("""
                SELECT sector, COUNT(*) as count
                FROM symbols
                WHERE sector IS NOT NULL AND sector != ''
                GROUP BY sector
                ORDER BY count DESC
                LIMIT 8
            """)
            
            if not sectors.empty:
                df = sectors.copy()
                
                # Create a modern donut chart
                fig = go.Figure(data=[go.Pie(
                    labels=df['sector'],
                    values=df['count'],
                    hole=.4,
                    marker=dict(
                        colors=px.colors.sequential.Viridis,
                        line=dict(color='#0e1117', width=2)
                    ),
                    textposition='inside',
                    textinfo='label+percent',
                    hovertemplate='<b>%{label}</b><br>Count: %{value}<br>Percent: %{percent}<extra></extra>'
                )])
                
                fig.update_layout(
                    showlegend=False,
                    height=400,
                    margin=dict(t=0, b=0, l=0, r=0),
                    paper_bgcolor='rgba(0,0,0,0)',
                    plot_bgcolor='rgba(0,0,0,0)',
                    font=dict(color='#fafafa', size=11)
                )
                
                st.plotly_chart(fig, use_container_width=True)
            else:
                st.info("💡 No sector data available. Load symbols first.")
        except Exception as e:
            st.error(f"Error loading sector data: {e}")
    
    st.markdown("---")
    
    # Market overview section
    st.subheader("🌍 Market Overview")
    
    col1, col2, col3 = st.columns(3)
    
    with col1:
        try:
            equity_count = pipeline.query("SELECT COUNT(*) as count FROM symbols WHERE asset_class='equity'")['count'].iloc[0]
            st.metric("📊 Equities", format_number(equity_count))
        except:
            st.metric("📊 Equities", "0")
    
    with col2:
        try:
            etf_count = pipeline.query("SELECT COUNT(*) as count FROM symbols WHERE asset_class='etf'")['count'].iloc[0]
            st.metric("🎯 ETFs", format_number(etf_count))
        except:
            st.metric("🎯 ETFs", "0")
    
    with col3:
        try:
            crypto_count = pipeline.query("SELECT COUNT(*) as count FROM symbols WHERE asset_class='crypto'")['count'].iloc[0]
            st.metric("₿ Crypto", format_number(crypto_count))
        except:
            st.metric("₿ Crypto", "0")
    
    st.markdown("---")
    
    # Database status section
    st.subheader("💾 Database Status")
    
    col1, col2, col3 = st.columns(3)
    
    with col1:
        try:
            db_size = Path(pipeline.db_path).stat().st_size / (1024 * 1024)
            st.info(f"**💽 Database Size:** {db_size:.2f} MB")
        except:
            st.info("**💽 Database Size:** Unknown")
    
    with col2:
        st.info(f"**📂 Location:** `{Path(pipeline.db_path).name}`")
    
    with col3:
        if st.button("🔄 Backup Database", use_container_width=True):
            with st.spinner("Creating backup..."):
                try:
                    backup_path = pipeline.backup()
                    st.success(f"✅ Backup created: {Path(backup_path).name}")
                except Exception as e:
                    st.error(f"❌ Backup failed: {e}")

# ============================================================================
# SYMBOL BROWSER PAGE
# ============================================================================

elif page == "🔍 Symbol Browser":
    st.title("🔍 Symbol Browser")
    st.markdown("Browse and search **300k+ symbols** from FinanceDatabase")
    
    st.markdown("---")
    
    # Search controls in columns
    col1, col2, col3, col4 = st.columns([3, 1, 1, 1])
    
    with col1:
        search_term = st.text_input(
            "🔎 Search", 
            placeholder="e.g., Apple, AAPL, Technology, Healthcare",
            label_visibility="collapsed"
        )
    
    with col2:
        asset_class = st.selectbox(
            "Asset Class",
            ["All", "equity", "etf", "crypto", "forex", "fund"],
            label_visibility="collapsed"
        )
    
    with col3:
        sort_by = st.selectbox(
            "Sort by",
            ["symbol", "name", "sector", "market_cap"],
            label_visibility="collapsed"
        )
    
    with col4:
        limit = st.number_input(
            "Results",
            min_value=10,
            max_value=1000,
            value=100,
            step=10,
            label_visibility="collapsed"
        )
    
    # Build query
    query = "SELECT * FROM symbols WHERE 1=1"
    params = []
    
    if search_term:
        query += " AND (symbol LIKE ? OR name LIKE ? OR sector LIKE ? OR industry LIKE ?)"
        search_pattern = f"%{search_term}%"
        params.extend([search_pattern] * 4)
    
    if asset_class != "All":
        query += " AND asset_class = ?"
        params.append(asset_class)
    
    query += f" ORDER BY {sort_by} LIMIT {limit}"
    
    # Execute search
    with st.spinner("Searching..."):
        try:
            results = pipeline.query(query, params) if params else pipeline.query(query)
            
            if not results.empty:
                st.success(f"✅ Found **{len(results):,}** symbols")
                
                # Column selector
                available_cols = [col for col in results.columns if col in 
                                ['symbol', 'name', 'sector', 'industry', 'market_cap', 
                                 'asset_class', 'exchange', 'country']]
                default_cols = [col for col in ['symbol', 'name', 'sector', 'industry', 'market_cap'] 
                               if col in available_cols]
                
                display_cols = st.multiselect(
                    "Display Columns",
                    available_cols,
                    default=default_cols
                )
                
                if display_cols:
                    df = results[display_cols].copy()
                    
                    # Format market cap if present
                    if 'market_cap' in df.columns:
                        df['market_cap'] = df['market_cap'].apply(
                            lambda x: format_number(x) if pd.notna(x) else "N/A"
                        )
                    
                    st.dataframe(
                        df,
                        use_container_width=True,
                        height=600,
                        hide_index=True
                    )
                    
                    # Export button
                    col1, col2, col3 = st.columns([1, 1, 4])
                    with col1:
                        st.download_button(
                            "📥 Download CSV",
                            results.to_csv(index=False),
                            "symbols_export.csv",
                            "text/csv",
                            use_container_width=True
                        )
                    with col2:
                        st.download_button(
                            "📄 Download JSON",
                            results.to_json(orient='records', indent=2),
                            "symbols_export.json",
                            "application/json",
                            use_container_width=True
                        )
                else:
                    st.warning("⚠️ Please select at least one column to display")
            else:
                st.warning("⚠️ No results found. Try a different search term.")
        except Exception as e:
            st.error(f"❌ Search failed: {e}")

# ============================================================================
# DATA FETCHER PAGE
# ============================================================================

elif page == "📥 Data Fetcher":
    st.title("📥 Data Fetcher")
    st.markdown("Fetch price data, fundamentals, and macro economic indicators")
    
    st.markdown("---")
    
    # Tab layout
    tab1, tab2, tab3 = st.tabs(["📈 Price Data", "📊 Fundamentals", "🌍 Macro Indicators"])
    
    # === PRICE FETCHER ===
    with tab1:
        st.subheader("📈 Fetch Historical Price Data")
        st.success("✅ **Powered by Yahoo Finance** - FREE & Unlimited! No API key required.")

        col1, col2 = st.columns([2, 1])

        with col1:
            symbols_input = st.text_area(
                "Enter Symbols (comma-separated)",
                placeholder="AAPL, MSFT, GOOGL, AMZN, TSLA",
                height=100,
                help="Enter stock symbols separated by commas"
            )

        with col2:
            st.markdown("**⚙️ Options**")

            period = st.selectbox(
                "Time Period",
                ["1y", "2y", "5y", "10y", "max"],
                index=0,
                help="1y=1 year, 2y=2 years, 5y=5 years, 10y=10 years, max=all available"
            )

            refetch = st.checkbox(
                "Clear existing data first",
                value=False,
                help="Delete existing price data for these symbols before fetching"
            )

        st.markdown("---")

        col1, col2, col3 = st.columns([1, 1, 4])

        with col1:
            if st.button("🚀 Fetch Data", type="primary", use_container_width=True):
                if symbols_input:
                    symbols = [s.strip().upper() for s in symbols_input.split(',')]

                    progress_bar = st.progress(0)
                    status_text = st.empty()

                    success_count = 0
                    fail_count = 0

                    for i, symbol in enumerate(symbols):
                        status_text.text(f"Fetching {symbol}... ({i+1}/{len(symbols)})")

                        try:
                            if refetch:
                                pipeline.clear_symbol_prices(symbol)
                            pipeline.fetch_prices_yahoo(symbol, period=period)
                            success_count += 1
                        except Exception as e:
                            st.error(f"❌ Failed to fetch {symbol}: {e}")
                            fail_count += 1

                        progress_bar.progress((i + 1) / len(symbols))

                    status_text.empty()
                    progress_bar.empty()

                    if success_count > 0:
                        st.success(f"✅ Successfully fetched {success_count} symbols!")
                    if fail_count > 0:
                        st.warning(f"⚠️ Failed to fetch {fail_count} symbols")
                else:
                    st.warning("⚠️ Please enter at least one symbol")

        with col2:
            if st.button("📋 Use Watchlist", use_container_width=True):
                st.info("Select a watchlist from the Watchlists page")
    
    # === FUNDAMENTALS ===
    with tab2:
        st.subheader("📊 Fundamental Data")
        st.info("🚧 Fundamental data fetching coming soon! Will include income statements, balance sheets, cash flows, and key ratios.")
    
    # === MACRO DATA ===
    with tab3:
        st.subheader("🌍 Macro Economic Indicators")
        
        st.markdown("""
        Fetch economic indicators from the **Federal Reserve Economic Data (FRED)**.
        
        **Popular Indicators:**
        - **GDP** - Gross Domestic Product
        - **UNRATE** - Unemployment Rate
        - **DFF** - Federal Funds Rate
        - **CPIAUCSL** - Consumer Price Index (Inflation)
        - **T10Y2Y** - 10-Year Treasury vs 2-Year (Yield Curve)
        """)
        
        col1, col2 = st.columns([2, 1])
        
        with col1:
            indicators_input = st.text_area(
                "FRED Indicator Codes",
                placeholder="GDP, UNRATE, DFF, CPIAUCSL",
                height=100
            )
        
        with col2:
            st.markdown("**📅 Date Range**")
            start_date = st.date_input("Start Date", value=datetime.now() - timedelta(days=365*5))
            end_date = st.date_input("End Date", value=datetime.now())
        
        if st.button("📥 Fetch Macro Data", type="primary"):
            if indicators_input:
                indicators = [i.strip().upper() for i in indicators_input.split(',')]
                
                progress_bar = st.progress(0)
                status_text = st.empty()
                
                for i, indicator in enumerate(indicators):
                    status_text.text(f"Fetching {indicator}...")
                    
                    try:
                        pipeline.fetch_fred(indicator)
                        st.success(f"✅ Fetched {indicator}")
                    except Exception as e:
                        st.error(f"❌ Failed: {e}")
                    
                    progress_bar.progress((i + 1) / len(indicators))
                
                status_text.empty()
                progress_bar.empty()
            else:
                st.warning("⚠️ Please enter at least one indicator code")

# ============================================================================
# QUERY & ANALYSIS PAGE
# ============================================================================

elif page == "📊 Query & Analysis":
    st.title("📊 Query & Analysis")
    st.markdown("Analyze your data with **SQL queries** and interactive visualizations")
    
    st.markdown("---")
    
    # Tab layout
    tab1, tab2, tab3 = st.tabs(["🔍 Quick Queries", "💻 SQL Editor", "📈 Chart Analysis"])
    
    # === QUICK QUERIES ===
    with tab1:
        st.subheader("🔍 Pre-built Analysis")
        
        col1, col2 = st.columns(2)
        
        with col1:
            if st.button("📊 Sector Performance", use_container_width=True):
                try:
                    results = pipeline.query("""
                        SELECT 
                            s.sector,
                            COUNT(DISTINCT s.symbol) as symbol_count,
                            AVG(p.close) as avg_price,
                            MAX(p.close) as max_price,
                            MIN(p.close) as min_price
                        FROM symbols s
                        JOIN daily_prices p ON s.symbol = p.symbol
                        WHERE s.sector IS NOT NULL
                        GROUP BY s.sector
                        ORDER BY symbol_count DESC
                        LIMIT 10
                    """)
                    
                    if not results.empty:
                        st.dataframe(results, use_container_width=True)
                        
                        # Chart
                        fig = px.bar(results, x='sector', y='symbol_count',
                                   title="Symbols per Sector")
                        st.plotly_chart(fig, use_container_width=True)
                except Exception as e:
                    st.error(f"❌ Query failed: {e}")
        
        with col2:
            if st.button("🔥 Top Movers (30 days)", use_container_width=True):
                try:
                    results = pipeline.query("""
                        SELECT 
                            s.symbol,
                            s.name,
                            MIN(p.close) as low_30d,
                            MAX(p.close) as high_30d,
                            ((MAX(p.close) - MIN(p.close)) / MIN(p.close) * 100) as change_pct
                        FROM symbols s
                        JOIN daily_prices p ON s.symbol = p.symbol
                        WHERE p.timestamp >= date('now', '-30 days')
                        GROUP BY s.symbol, s.name
                        HAVING change_pct > 0
                        ORDER BY change_pct DESC
                        LIMIT 20
                    """)
                    
                    if not results.empty:
                        df = results.copy()
                        df['change_pct'] = df['change_pct'].apply(lambda x: f"{x:.2f}%")
                        st.dataframe(df, use_container_width=True)
                except Exception as e:
                    st.error(f"❌ Query failed: {e}")
    
    # === SQL EDITOR ===
    with tab2:
        st.subheader("💻 Custom SQL Query")
        
        # Example queries dropdown
        examples = {
            "Top 10 Symbols": "SELECT * FROM symbols LIMIT 10",
            "Recent Prices": """SELECT s.symbol, s.name, p.close, p.timestamp 
FROM symbols s 
JOIN daily_prices p ON s.symbol = p.symbol 
ORDER BY p.timestamp DESC 
LIMIT 100""",
            "Technology Sector": """SELECT symbol, name, market_cap 
FROM symbols 
WHERE sector = 'Technology' 
ORDER BY market_cap DESC 
LIMIT 50""",
            "Price Summary": """SELECT 
    symbol,
    MIN(close) as low,
    MAX(close) as high,
    AVG(close) as avg,
    COUNT(*) as days
FROM daily_prices
GROUP BY symbol
ORDER BY days DESC
LIMIT 20"""
        }
        
        example = st.selectbox("📚 Load Example Query", ["Custom"] + list(examples.keys()))
        
        if example != "Custom":
            default_sql = examples[example]
        else:
            default_sql = "SELECT * FROM symbols LIMIT 100"
        
        sql_query = st.text_area(
            "SQL Query",
            value=default_sql,
            height=250,
            help="Write your SQL query here"
        )
        
        col1, col2, col3 = st.columns([1, 1, 4])
        
        with col1:
            if st.button("▶️ Execute", type="primary", use_container_width=True):
                with st.spinner("Executing query..."):
                    try:
                        results = pipeline.query(sql_query)
                        
                        if not results.empty:
                            st.success(f"✅ Returned {len(results):,} rows × {len(results.columns)} columns")
                            
                            # Display results
                            st.dataframe(results, use_container_width=True, height=400)
                            
                            # Download options
                            col1, col2 = st.columns(2)
                            with col1:
                                st.download_button(
                                    "📥 Download CSV",
                                    results.to_csv(index=False),
                                    "query_results.csv",
                                    "text/csv",
                                    use_container_width=True
                                )
                            with col2:
                                st.download_button(
                                    "📄 Download JSON",
                                    results.to_json(orient='records', indent=2),
                                    "query_results.json",
                                    "application/json",
                                    use_container_width=True
                                )
                        else:
                            st.info("✅ Query executed successfully but returned no results")
                    except Exception as e:
                        st.error(f"❌ Query failed: {e}")
        
        with col2:
            with st.expander("📖 Schema Reference"):
                st.markdown("""
                **Tables:**
                - `symbols` - Symbol catalog
                - `daily_prices` - OHLCV data
                - `fundamentals` - Financial data
                - `macro_data` - Economic indicators
                - `watchlists` - User watchlists
                - `api_calls` - API usage log
                
                **Views:**
                - `latest_prices` - Most recent prices
                - `symbol_summary` - Symbols with latest prices
                """)
    
    # === ANALYSIS ===
    with tab3:
        st.subheader("📈 Advanced Price Analysis")

        # Input controls in columns
        col1, col2, col3 = st.columns([2, 1, 1])

        with col1:
            symbol = st.text_input("Symbol", "AAPL")

        with col2:
            chart_type = st.selectbox("Chart Type", [
                "Line",
                "Candlestick",
                "Area",
                "OHLC Bar"
            ])

        with col3:
            comparison_symbol = st.text_input("Compare With", placeholder="Optional")

        # Time range controls
        col1, col2, col3, col4 = st.columns(4)

        with col1:
            time_range = st.selectbox("Time Range", [
                "All Data",
                "1 Month",
                "3 Months",
                "6 Months",
                "1 Year",
                "2 Years",
                "5 Years",
                "Custom"
            ])

        with col2:
            if time_range == "Custom":
                start_date = st.date_input("Start Date",
                    value=pd.Timestamp.now() - pd.DateOffset(years=1))
            else:
                start_date = None

        with col3:
            if time_range == "Custom":
                end_date = st.date_input("End Date",
                    value=pd.Timestamp.now())
            else:
                end_date = None

        with col4:
            show_volume = st.checkbox("Show Volume", value=True)

        # Additional options
        with st.expander("📊 Chart Options"):
            col1, col2, col3 = st.columns(3)

            with col1:
                show_ma20 = st.checkbox("20-Day MA", value=False)
                show_ma50 = st.checkbox("50-Day MA", value=False)

            with col2:
                show_ma100 = st.checkbox("100-Day MA", value=False)
                show_ma200 = st.checkbox("200-Day MA", value=False)

            with col3:
                log_scale = st.checkbox("Log Scale", value=False)
                show_grid = st.checkbox("Show Grid", value=True)

        if st.button("📊 Analyze", type="primary"):
            # Build date filter query
            date_filter = ""
            if time_range != "All Data" and time_range != "Custom":
                days_map = {
                    "1 Month": 30,
                    "3 Months": 90,
                    "6 Months": 180,
                    "1 Year": 365,
                    "2 Years": 730,
                    "5 Years": 1825
                }
                days = days_map[time_range]
                date_filter = f"AND timestamp >= date('now', '-{days} days')"
            elif time_range == "Custom" and start_date and end_date:
                date_filter = f"AND timestamp BETWEEN '{start_date}' AND '{end_date}'"

            # Get price history
            query = f"""
                SELECT timestamp, open, high, low, close, volume
                FROM daily_prices
                WHERE symbol = ?
                {date_filter}
                ORDER BY timestamp ASC
            """

            results = pipeline.query(query, [symbol])

            if not results.empty:
                df = results.copy()
                df['timestamp'] = pd.to_datetime(df['timestamp'])

                # Calculate moving averages if requested
                if show_ma20:
                    df['MA20'] = df['close'].rolling(window=20).mean()
                if show_ma50:
                    df['MA50'] = df['close'].rolling(window=50).mean()
                if show_ma100:
                    df['MA100'] = df['close'].rolling(window=100).mean()
                if show_ma200:
                    df['MA200'] = df['close'].rolling(window=200).mean()

                # Create figure based on chart type
                if chart_type == "Candlestick":
                    fig = go.Figure(data=[go.Candlestick(
                        x=df['timestamp'],
                        open=df['open'],
                        high=df['high'],
                        low=df['low'],
                        close=df['close'],
                        name=symbol
                    )])
                elif chart_type == "OHLC Bar":
                    fig = go.Figure(data=[go.Ohlc(
                        x=df['timestamp'],
                        open=df['open'],
                        high=df['high'],
                        low=df['low'],
                        close=df['close'],
                        name=symbol
                    )])
                elif chart_type == "Area":
                    fig = go.Figure()
                    fig.add_trace(go.Scatter(
                        x=df['timestamp'],
                        y=df['close'],
                        fill='tozeroy',
                        name=symbol,
                        line=dict(color='rgb(0, 176, 246)', width=2)
                    ))
                else:  # Line chart
                    fig = go.Figure()
                    fig.add_trace(go.Scatter(
                        x=df['timestamp'],
                        y=df['close'],
                        mode='lines',
                        name=symbol,
                        line=dict(width=2)
                    ))

                # Add moving averages
                if show_ma20:
                    fig.add_trace(go.Scatter(
                        x=df['timestamp'], y=df['MA20'],
                        name='MA20', line=dict(dash='dash', width=1)
                    ))
                if show_ma50:
                    fig.add_trace(go.Scatter(
                        x=df['timestamp'], y=df['MA50'],
                        name='MA50', line=dict(dash='dash', width=1)
                    ))
                if show_ma100:
                    fig.add_trace(go.Scatter(
                        x=df['timestamp'], y=df['MA100'],
                        name='MA100', line=dict(dash='dot', width=1)
                    ))
                if show_ma200:
                    fig.add_trace(go.Scatter(
                        x=df['timestamp'], y=df['MA200'],
                        name='MA200', line=dict(dash='dot', width=1)
                    ))

                # Add comparison symbol if provided
                if comparison_symbol:
                    comp_results = pipeline.query(query, [comparison_symbol])
                    if not comp_results.empty:
                        comp_df = comp_results.copy()
                        comp_df['timestamp'] = pd.to_datetime(comp_df['timestamp'])

                        # Normalize to percentage change for comparison
                        df['normalized'] = (df['close'] / df['close'].iloc[0] - 1) * 100
                        comp_df['normalized'] = (comp_df['close'] / comp_df['close'].iloc[0] - 1) * 100

                        fig.add_trace(go.Scatter(
                            x=comp_df['timestamp'],
                            y=comp_df['normalized'],
                            mode='lines',
                            name=f"{comparison_symbol} (% change)",
                            yaxis='y2',
                            line=dict(dash='dash')
                        ))

                        # Add second y-axis for comparison
                        fig.update_layout(
                            yaxis2=dict(
                                title="% Change",
                                overlaying='y',
                                side='right'
                            )
                        )

                # Update layout
                fig.update_layout(
                    title=f"{symbol} - {chart_type} Chart ({time_range})",
                    xaxis_title="Date",
                    yaxis_title="Price ($)",
                    yaxis_type='log' if log_scale else 'linear',
                    xaxis=dict(showgrid=show_grid),
                    yaxis=dict(showgrid=show_grid),
                    hovermode='x unified',
                    height=600
                )

                st.plotly_chart(fig, use_container_width=True)

                # Volume chart (if enabled)
                if show_volume and 'volume' in df.columns:
                    vol_fig = go.Figure()
                    vol_fig.add_trace(go.Bar(
                        x=df['timestamp'],
                        y=df['volume'],
                        name='Volume',
                        marker_color='lightblue'
                    ))
                    vol_fig.update_layout(
                        title="Trading Volume",
                        xaxis_title="Date",
                        yaxis_title="Volume",
                        height=200
                    )
                    st.plotly_chart(vol_fig, use_container_width=True)

                # Statistics
                st.markdown("---")
                st.subheader("📊 Statistics")

                col1, col2, col3, col4, col5 = st.columns(5)

                with col1:
                    st.metric("Current", f"${df['close'].iloc[-1]:.2f}")

                with col2:
                    period_high = df['close'].max()
                    st.metric("Period High", f"${period_high:.2f}")

                with col3:
                    period_low = df['close'].min()
                    st.metric("Period Low", f"${period_low:.2f}")

                with col4:
                    change = ((df['close'].iloc[-1] - df['close'].iloc[0]) / df['close'].iloc[0] * 100)
                    st.metric("Change", f"{change:+.2f}%")

                with col5:
                    avg_volume = df['volume'].mean() if 'volume' in df.columns else 0
                    st.metric("Avg Volume", f"{avg_volume/1e6:.1f}M")

                # Additional stats in expandable section
                with st.expander("📈 More Statistics"):
                    col1, col2, col3 = st.columns(3)

                    with col1:
                        st.write("**Price Stats**")
                        st.write(f"Mean: ${df['close'].mean():.2f}")
                        st.write(f"Median: ${df['close'].median():.2f}")
                        st.write(f"Std Dev: ${df['close'].std():.2f}")

                    with col2:
                        st.write("**Returns**")
                        daily_returns = df['close'].pct_change()
                        st.write(f"Best Day: {daily_returns.max()*100:.2f}%")
                        st.write(f"Worst Day: {daily_returns.min()*100:.2f}%")
                        st.write(f"Avg Daily: {daily_returns.mean()*100:.2f}%")

                    with col3:
                        st.write("**Volatility**")
                        volatility = daily_returns.std() * (252 ** 0.5) * 100  # Annualized
                        st.write(f"Annual: {volatility:.2f}%")
                        st.write(f"Data Points: {len(df)}")
                        st.write(f"Date Range: {len(df)} days")

                # Download option
                st.download_button(
                    "📥 Download Data as CSV",
                    df.to_csv(index=False),
                    f"{symbol}_data.csv",
                    "text/csv"
                )

            else:
                st.info(f"No data for {symbol}")

# ============================================================================
# WATCHLISTS PAGE
# ============================================================================

elif page == "⭐ Watchlists":
    st.title("⭐ Watchlist Manager")
    st.markdown("Organize and track your favorite symbols")
    
    st.markdown("---")
    
    col1, col2 = st.columns([1, 2])
    
    with col1:
        st.subheader("📋 My Watchlists")
        
        # Get existing watchlists
        try:
            watchlists = pipeline.query("SELECT DISTINCT name FROM watchlists ORDER BY name")
            watchlist_names = watchlists['name'].tolist() if not watchlists.empty else []
        except:
            watchlist_names = []
        
        if watchlist_names:
            selected = st.selectbox("Select Watchlist", watchlist_names)
        else:
            st.info("💡 No watchlists yet. Create one below!")
            selected = None
        
        st.markdown("---")
        
        # Create new watchlist
        st.subheader("➕ Create New")
        
        with st.form("create_watchlist"):
            new_name = st.text_input("Watchlist Name", placeholder="e.g., Tech Giants")
            new_symbols = st.text_area(
                "Symbols (comma-separated)",
                placeholder="AAPL, MSFT, GOOGL, AMZN",
                height=100
            )
            
            submitted = st.form_submit_button("Create Watchlist", type="primary", use_container_width=True)
            
            if submitted:
                if new_name and new_symbols:
                    symbols = [s.strip().upper() for s in new_symbols.split(',')]
                    try:
                        pipeline.create_watchlist(new_name, symbols)
                        st.success(f"✅ Created watchlist: {new_name}")
                        st.rerun()
                    except Exception as e:
                        st.error(f"❌ Failed: {e}")
                else:
                    st.warning("⚠️ Please fill in all fields")
    
    with col2:
        if selected:
            st.subheader(f"📊 {selected}")
            
            try:
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
                            p.volume,
                            p.timestamp as last_update
                        FROM symbols s
                        LEFT JOIN daily_prices p ON s.symbol = p.symbol
                        WHERE s.symbol IN ('{symbol_list}')
                        AND (p.timestamp = (SELECT MAX(timestamp) FROM daily_prices WHERE symbol = s.symbol) 
                             OR p.timestamp IS NULL)
                        ORDER BY s.symbol
                    """)
                    
                    if not results.empty:
                        df = results.copy()
                        df['price'] = df['price'].apply(lambda x: f"${x:.2f}" if pd.notna(x) else "N/A")
                        df['volume'] = df['volume'].apply(lambda x: format_number(x) if pd.notna(x) else "N/A")
                        df['last_update'] = pd.to_datetime(df['last_update']).dt.strftime('%Y-%m-%d')
                        
                        st.dataframe(
                            df,
                            use_container_width=True,
                            height=500,
                            hide_index=True,
                            column_config={
                                "symbol": st.column_config.TextColumn("Symbol", width="small"),
                                "name": st.column_config.TextColumn("Name", width="medium"),
                                "sector": st.column_config.TextColumn("Sector", width="medium"),
                                "price": st.column_config.TextColumn("Price", width="small"),
                                "volume": st.column_config.TextColumn("Volume", width="small"),
                                "last_update": st.column_config.TextColumn("Updated", width="small")
                            }
                        )
                        
                        # Action buttons
                        col1, col2, col3 = st.columns(3)
                        
                        with col1:
                            if st.button(f"📥 Fetch All Prices ({len(symbols)})", use_container_width=True):
                                st.info(f"Fetching {len(symbols)} symbols from Yahoo Finance...")
                                try:
                                    pipeline.fetch_prices_batch_yahoo(symbols, period="1y")
                                    st.success(f"✅ Successfully fetched {len(symbols)} symbols!")
                                    st.rerun()
                                except Exception as e:
                                    st.error(f"❌ Fetch failed: {e}")
                        
                        with col2:
                            st.download_button(
                                "📥 Export CSV",
                                results.to_csv(index=False),
                                f"{selected}_watchlist.csv",
                                "text/csv",
                                use_container_width=True
                            )
                        
                        with col3:
                            if st.button("🗑️ Delete Watchlist", use_container_width=True):
                                try:
                                    pipeline.query("DELETE FROM watchlists WHERE name = ?", [selected])
                                    st.success(f"✅ Deleted watchlist: {selected}")
                                    st.rerun()
                                except Exception as e:
                                    st.error(f"❌ Failed: {e}")
                    else:
                        st.info("💡 No data loaded for these symbols yet")
            except Exception as e:
                st.error(f"❌ Error loading watchlist: {e}")

# ============================================================================
# SETTINGS PAGE
# ============================================================================

elif page == "⚙️ Settings":
    st.title("⚙️ Settings & Configuration")
    st.markdown("Manage API keys, rate limits, and database settings")
    
    st.markdown("---")
    
    tab1, tab2, tab3, tab4 = st.tabs(["🔑 API Keys", "⏱️ Rate Limits", "💾 Database", "🎨 Appearance"])
    
    # === API KEYS ===
    with tab1:
        st.subheader("🔑 API Configuration")

        st.success("✅ **No API keys required!** This pipeline uses Yahoo Finance which is completely free.")

        st.markdown("---")

        st.markdown("""
        ### Data Sources

        **Yahoo Finance (yfinance)**
        - **Cost:** FREE
        - **Rate Limits:** None
        - **API Key:** Not required
        - **Data:** Historical OHLCV, adjusted for splits and dividends

        **FRED (Federal Reserve Economic Data)**
        - **Cost:** FREE
        - **Rate Limits:** None
        - **API Key:** Not required
        - **Data:** Macro economic indicators (GDP, unemployment, inflation, etc.)
        """)
    
    # === RATE LIMITS ===
    with tab2:
        st.subheader("⏱️ API Usage")

        st.success("✅ **Yahoo Finance** - No rate limits! Fetch as much data as you need.")

        st.markdown("---")

        # API call history
        st.subheader("📊 Recent API Calls")

        try:
            recent_calls = pipeline.query("""
                SELECT source, COUNT(*) as calls, MAX(timestamp) as last_call
                FROM api_calls
                WHERE timestamp >= datetime('now', '-24 hours')
                GROUP BY source
                ORDER BY calls DESC
            """)

            if not recent_calls.empty:
                st.dataframe(recent_calls, use_container_width=True, hide_index=True)
            else:
                st.info("No API calls in the last 24 hours")
        except:
            st.error("Error loading API call history")

        # Reset button
        if st.button("🔄 Clear API Call History", type="secondary"):
            try:
                pipeline.query("DELETE FROM api_calls")
                st.success("✅ API call history cleared")
                st.rerun()
            except Exception as e:
                st.error(f"❌ Failed: {e}")
    
    # === DATABASE ===
    with tab3:
        st.subheader("💾 Database Management")
        
        try:
            db_path = Path(pipeline.db_path)
            if db_path.exists():
                size_mb = db_path.stat().st_size / (1024 * 1024)
                
                col1, col2, col3 = st.columns(3)
                
                with col1:
                    st.metric("💽 Database Size", f"{size_mb:.2f} MB")
                
                with col2:
                    # Count tables
                    tables = pipeline.query("SELECT COUNT(*) as count FROM sqlite_master WHERE type='table'")
                    table_count = tables['count'].iloc[0] if not tables.empty else 0
                    st.metric("📋 Tables", table_count)
                
                with col3:
                    st.metric("📂 Location", db_path.name)
                
                st.info(f"**Full Path:** `{pipeline.db_path}`")
        except Exception as e:
            st.error(f"Error loading database info: {e}")
        
        st.markdown("---")
        
        st.subheader("🛠️ Maintenance Operations")
        
        col1, col2, col3 = st.columns(3)
        
        with col1:
            if st.button("🗜️ Vacuum (Optimize)", use_container_width=True, type="primary"):
                with st.spinner("Optimizing database..."):
                    try:
                        pipeline.vacuum()
                        st.success("✅ Database optimized!")
                        st.info("💡 Vacuum reclaims unused space and defragments the database")
                    except Exception as e:
                        st.error(f"❌ Optimization failed: {e}")
        
        with col2:
            if st.button("💾 Create Backup", use_container_width=True, type="primary"):
                with st.spinner("Creating backup..."):
                    try:
                        backup_path = pipeline.backup()
                        st.success(f"✅ Backup created!")
                        st.info(f"📁 {Path(backup_path).name}")
                    except Exception as e:
                        st.error(f"❌ Backup failed: {e}")
        
        with col3:
            if st.button("📊 Analyze Schema", use_container_width=True, type="primary"):
                with st.spinner("Analyzing database..."):
                    try:
                        pipeline.query("ANALYZE")
                        st.success("✅ Database analyzed!")
                        st.info("💡 ANALYZE updates query planner statistics")
                    except Exception as e:
                        st.error(f"❌ Analysis failed: {e}")
        
        st.markdown("---")
        
        # Table statistics
        st.subheader("📊 Table Statistics")
        
        try:
            table_stats = pipeline.query("""
                SELECT 
                    name as table_name,
                    (SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND tbl_name=m.name) as index_count
                FROM sqlite_master m
                WHERE type='table'
                AND name NOT LIKE 'sqlite_%'
                ORDER BY name
            """)
            
            if not table_stats.empty:
                st.dataframe(table_stats, use_container_width=True, hide_index=True)
        except Exception as e:
            st.error(f"Error loading table stats: {e}")
    
    # === APPEARANCE ===
    with tab4:
        st.subheader("🎨 Appearance Settings")
        
        st.info("🎨 **Theme:** Dark Mode (Default)\n\nThis app is optimized for dark mode with custom gradient styling.")
        
        st.markdown("---")
        
        st.markdown("### 🌈 Color Scheme")
        st.markdown("""
        Current theme uses a modern gradient design:
        - **Primary:** Purple-Blue Gradient (#667eea → #764ba2)
        - **Success:** Green (#2ecc71)
        - **Error:** Red (#e74c3c)
        - **Warning:** Orange (#f39c12)
        """)
        
        # Color preview
        col1, col2, col3, col4 = st.columns(4)
        
        with col1:
            st.markdown("""
            <div style="background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); 
                        padding: 2rem; border-radius: 8px; text-align: center; color: white; font-weight: bold;">
                Primary
            </div>
            """, unsafe_allow_html=True)
        
        with col2:
            st.markdown("""
            <div style="background: #2ecc71; 
                        padding: 2rem; border-radius: 8px; text-align: center; color: white; font-weight: bold;">
                Success
            </div>
            """, unsafe_allow_html=True)
        
        with col3:
            st.markdown("""
            <div style="background: #e74c3c; 
                        padding: 2rem; border-radius: 8px; text-align: center; color: white; font-weight: bold;">
                Error
            </div>
            """, unsafe_allow_html=True)
        
        with col4:
            st.markdown("""
            <div style="background: #f39c12; 
                        padding: 2rem; border-radius: 8px; text-align: center; color: white; font-weight: bold;">
                Warning
            </div>
            """, unsafe_allow_html=True)

# ============================================================================
# FOOTER
# ============================================================================

st.sidebar.markdown("---")
st.sidebar.caption("💡 **Pro Tip:** Bookmark frequently used queries")
st.sidebar.caption("📖 [View Documentation](docs/QUICKSTART.md)")
st.sidebar.caption("🐛 [Report Issues](https://github.com/yourusername/FinancePipeline)")
