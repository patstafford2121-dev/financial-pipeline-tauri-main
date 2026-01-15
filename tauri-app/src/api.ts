// Tauri API wrapper

import { invoke } from '@tauri-apps/api/core';

// Types matching Rust structs
export interface SymbolPrice {
    symbol: string;
    price: number;
    change_percent: number;
    change_direction: string;
}

export interface CommandResult {
    success: boolean;
    message: string;
}

export interface IndicatorData {
    name: string;
    value: number;
    date: string;
}

export interface PriceData {
    date: string;
    open: number;
    high: number;
    low: number;
    close: number;
    volume: number;
}

export interface MacroData {
    indicator: string;
    value: number;
    date: string;
    source: string;
}

export interface Alert {
    id: number;
    symbol: string;
    target_price: number;
    condition: string;
    triggered: boolean;
}

export interface Position {
    id: number;
    symbol: string;
    quantity: number;
    price: number;
    position_type: string;
    date: string;
    current_price: number;
    current_value: number;
    profit_loss: number;
    profit_loss_percent: number;
}

export interface Portfolio {
    positions: Position[];
    total_value: number;
    total_profit_loss: number;
    total_profit_loss_percent: number;
}

// API functions
export async function getSymbols(): Promise<SymbolPrice[]> {
    return invoke('get_symbols');
}

export async function fetchPrices(symbols: string, period: string): Promise<CommandResult> {
    return invoke('fetch_prices', { symbols, period });
}

export async function fetchFred(indicators: string): Promise<CommandResult> {
    return invoke('fetch_fred', { indicators });
}

export async function getMacroData(): Promise<MacroData[]> {
    return invoke('get_macro_data');
}

export async function calculateIndicators(symbol: string): Promise<CommandResult> {
    return invoke('calculate_indicators', { symbol });
}

export async function getIndicators(symbol: string): Promise<IndicatorData[]> {
    return invoke('get_indicators', { symbol });
}

export async function getIndicatorHistory(symbol: string, indicatorName: string): Promise<{ date: string; value: number }[]> {
    return invoke('get_indicator_history', { symbol, indicatorName });
}

export async function getPriceHistory(symbol: string): Promise<PriceData[]> {
    return invoke('get_price_history', { symbol });
}

export async function searchSymbol(query: string): Promise<string[]> {
    return invoke('search_symbol', { query });
}

export async function exportCsv(symbol: string): Promise<CommandResult> {
    return invoke('export_csv', { symbol });
}

// Alerts
export async function addAlert(symbol: string, targetPrice: number, condition: string): Promise<CommandResult> {
    return invoke('add_alert', { symbol, targetPrice, condition });
}

export async function getAlerts(onlyActive: boolean): Promise<Alert[]> {
    return invoke('get_alerts', { onlyActive });
}

export async function deleteAlert(alertId: number): Promise<CommandResult> {
    return invoke('delete_alert', { alertId });
}

export async function checkAlerts(): Promise<Alert[]> {
    return invoke('check_alerts');
}

// Portfolio
export async function addPosition(
    symbol: string,
    quantity: number,
    price: number,
    positionType: string,
    date: string,
    notes: string | null
): Promise<CommandResult> {
    return invoke('add_position', { symbol, quantity, price, positionType, date, notes });
}

export async function getPortfolio(): Promise<Portfolio> {
    return invoke('get_portfolio');
}

export async function deletePosition(positionId: number): Promise<CommandResult> {
    return invoke('delete_position', { positionId });
}

// Google Trends
export async function fetchTrends(keyword: string): Promise<CommandResult> {
    return invoke('fetch_trends', { keyword });
}

export async function getTrends(keyword: string): Promise<{ date: string; value: number }[]> {
    return invoke('get_trends', { keyword });
}
