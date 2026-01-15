// Main entry point
import './styles.css';
import * as api from './api';
import { TradingViewChart, IndicatorChart } from './chart';

// State
let tvChart: TradingViewChart | null = null;
let indicatorChart: IndicatorChart | null = null;
let autoRefreshTimer: number | null = null;

// Logger
function log(message: string, type: 'info' | 'success' | 'error' = 'info'): void {
    const container = document.getElementById('log-container');
    if (!container) return;

    const entry = document.createElement('div');
    entry.className = `log-entry log-${type}`;
    entry.textContent = `[${new Date().toLocaleTimeString()}] ${message}`;
    container.appendChild(entry);
    container.scrollTop = container.scrollHeight;

    console.log(`[${type.toUpperCase()}] ${message}`);
}

// Tab management
function switchTab(tabName: string): void {
    // Update tab buttons
    document.querySelectorAll('.tab').forEach(tab => {
        tab.classList.toggle('active', tab.getAttribute('data-tab') === tabName);
    });

    // Update tab content
    document.querySelectorAll('.tab-content').forEach(content => {
        const id = content.id.replace('-tab', '');
        (content as HTMLElement).style.display = id === tabName ? 'block' : 'none';
    });

    // Load data for specific tabs
    if (tabName === 'alerts') loadAlerts();
    if (tabName === 'portfolio') loadPortfolio();
    if (tabName === 'macro') loadMacroData();
    if (tabName === 'chart') initializeChart();
    if (tabName === 'indicators') initializeIndicatorChart();
}

// Initialize TradingView chart
function initializeChart(): void {
    if (!tvChart) {
        try {
            tvChart = new TradingViewChart('tradingview-chart', 'volume-chart');
            tvChart.initialize();
            log('TradingView chart initialized', 'success');
        } catch (error) {
            log(`Chart init error: ${error}`, 'error');
        }
    }
}

function initializeIndicatorChart(): void {
    if (!indicatorChart) {
        try {
            indicatorChart = new IndicatorChart('indicator-chart-container');
            indicatorChart.initialize();
        } catch (error) {
            log(`Indicator chart init error: ${error}`, 'error');
        }
    }
}

// Symbol list
async function refreshSymbolList(): Promise<void> {
    try {
        log('Refreshing symbol list...', 'info');
        const symbols = await api.getSymbols();
        const list = document.getElementById('symbol-list');
        const sortSelect = document.getElementById('sort-select') as HTMLSelectElement;

        if (!list) return;

        if (symbols && symbols.length > 0) {
            // Sort
            const sortOption = sortSelect?.value || 'change-desc';
            symbols.sort((a, b) => {
                switch (sortOption) {
                    case 'change-desc': return b.change_percent - a.change_percent;
                    case 'change-asc': return a.change_percent - b.change_percent;
                    case 'price-desc': return b.price - a.price;
                    case 'price-asc': return a.price - b.price;
                    case 'symbol-asc': return a.symbol.localeCompare(b.symbol);
                    case 'symbol-desc': return b.symbol.localeCompare(a.symbol);
                    default: return b.change_percent - a.change_percent;
                }
            });

            list.innerHTML = symbols.map(s => {
                const changeColor = s.change_direction === 'up' ? 'price-up' :
                                   s.change_direction === 'down' ? 'price-down' : 'price-unchanged';
                const changeSign = s.change_percent >= 0 ? '+' : '';
                const arrow = s.change_direction === 'up' ? '▲' :
                             s.change_direction === 'down' ? '▼' : '';

                return `
                    <li class="symbol-item" data-symbol="${s.symbol}">
                        <span class="symbol-ticker">${s.symbol}</span>
                        <div>
                            <span class="symbol-price ${changeColor}">$${s.price.toFixed(2)}</span>
                            <span class="${changeColor}" style="margin-left: 8px;">
                                ${arrow} ${changeSign}${s.change_percent.toFixed(2)}%
                            </span>
                        </div>
                    </li>
                `;
            }).join('');

            document.getElementById('symbol-count')!.textContent = symbols.length.toString();
            log(`Loaded ${symbols.length} symbols`, 'success');
        } else {
            list.innerHTML = '<li class="empty-state">No data loaded. Fetch some symbols to get started.</li>';
        }
    } catch (error) {
        log(`Error refreshing: ${error}`, 'error');
    }
}

