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
    /// Maxima excepcion adversa: cuanto se fue en contra la posicion (precio).
    pub mae: f64,
    /// Maxima excepcion favorable: cuanto llego a favor de la posicion (precio).
    pub mfe: f64,
}

/// Parámetros de configuración del backtest (engine_cfg en la DB).
/// Sesión de mercado (horas UTC). El spread real no es el mismo de día.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Session {
    Asia,
    London,
    Ny,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct EngineCfg {
    pub initial_balance: f64,
    pub fixed_lots: f64,
    pub spread: f64,        // en pips (si no hay spreads por sesion)
    pub slippage: f64,      // en pips
    pub commission_per_lot: f64,
    pub tick_size: f64,
    pub pip_size: f64,
    /// Unidades por lote: 100 000 en Forex, 50 en un futuro del Dow, 1 en acciones.
    /// El P&L es (precio de salida - precio de entrada) × contract_size × lotes.
    pub contract_size: f64,
    /// Swap del broker: dinero por lote y noche (puede ser negativo).
    pub swap_long: f64,
    pub swap_short: f64,
    /// Spreads por sesion (pips). Si los tres son None, el spread es plano.
    pub spread_asia: Option<f64>,
    pub spread_london: Option<f64>,
    pub spread_ny: Option<f64>,
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
            contract_size: 100_000.0,
            swap_long: 0.0,
            swap_short: 0.0,
            spread_asia: None,
            spread_london: None,
            spread_ny: None,
            sl_atr_mult: 2.0,
            tp_atr_mult: 3.0,
        }
    }
}

impl EngineCfg {
    /// Sesión a la que pertenece una marca de tiempo (UTC).
    pub fn session_of(time: i64) -> Session {
        let hora = ((time % 86_400) + 86_400) % 86_400 / 3_600;
        match hora {
            8..=13 => Session::London,
            14..=20 => Session::Ny,
            _ => Session::Asia,
        }
    }

    /// Spread en pips en un momento concreto (si hay spreads por sesion).
    pub fn spread_pips(&self, time: i64) -> f64 {
        let valor = match EngineCfg::session_of(time) {
            Session::Asia => self.spread_asia,
            Session::London => self.spread_london,
            Session::Ny => self.spread_ny,
        };
        valor.unwrap_or(self.spread)
    }

    /// Valor de un pip por lote (Forex) o unidades por lote (otros).
    pub fn unit_value(&self) -> f64 {
        self.contract_size
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
    /// MAE medio en % del precio de entrada: cuanto se fue en contra la estrategia.
    pub mae_pct: f64,
    /// MFE medio en % del precio de entrada: cuanto llego a favor.
    pub mfe_pct: f64,
    /// Sharpe anualizado sobre retornos DIARIOS (comparable con un equity por
    /// tiempo y con el tester de MT5). El `sharpe` de arriba es por operacion.
    pub sharpe_anual: f64,
    /// Sortino anualizado (misma base que `sharpe_anual`).
    pub sortino_anual: f64,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            net_profit: 0.0, gross_profit: 0.0, profit_factor: 0.0, win_rate: 0.0,
            max_dd: 0.0, rdd: 0.0, sharpe: 0.0, sortino: 0.0, z_score: 0.0,
            r_squared: 0.0, n_trades: 0, avg_trade: 0.0, wfe: 0.0,
            mae_pct: 0.0, mfe_pct: 0.0,
            sharpe_anual: 0.0, sortino_anual: 0.0,
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