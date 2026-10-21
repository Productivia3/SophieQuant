//! engine/types.rs — tipos compartidos del motor.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bar {
    pub time: i64,   // unix epoch seconds UTC (inicio de barra)
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Direction { Long, Short }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trade {
    pub direction: Direction,
    pub open_time: i64,
    pub open_price: f64,
    pub close_time: i64,
    pub close_price: f64,
    pub profit: f64, // en dinero de cuenta, tras comisiones y spread
    pub bars: u32,
}

/// Parámetros de configuración del backtest (engine_cfg en la DB).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct EngineCfg {
    pub initial_balance: f64,
    pub fixed_lots: f64,
    pub spread: f64,        // en pips
    pub slippage: f64,      // en pips
    pub commission_per_lot: f64,
    pub tick_size: f64,
    pub pip_size: f64,
    // gestión de salida:
    pub sl_atr_mult: f64,   // stop = N × ATR
    pub tp_atr_mult: f64,   // target = M × ATR (0 = desactivado)
}

impl Default for EngineCfg {
    fn default() -> Self {
        Self {
            initial_balance: 10_000.0,
            fixed_lots: 0.1,
            spread: 1.0,
            slippage: 0.5,
            commission_per_lot: 0.0,
            tick_size: 0.00001,
            pip_size: 0.0001,
            sl_atr_mult: 2.0,
            tp_atr_mult: 3.0,
        }
    }
}

/// Resultado agregado de un backtest.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Metrics {
    pub net_profit: f64,
    pub gross_profit: f64,
    pub profit_factor: f64,
    pub win_rate: f64,
    pub max_dd: f64,       // % del peak
    pub rdd: f64,          // recovery factor (net / maxDD en dinero)
    pub sharpe: f64,
    pub sortino: f64,
    pub z_score: f64,
    pub r_squared: f64,
    pub n_trades: u32,
    pub avg_trade: f64,
    pub wfe: f64,          // placeholder V1: se calcula en Matriz WF
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            net_profit: 0.0, gross_profit: 0.0, profit_factor: 0.0, win_rate: 0.0,
            max_dd: 0.0, rdd: 0.0, sharpe: 0.0, sortino: 0.0, z_score: 0.0,
            r_squared: 0.0, n_trades: 0, avg_trade: 0.0, wfe: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct BacktestOutput {
    pub metrics: Metrics,
    pub equity: Vec<f64>,   // muestras de equity (espaciadas)
    pub trades: Vec<Trade>,
}

impl Default for BacktestOutput {
    fn default() -> Self {
        Self { metrics: Metrics::default(), equity: vec![], trades: vec![] }
    }
}