// Fetch prices
async function fetchPrices(symbols: string, period: string): Promise<void> {
    try {
        log(`Fetching ${symbols} (${period})...`, 'info');
        const result = await api.fetchPrices(symbols, period);
        log(result.message, result.success ? 'success' : 'error');
        alert(result.message);
        await refreshSymbolList();
    } catch (error) {
        log(`Error fetching: ${error}`, 'error');
        alert(`Error: ${error}`);
    }
}

// Fetch FRED macro data
async function fetchFred(indicators: string): Promise<void> {
    try {
        log(`Fetching FRED indicators: ${indicators}...`, 'info');
        const result = await api.fetchFred(indicators);
        log(result.message, result.success ? 'success' : 'error');
        alert(result.message);
        if (result.success) {
            await loadMacroData();
            switchTab('macro');
        }
    } catch (error) {
        log(`Error fetching FRED: ${error}`, 'error');
        alert(`Error: ${error}`);
    }
}

// Load macro data
async function loadMacroData(): Promise<void> {
    try {
        log('Loading macro data...', 'info');
        const data = await api.getMacroData();
        const list = document.getElementById('macro-list');

        if (!list) return;

        if (data && data.length > 0) {
            const names: Record<string, string> = {
                'DFF': 'Fed Funds Rate',
                'UNRATE': 'Unemployment Rate',
                'GDP': 'Real GDP',
                'CPIAUCSL': 'CPI (Consumer Price Index)',
                'DGS10': '10-Year Treasury',
                'DGS2': '2-Year Treasury',
                'SP500': 'S&P 500',
                'VIXCLS': 'VIX Volatility',
                'PSAVERT': 'Personal Savings Rate',
                'INDPRO': 'Industrial Production',
            };

            list.innerHTML = data.map(d => {
                let formattedValue = d.value.toFixed(2);
                if (d.indicator === 'DFF' || d.indicator.includes('RATE')) {
                    formattedValue = d.value.toFixed(2) + '%';
                } else if (d.indicator === 'GDP') {
                    formattedValue = '$' + (d.value / 1000).toFixed(1) + 'T';
                }

                const displayName = names[d.indicator] || d.indicator;

                return `
                    <li class="symbol-item">
                        <div>
                            <span class="symbol-ticker">${d.indicator}</span>
                            <span style="color: var(--text-secondary); margin-left: 10px;">${displayName}</span>
                        </div>
                        <div>
                            <span class="symbol-price">${formattedValue}</span>
                            <span style="color: var(--text-secondary); font-size: 0.8rem; margin-left: 10px;">${d.date}</span>
                        </div>
                    </li>
                `;
            }).join('');
            log(`Loaded ${data.length} macro indicators`, 'success');
        } else {
            list.innerHTML = '<li class="empty-state">No macro data loaded. Click "FRED Macro" to fetch data.</li>';
        }
    } catch (error) {
        log(`Error loading macro data: ${error}`, 'error');
    }
}

// Alerts
async function loadAlerts(): Promise<void> {
    try {
        const alerts = await api.getAlerts(false);
        const list = document.getElementById('alerts-list');

        if (!list) return;

        if (alerts && alerts.length > 0) {
            list.innerHTML = alerts.map(a => `
                <li class="symbol-item">
                    <div>
                        <span class="symbol-ticker">${a.symbol}</span>
                        <span style="color: var(--text-secondary); margin-left: 10px;">
                            ${a.condition === 'above' ? '>=' : '<='} $${a.target_price.toFixed(2)}
                        </span>
                    </div>
                    <div>
                        <span style="color: ${a.triggered ? 'var(--success)' : 'var(--text-secondary)'};">
                            ${a.triggered ? 'TRIGGERED' : 'Active'}
                        </span>
                        <button class="btn-secondary delete-alert-btn" data-id="${a.id}" style="padding: 5px 10px; font-size: 0.8rem; margin-left: 10px;">
                            Delete
                        </button>
                    </div>
                </li>
            `).join('');
            log(`Loaded ${alerts.length} alerts`, 'success');
        } else {
            list.innerHTML = '<li class="empty-state">No alerts set. Add one to get started.</li>';
        }
    } catch (error) {
        log(`Error loading alerts: ${error}`, 'error');
    }
}

