// TradingView Lightweight Charts integration

import {
    createChart,
    ColorType,
    CrosshairMode,
    IChartApi,
    ISeriesApi,
    CandlestickData,
    LineData,
    HistogramData,
    Time,
} from 'lightweight-charts';
import { getPriceHistory, getIndicatorHistory, PriceData } from './api';

// Chart colors matching our theme
const CHART_COLORS = {
    background: '#0f172a',
    text: '#94a3b8',
    grid: '#1e293b',
    upColor: '#22c55e',
    downColor: '#ef4444',
    wickUp: '#22c55e',
    wickDown: '#ef4444',
    sma20: '#38bdf8',
    sma50: '#f59e0b',
    ema12: '#8b5cf6',
    bbUpper: '#ec4899',
    bbLower: '#ec4899',
    bbMiddle: '#ec4899',
    volume: '#475569',
    volumeUp: '#22c55e80',
    volumeDown: '#ef444480',
};

// Chart manager class
export class TradingViewChart {
    private chart: IChartApi | null = null;
    private volumeChart: IChartApi | null = null;
    private candleSeries: ISeriesApi<'Candlestick'> | null = null;
    private volumeSeries: ISeriesApi<'Histogram'> | null = null;
    private indicatorSeries: Map<string, ISeriesApi<'Line'>> = new Map();

    private mainContainer: HTMLElement;
    private volumeContainer: HTMLElement;
    private currentSymbol: string = '';

    constructor(mainContainerId: string, volumeContainerId: string) {
        const mainEl = document.getElementById(mainContainerId);
        const volumeEl = document.getElementById(volumeContainerId);

        if (!mainEl || !volumeEl) {
            throw new Error('Chart containers not found');
        }

        this.mainContainer = mainEl;
        this.volumeContainer = volumeEl;
    }

    public initialize(): void {
        // Main price chart
        this.chart = createChart(this.mainContainer, {
            layout: {
                background: { type: ColorType.Solid, color: CHART_COLORS.background },
                textColor: CHART_COLORS.text,
            },
            grid: {
                vertLines: { color: CHART_COLORS.grid },
                horzLines: { color: CHART_COLORS.grid },
            },
            crosshair: {
                mode: CrosshairMode.Normal,
            },
            rightPriceScale: {
                borderColor: CHART_COLORS.grid,
            },
            timeScale: {
                borderColor: CHART_COLORS.grid,
                timeVisible: true,
                secondsVisible: false,
            },
            width: this.mainContainer.clientWidth,
            height: 400,
        });

        // Volume chart
        this.volumeChart = createChart(this.volumeContainer, {
            layout: {
                background: { type: ColorType.Solid, color: CHART_COLORS.background },
                textColor: CHART_COLORS.text,
            },
            grid: {
                vertLines: { color: CHART_COLORS.grid },
                horzLines: { color: CHART_COLORS.grid },
            },
            rightPriceScale: {
                borderColor: CHART_COLORS.grid,
            },
            timeScale: {
                borderColor: CHART_COLORS.grid,
                visible: false,
            },
            width: this.volumeContainer.clientWidth,
            height: 100,
        });

        // Create candlestick series
        this.candleSeries = this.chart.addCandlestickSeries({
            upColor: CHART_COLORS.upColor,
            downColor: CHART_COLORS.downColor,
            borderVisible: false,
            wickUpColor: CHART_COLORS.wickUp,
            wickDownColor: CHART_COLORS.wickDown,
        });

        // Create volume series
        this.volumeSeries = this.volumeChart.addHistogramSeries({
            color: CHART_COLORS.volume,
            priceFormat: {
                type: 'volume',
            },
            priceScaleId: '',
        });

        // Sync time scales
        this.chart.timeScale().subscribeVisibleTimeRangeChange(() => {
            const range = this.chart?.timeScale().getVisibleRange();
            if (range && this.volumeChart) {
                this.volumeChart.timeScale().setVisibleRange(range);
            }
        });

        // Handle resize
        window.addEventListener('resize', this.handleResize.bind(this));
    }

    private handleResize(): void {
        if (this.chart) {
            this.chart.applyOptions({ width: this.mainContainer.clientWidth });
        }
        if (this.volumeChart) {
            this.volumeChart.applyOptions({ width: this.volumeContainer.clientWidth });
        }
    }

