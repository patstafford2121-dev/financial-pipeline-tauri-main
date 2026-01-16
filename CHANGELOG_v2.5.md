# Financial Pipeline v2.5 - Feature Update Report

**From:** KALIC (Terminal Agent)
**To:** Lunacy & Team
**Date:** 2026-01-16
**Re:** Watchlist System, Favorites, and UI Improvements

---

## New Features Implemented

### 1. Symbol Groups / Watchlists
Full CRUD operations for organizing symbols into named groups.

**Backend (db.rs):**
- `create_watchlist(name, symbols, description)` - Create with initial symbols
- `get_watchlist(name)` - Get symbols in a watchlist
- `get_all_watchlists()` - List all with symbol counts
- `get_watchlist_full(name)` - Full details including symbols
- `delete_watchlist(name)` - Remove watchlist and associations
- `add_symbol_to_watchlist()` / `remove_symbol_from_watchlist()` - Manage membership
- `update_watchlist_description()` / `rename_watchlist()` - Edit metadata

**Frontend:**
- New "Groups" tab with create form
- Quick preset buttons (AI Stocks, Agriculture, Crypto, Gold Miners, EV Makers)
- Group detail view with symbol management
- "Fetch All Prices" for entire group

### 2. Favorites System (Moon Icons)
Symbols can be marked as "favorites" for targeted auto-refresh.

**Database:**
- New `favorited INTEGER DEFAULT 0` column in `symbols` table
- Migration runs automatically on schema init (checks if column exists first)

**Methods:**
- `toggle_symbol_favorite(symbol)` - Toggle and return new state
- `is_symbol_favorited(symbol)` - Check status
- `get_favorited_symbols()` - List all favorited

**UI:**
- Yellow pill "FAV" buttons in Chart, Indicators, Alerts, Groups tabs
- Circular moon toggle buttons in Stocks list
- Click to toggle - yellow fill when active

### 3. Targeted Auto-Refresh
Auto-refresh now ONLY updates favorited symbols instead of all loaded symbols.

**Before:** Refreshed ALL symbols (could be 100+ API calls)
**After:** Only refreshes symbols marked as favorites (user-controlled scope)

**Code location:** `main.ts:autoRefreshPrices()`

### 4. Expanded Indicator Dropdown
Increased from 6 to 20+ indicators, organized by category:

| Category | Indicators |
|----------|------------|
| Momentum | RSI(14), Stochastic %K/%D, CCI(20) |
| Trend | MACD Line/Signal/Histogram, ADX(14) |
| Moving Averages | SMA(20/50), EMA(12/26) |
| Volatility | ATR(14), Bollinger Upper/Middle/Lower |
| Volume | OBV |

### 5. Indicator List Click-to-Chart
Clicking any indicator in the calculated values list now displays its chart automatically.

**Code location:** `main.ts` event listener on `#indicator-list`

### 6. Reduced Batch Sizes
Changed from 20 to 5 symbols per API batch for S&P 100, ASX 100, and group fetches.

**Rationale:** Improves stability, reduces chance of rate limiting, clearer progress feedback.

---

## Problems Fixed

### 1. WebView Caching Issue
**Problem:** HTML/CSS/JS changes weren't appearing in the app.
**Solution:** Run `npm run build` to force Vite rebuild before `cargo tauri dev`.
**Prevention:** Clear `.vite` cache when changes don't appear.

### 2. Auto-Refresh Overwhelming API
**Problem:** Auto-refresh was fetching ALL symbols (100+), causing slow updates and potential rate limits.
**Solution:** Favorites system lets users choose which symbols to auto-refresh.

### 3. Limited Indicator Selection
**Problem:** Only 6 indicators in dropdown, missing common ones.
**Solution:** Expanded to 20+ with categorized `<optgroup>` organization.

### 4. Indicator List Not Interactive
**Problem:** Clicking calculated indicators did nothing.
**Solution:** Added click handler to load chart for clicked indicator.

---

## Best Practices for Expansion

### Adding New Database Features
1. Add column to `SCHEMA_SQL` const in `db.rs`
2. Add migration check in `run_migrations()` using PRAGMA table_info
3. Add CRUD methods to `Database` impl
4. Add Tauri commands in `lib.rs`
5. Add API functions in `api.ts`
6. Add UI in `index.html` and handlers in `main.ts`

**Example migration pattern:**
```rust
fn run_migrations(&self) -> Result<()> {
    let columns: Vec<String> = self.conn
        .prepare("PRAGMA table_info(table_name)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<SqliteResult<Vec<_>>>()?;

    if !columns.contains(&"new_column".to_string()) {
        self.conn.execute(
            "ALTER TABLE table_name ADD COLUMN new_column TYPE DEFAULT value",
            []
        )?;
    }
    Ok(())
}
```

### Adding New Indicators
1. Implement calculation in `src/indicators.rs`
2. Add to `calculate_all_indicators()` function
3. Add option to dropdown in `index.html`

### Frontend Changes Not Appearing
```bash
cd tauri-app
rm -rf node_modules/.vite dist
npm run build
cargo tauri dev
```

### Tauri Command Pattern
```rust
#[tauri::command]
fn my_command(state: State<AppState>, param: String) -> Result<ReturnType, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.my_method(&param).map_err(|e| e.to_string())
}
```

Register in `invoke_handler` array.

---

## File Locations Summary

| Component | Path |
|-----------|------|
| Database layer | `src/db.rs` |
| Tauri commands | `tauri-app/src-tauri/src/lib.rs` |
| Frontend API | `tauri-app/src/api.ts` |
| Main UI logic | `tauri-app/src/main.ts` |
| HTML structure | `tauri-app/index.html` |
| Styles | `tauri-app/src/styles.css` |
| Indicators | `src/indicators.rs` |

---

## Commit Reference
`2a5dc81` - Add watchlists, favorites system, and expanded indicators

---

*Report generated by KALIC - STRYK's Terminal Agent*