async function addAlert(): Promise<void> {
    const symbol = (document.getElementById('alert-symbol') as HTMLInputElement).value.trim();
    const price = parseFloat((document.getElementById('alert-price') as HTMLInputElement).value);
    const condition = (document.getElementById('alert-condition') as HTMLSelectElement).value;

    if (!symbol) {
        alert('Please enter a symbol');
        return;
    }

    if (isNaN(price) || price <= 0) {
        alert('Please enter a valid price');
        return;
    }

    try {
        log(`Adding alert: ${symbol} ${condition} $${price.toFixed(2)}...`, 'info');
        const result = await api.addAlert(symbol, price, condition);
        log(result.message, result.success ? 'success' : 'error');
        alert(result.message);

        // Clear form and reload
        (document.getElementById('alert-symbol') as HTMLInputElement).value = '';
        (document.getElementById('alert-price') as HTMLInputElement).value = '';
        await loadAlerts();
    } catch (error) {
        log(`Error adding alert: ${error}`, 'error');
        alert(`Error: ${error}`);
    }
}

// Portfolio
async function loadPortfolio(): Promise<void> {
    try {
        const portfolio = await api.getPortfolio();
        const list = document.getElementById('portfolio-list');

        if (!list) return;

        // Update summary
        document.getElementById('portfolio-total-value')!.textContent = `$${portfolio.total_value.toFixed(2)}`;

        const plEl = document.getElementById('portfolio-total-pl')!;
        const plColor = portfolio.total_profit_loss >= 0 ? 'var(--success)' : 'var(--error)';
        const plSign = portfolio.total_profit_loss >= 0 ? '+' : '';
        plEl.style.color = plColor;
        plEl.textContent = `${plSign}$${portfolio.total_profit_loss.toFixed(2)} (${plSign}${portfolio.total_profit_loss_percent.toFixed(2)}%)`;

        if (portfolio.positions && portfolio.positions.length > 0) {
            list.innerHTML = portfolio.positions.map(p => {
                const plColor = p.profit_loss >= 0 ? 'price-up' : 'price-down';
                const plSign = p.profit_loss >= 0 ? '+' : '';
                const arrow = p.profit_loss > 0 ? '▲' : p.profit_loss < 0 ? '▼' : '';
                const typeLabel = p.position_type === 'buy' ? 'LONG' : 'SHORT';
                const typeBadge = p.position_type === 'buy' ? 'badge-long' : 'badge-short';

                return `
                    <li class="symbol-item" style="flex-direction: column; align-items: stretch;">
                        <div style="display: flex; justify-content: space-between; align-items: center;">
                            <div>
                                <span class="symbol-ticker">${p.symbol}</span>
                                <span class="badge ${typeBadge}">${typeLabel}</span>
                                <span style="color: var(--text-secondary); margin-left: 10px; font-size: 0.85rem;">
                                    ${p.quantity} shares @ $${p.price.toFixed(2)}
                                </span>
                            </div>
                            <div>
                                <span>$${p.current_value.toFixed(2)}</span>
                                <span class="${plColor}" style="margin-left: 10px;">
                                    ${arrow} ${plSign}$${p.profit_loss.toFixed(2)} (${plSign}${p.profit_loss_percent.toFixed(2)}%)
                                </span>
                            </div>
                        </div>
                        <div style="display: flex; justify-content: space-between; align-items: center; margin-top: 8px; font-size: 0.8rem; color: var(--text-secondary);">
                            <span>Bought: ${p.date} | Current: $${p.current_price.toFixed(2)}</span>
                            <button class="btn-secondary delete-position-btn" data-id="${p.id}" style="padding: 4px 8px; font-size: 0.75rem;">
                                Remove
                            </button>
                        </div>
                    </li>
                `;
            }).join('');
            log(`Loaded ${portfolio.positions.length} positions`, 'success');
        } else {
            list.innerHTML = '<li class="empty-state">No positions. Add your first trade to start tracking.</li>';
        }
    } catch (error) {
        log(`Error loading portfolio: ${error}`, 'error');
    }
}