    public async loadSymbol(symbol: string): Promise<void> {
        this.currentSymbol = symbol.toUpperCase();

        try {
            const priceData = await getPriceHistory(this.currentSymbol);

            if (!priceData || priceData.length === 0) {
                throw new Error('No price data available');
            }

            // Convert to chart format
            const candleData: CandlestickData[] = priceData.map((p: PriceData) => ({
                time: p.date as Time,
                open: p.open,
                high: p.high,
                low: p.low,
                close: p.close,
            }));

            const volumeData: HistogramData[] = priceData.map((p: PriceData) => ({
                time: p.date as Time,
                value: p.volume,
                color: p.close >= p.open ? CHART_COLORS.volumeUp : CHART_COLORS.volumeDown,
            }));

            // Update series
            this.candleSeries?.setData(candleData);
            this.volumeSeries?.setData(volumeData);

            // Fit content
            this.chart?.timeScale().fitContent();
            this.volumeChart?.timeScale().fitContent();

        } catch (error) {
            console.error('Error loading chart data:', error);
            throw error;
        }
    }

    public async addIndicator(name: string, color: string): Promise<void> {
        if (!this.chart || !this.currentSymbol) return;

        // Remove existing if present
        this.removeIndicator(name);

        try {
            const data = await getIndicatorHistory(this.currentSymbol, name);

            if (!data || data.length === 0) {
                console.warn(`No data for indicator ${name}`);
                return;
            }

            const lineData: LineData[] = data.map(d => ({
                time: d.date as Time,
                value: d.value,
            }));

            const series = this.chart.addLineSeries({
                color: color,
                lineWidth: 2,
                priceLineVisible: false,
                lastValueVisible: true,
                title: name,
            });

            series.setData(lineData);
            this.indicatorSeries.set(name, series);

        } catch (error) {
            console.error(`Error adding indicator ${name}:`, error);
        }
    }

    public removeIndicator(name: string): void {
        const series = this.indicatorSeries.get(name);
        if (series && this.chart) {
            this.chart.removeSeries(series);
            this.indicatorSeries.delete(name);
        }
    }

    public clearIndicators(): void {
        this.indicatorSeries.forEach((_series, name) => {
            this.removeIndicator(name);
        });
    }

    public destroy(): void {
        window.removeEventListener('resize', this.handleResize.bind(this));
        this.chart?.remove();
        this.volumeChart?.remove();
        this.chart = null;
        this.volumeChart = null;
    }
}

// Indicator chart for separate panel (RSI, MACD, etc.)
export class IndicatorChart {
    private chart: IChartApi | null = null;
    private series: ISeriesApi<'Line'> | null = null;
    private container: HTMLElement;

    constructor(containerId: string) {
        const el = document.getElementById(containerId);
        if (!el) {
            throw new Error('Indicator chart container not found');
        }
        this.container = el;
    }

    public initialize(): void {
        this.chart = createChart(this.container, {
            layout: {
                background: { type: ColorType.Solid, color: CHART_COLORS.background },
                textColor: CHART_COLORS.text,
            },
            grid: {
                vertLines: { color: CHART_COLORS.grid },
                horzLines: { color: CHART_COLORS.grid },
            },
            rightPriceScale: {
                borderColor: CHART_COLORS.grid,
            },
            timeScale: {
                borderColor: CHART_COLORS.grid,
                timeVisible: true,
            },
            width: this.container.clientWidth,
            height: 280,
        });

        window.addEventListener('resize', this.handleResize.bind(this));
    }

    private handleResize(): void {
        if (this.chart) {
            this.chart.applyOptions({ width: this.container.clientWidth });
        }
    }

    public async loadIndicator(symbol: string, indicatorName: string): Promise<void> {
        if (!this.chart) return;

        // Remove existing series
        if (this.series) {
            this.chart.removeSeries(this.series);
        }

        try {
            const data = await getIndicatorHistory(symbol.toUpperCase(), indicatorName);

            if (!data || data.length === 0) {
                throw new Error('No indicator data available');
            }

            // Choose color based on indicator
            let color = CHART_COLORS.sma20;
            if (indicatorName.startsWith('RSI')) color = '#f59e0b';
            else if (indicatorName.startsWith('MACD')) color = '#ef4444';
            else if (indicatorName.startsWith('STOCH')) color = '#06b6d4';
            else if (indicatorName.startsWith('ADX')) color = '#f97316';
            else if (indicatorName === 'OBV') color = '#a855f7';

            const lineData: LineData[] = data.map(d => ({
                time: d.date as Time,
                value: d.value,
            }));

            this.series = this.chart.addLineSeries({
                color: color,
                lineWidth: 2,
                title: `${symbol} ${indicatorName}`,
            });

            this.series.setData(lineData);
            this.chart.timeScale().fitContent();

        } catch (error) {
            console.error('Error loading indicator:', error);
            throw error;
        }
    }

    public destroy(): void {
        window.removeEventListener('resize', this.handleResize.bind(this));
        this.chart?.remove();
        this.chart = null;
    }
}