async function addPosition(): Promise<void> {
    const symbol = (document.getElementById('position-symbol') as HTMLInputElement).value.trim();
    const quantity = parseFloat((document.getElementById('position-quantity') as HTMLInputElement).value);
    const price = parseFloat((document.getElementById('position-price') as HTMLInputElement).value);
    const positionType = (document.getElementById('position-type') as HTMLSelectElement).value;
    const date = (document.getElementById('position-date') as HTMLInputElement).value;
    const notes = (document.getElementById('position-notes') as HTMLInputElement).value.trim() || null;

    if (!symbol || isNaN(quantity) || quantity <= 0 || isNaN(price) || price <= 0 || !date) {
        alert('Please fill in all required fields');
        return;
    }

    try {
        log(`Adding ${positionType} position: ${quantity} x ${symbol} @ $${price.toFixed(2)}...`, 'info');
        const result = await api.addPosition(symbol, quantity, price, positionType, date, notes);
        log(result.message, result.success ? 'success' : 'error');
        alert(result.message);

        // Clear form and reload
        (document.getElementById('position-symbol') as HTMLInputElement).value = '';
        (document.getElementById('position-quantity') as HTMLInputElement).value = '';
        (document.getElementById('position-price') as HTMLInputElement).value = '';
        (document.getElementById('position-notes') as HTMLInputElement).value = '';
        await loadPortfolio();
    } catch (error) {
        log(`Error adding position: ${error}`, 'error');
        alert(`Error: ${error}`);
    }
}

// Search
async function searchCompany(query: string): Promise<void> {
    const resultsDiv = document.getElementById('search-results');
    if (!resultsDiv || query.length < 2) {
        if (resultsDiv) resultsDiv.innerHTML = '';
        return;
    }

    try {
        const symbols = await api.searchSymbol(query);
        if (symbols && symbols.length > 0) {
            resultsDiv.innerHTML = symbols.map(s =>
                `<button class="btn-secondary search-result" data-symbol="${s}">${s}</button>`
            ).join('');
        } else {
            resultsDiv.innerHTML = '<span style="color: var(--text-secondary);">No matches</span>';
        }
    } catch {
        resultsDiv.innerHTML = '';
    }
}

// Auto-refresh
function updateLastRefreshTime(): void {
    const now = new Date();
    document.getElementById('last-refresh')!.textContent = now.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
}

function toggleAutoRefresh(): void {
    const toggle = document.getElementById('auto-refresh-toggle') as HTMLInputElement;
    const statusEl = document.getElementById('refresh-status')!;

    if (toggle.checked) {
        startAutoRefresh();
        statusEl.textContent = 'Auto-refresh on';
        statusEl.classList.add('active');
    } else {
        stopAutoRefresh();
        statusEl.textContent = 'Auto-refresh off';
        statusEl.classList.remove('active');
    }
}

function startAutoRefresh(): void {
    const interval = parseInt((document.getElementById('refresh-interval') as HTMLSelectElement).value);
    log(`Auto-refresh started (every ${interval / 60000} min)`, 'info');

    autoRefreshPrices();
    autoRefreshTimer = window.setInterval(autoRefreshPrices, interval);
}

function stopAutoRefresh(): void {
    if (autoRefreshTimer) {
        clearInterval(autoRefreshTimer);
        autoRefreshTimer = null;
        log('Auto-refresh stopped', 'info');
    }
}

async function autoRefreshPrices(): Promise<void> {
    try {
        const symbols = await api.getSymbols();
        if (!symbols || symbols.length === 0) {
            updateLastRefreshTime();
            return;
        }

        const symbolList = symbols.map(s => s.symbol).join(',');
        log(`Auto-refreshing ${symbols.length} symbols...`, 'info');

        const result = await api.fetchPrices(symbolList, '1d');
        log(`Auto-refresh: ${result.message}`, result.success ? 'success' : 'error');

        await refreshSymbolList();
        updateLastRefreshTime();

        // Check alerts
        const triggered = await api.checkAlerts();
        if (triggered && triggered.length > 0) {
            const messages = triggered.map(a =>
                `${a.symbol} ${a.condition === 'above' ? 'reached' : 'dropped to'} $${a.target_price.toFixed(2)}`
            ).join('\n');
            alert(`Alerts triggered!\n\n${messages}`);
            log(`${triggered.length} alerts triggered!`, 'success');
            await loadAlerts();
        }
    } catch (error) {
        log(`Auto-refresh error: ${error}`, 'error');
    }
}

// Load chart
async function loadChart(): Promise<void> {
    const symbol = (document.getElementById('chart-symbol') as HTMLInputElement).value.trim();
    if (!symbol) {
        alert('Please enter a symbol');
        return;
    }

    initializeChart();

    try {
        log(`Loading chart for ${symbol}...`, 'info');
        await tvChart?.loadSymbol(symbol);
        log(`Chart loaded for ${symbol}`, 'success');

        // Add selected indicators
        await updateChartIndicators();
    } catch (error) {
        log(`Error loading chart: ${error}`, 'error');
        alert(`Error: ${error}`);
    }
}

async function updateChartIndicators(): Promise<void> {
    if (!tvChart) return;

    tvChart.clearIndicators();

    if ((document.getElementById('show-sma20') as HTMLInputElement).checked) {
        await tvChart.addIndicator('SMA_20', '#38bdf8');
    }
    if ((document.getElementById('show-sma50') as HTMLInputElement).checked) {
        await tvChart.addIndicator('SMA_50', '#f59e0b');
    }
    if ((document.getElementById('show-ema12') as HTMLInputElement).checked) {
        await tvChart.addIndicator('EMA_12', '#8b5cf6');
    }
    if ((document.getElementById('show-bb') as HTMLInputElement).checked) {
        await tvChart.addIndicator('BB_UPPER_20', '#ec4899');
        await tvChart.addIndicator('BB_MIDDLE_20', '#ec489980');
        await tvChart.addIndicator('BB_LOWER_20', '#ec4899');
    }
}

// Calculate indicators
async function calculateIndicators(): Promise<void> {
    const symbol = (document.getElementById('indicator-symbol') as HTMLInputElement).value.trim();
    if (!symbol) {
        alert('Please enter a symbol');
        return;
    }

    try {
        log(`Calculating indicators for ${symbol}...`, 'info');
        const result = await api.calculateIndicators(symbol);
        log(result.message, result.success ? 'success' : 'error');
        alert(result.message);

        if (result.success) {
            await loadIndicatorList(symbol);
        }
    } catch (error) {
        log(`Error calculating: ${error}`, 'error');
        alert(`Error: ${error}`);
    }
}

async function loadIndicatorList(symbol: string): Promise<void> {
    try {
        const indicators = await api.getIndicators(symbol);
        const list = document.getElementById('indicator-list');

        if (!list) return;

        if (indicators && indicators.length > 0) {
            list.innerHTML = indicators.map(ind => `
                <li class="symbol-item" data-indicator="${ind.name}">
                    <span class="symbol-ticker">${ind.name}</span>
                    <span class="symbol-price">${ind.value.toFixed(2)}</span>
                </li>
            `).join('');
            log(`Loaded ${indicators.length} indicators for ${symbol}`, 'success');
        } else {
            list.innerHTML = '<li class="empty-state">No indicators calculated. Click Calculate first.</li>';
        }
    } catch (error) {
        log(`Error loading indicators: ${error}`, 'error');
    }
}

async function showIndicatorChart(): Promise<void> {
    const symbol = (document.getElementById('indicator-symbol') as HTMLInputElement).value.trim();
    const indicatorName = (document.getElementById('indicator-select') as HTMLSelectElement).value;

    if (!symbol || !indicatorName) {
        alert('Please enter a symbol and select an indicator');
        return;
    }

    initializeIndicatorChart();

    try {
        log(`Loading ${indicatorName} chart for ${symbol}...`, 'info');
        await indicatorChart?.loadIndicator(symbol, indicatorName);
        log(`Indicator chart loaded`, 'success');
    } catch (error) {
        log(`Error loading indicator chart: ${error}`, 'error');
        alert(`Error: ${error}`);
    }
}

// Event listeners
function setupEventListeners(): void {
    // Tab switching
    document.querySelectorAll('.tab').forEach(tab => {
        tab.addEventListener('click', () => {
            const tabName = tab.getAttribute('data-tab');
            if (tabName) switchTab(tabName);
        });
    });

    // Fetch form
    document.getElementById('fetch-form')?.addEventListener('submit', (e) => {
        e.preventDefault();
        const symbols = (document.getElementById('symbols') as HTMLInputElement).value;
        const period = (document.getElementById('period') as HTMLSelectElement).value;
        if (symbols) fetchPrices(symbols, period);
    });

    // Quick actions
    document.querySelectorAll('.quick-action').forEach(btn => {
        btn.addEventListener('click', () => {
            const symbols = btn.getAttribute('data-symbols');
            const fred = btn.getAttribute('data-fred');
            const period = (document.getElementById('period') as HTMLSelectElement).value;

            if (symbols) fetchPrices(symbols, period);
            if (fred) fetchFred(fred);
        });
    });

    // Refresh button
    document.getElementById('refresh-btn')?.addEventListener('click', refreshSymbolList);

    // Sort select
    document.getElementById('sort-select')?.addEventListener('change', refreshSymbolList);

    // Search
    document.getElementById('search-input')?.addEventListener('input', (e) => {
        searchCompany((e.target as HTMLInputElement).value);
    });

    // Search results (event delegation)
    document.getElementById('search-results')?.addEventListener('click', (e) => {
        const target = e.target as HTMLElement;
        if (target.classList.contains('search-result')) {
            const symbol = target.getAttribute('data-symbol');
            if (symbol) {
                (document.getElementById('search-input') as HTMLInputElement).value = '';
                document.getElementById('search-results')!.innerHTML = '';
                const period = (document.getElementById('period') as HTMLSelectElement).value;
                fetchPrices(symbol, period);
            }
        }
    });

    // Auto-refresh
    document.getElementById('auto-refresh-toggle')?.addEventListener('change', toggleAutoRefresh);
    document.getElementById('refresh-interval')?.addEventListener('change', () => {
        const toggle = document.getElementById('auto-refresh-toggle') as HTMLInputElement;
        if (toggle.checked) {
            stopAutoRefresh();
            startAutoRefresh();
        }
    });

    // Chart controls
    document.getElementById('load-chart-btn')?.addEventListener('click', loadChart);
    document.querySelectorAll('.chart-indicators input').forEach(checkbox => {
        checkbox.addEventListener('change', updateChartIndicators);
    });

    // Indicators
    document.getElementById('calc-indicators-btn')?.addEventListener('click', calculateIndicators);
    document.getElementById('show-indicator-chart-btn')?.addEventListener('click', showIndicatorChart);

    // Alerts
    document.getElementById('add-alert-btn')?.addEventListener('click', addAlert);
    document.getElementById('check-alerts-btn')?.addEventListener('click', async () => {
        try {
            log('Checking alerts...', 'info');
            const triggered = await api.checkAlerts();
            if (triggered && triggered.length > 0) {
                const messages = triggered.map(a =>
                    `${a.symbol} ${a.condition === 'above' ? 'reached' : 'dropped to'} $${a.target_price.toFixed(2)}`
                ).join('\n');
                alert(`Alerts triggered!\n\n${messages}`);
                log(`${triggered.length} alerts triggered!`, 'success');
            } else {
                alert('No alerts triggered.');
                log('No alerts triggered', 'info');
            }
            await loadAlerts();
        } catch (error) {
            log(`Error checking alerts: ${error}`, 'error');
        }
    });

    // Alert deletion (event delegation)
    document.getElementById('alerts-list')?.addEventListener('click', async (e) => {
        const target = e.target as HTMLElement;
        if (target.classList.contains('delete-alert-btn')) {
            const id = parseInt(target.getAttribute('data-id') || '0');
            if (id) {
                try {
                    const result = await api.deleteAlert(id);
                    log(result.message, result.success ? 'success' : 'error');
                    await loadAlerts();
                } catch (error) {
                    log(`Error deleting alert: ${error}`, 'error');
                }
            }
        }
    });

    // Portfolio
    document.getElementById('add-position-btn')?.addEventListener('click', addPosition);

    // Position deletion (event delegation)
    document.getElementById('portfolio-list')?.addEventListener('click', async (e) => {
        const target = e.target as HTMLElement;
        if (target.classList.contains('delete-position-btn')) {
            if (!confirm('Remove this position?')) return;
            const id = parseInt(target.getAttribute('data-id') || '0');
            if (id) {
                try {
                    const result = await api.deletePosition(id);
                    log(result.message, result.success ? 'success' : 'error');
                    await loadPortfolio();
                } catch (error) {
                    log(`Error deleting position: ${error}`, 'error');
                }
            }
        }
    });

    // Symbol list click - view in chart
    document.getElementById('symbol-list')?.addEventListener('click', (e) => {
        const target = (e.target as HTMLElement).closest('.symbol-item');
        if (target) {
            const symbol = target.getAttribute('data-symbol');
            if (symbol) {
                (document.getElementById('chart-symbol') as HTMLInputElement).value = symbol;
                (document.getElementById('indicator-symbol') as HTMLInputElement).value = symbol;
                switchTab('chart');
                loadChart();
            }
        }
    });
}

// Initialize
document.addEventListener('DOMContentLoaded', () => {
    log('Financial Pipeline UI loaded', 'success');
    setupEventListeners();
    refreshSymbolList();
    updateLastRefreshTime();

    // Set default date for position form
    const dateInput = document.getElementById('position-date') as HTMLInputElement;
    if (dateInput) {
        dateInput.valueAsDate = new Date();
    }
});
