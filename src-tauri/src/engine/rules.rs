//! engine/rules.rs — plantillas de estrategia + evaluación de señales.
//! Cada plantilla es una función pura de (barras, params) → señales. Clean-room.

use super::types::{Bar, EngineCfg};
use super::indicators as ind;

use serde::{Deserialize, Serialize};

/// Quieres tres plantillas built-in en V1; params p1..p3 con significado propio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Template {
    MaCross,        // cruce de medias rápidas/lentas
    RsiReversion,   // reversión con RSI
    AtrBreakout,    // ruptura de N barras con stop ATR
}

impl Template {
    pub fn from_id(id: i64) -> Option<Self> {
        match id {
            1 => Some(Template::MaCross),
            2 => Some(Template::RsiReversion),
            3 => Some(Template::AtrBreakout),
            _ => None,
        }
    }
    pub fn label(&self) -> &'static str {
        match self {
            Template::MaCross => "Cruce de medias",
            Template::RsiReversion => "Reversión osciladora",
            Template::AtrBreakout => "Ruptura con filtro de tendencia",
        }
    }
    pub fn default_params(&self) -> [f64; 3] {
        match self {
            Template::MaCross => [20.0, 50.0, 0.0],
            Template::RsiReversion => [14.0, 30.0, 70.0],
            Template::AtrBreakout => [20.0, 2.0, 3.0],
        }
    }
    pub fn param_names(&self) -> [&'static str; 3] {
        match self {
            Template::MaCross => ["p_fast", "p_slow", "—"],
            Template::RsiReversion => ["p_rsi", "th_low", "th_high"],
            Template::AtrBreakout => ["p_lookback", "sl_atr", "tp_atr"],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal { None, EnterLong, ExitLong }

/// Evaluador por barra: genera señales Long-only en V1 (short en Fase 2).
pub fn evaluate(tmpl: Template, p: [f64; 3], bars: &[Bar], _cfg: &EngineCfg) -> Vec<Signal> {
    let n = bars.len();
    let mut out = vec![Signal::None; n];
    let closes: Vec<f64> = bars.iter().map(|b| b.close).collect();

    match tmpl {
        Template::MaCross => {
            let fast = ind::ema(&closes, p[0].max(2.0) as usize);
            let slow = ind::sma(&closes, p[1].max(3.0) as usize);
            for i in 1..n {
                if !fast[i].is_nan() && !slow[i].is_nan() && !fast[i - 1].is_nan() && !slow[i - 1].is_nan() {
                    if fast[i - 1] <= slow[i - 1] && fast[i] > slow[i] {
                        out[i] = Signal::EnterLong;
                    } else if fast[i - 1] >= slow[i - 1] && fast[i] < slow[i] {
                        out[i] = Signal::ExitLong;
                    }
                }
            }
        }
        Template::RsiReversion => {
            let r = ind::rsi(&closes, p[0].max(2.0) as usize);
            for i in 1..n {
                if r[i - 1].is_nan() || r[i].is_nan() { continue }
                if r[i - 1] >= p[1] && r[i] < p[1] {
                    out[i] = Signal::EnterLong;   // sobreventa
                } else if ind::crosses_above(&r, &|_| p[2], i) {
                    out[i] = Signal::ExitLong;    // sobrecompra (cruce al alza del nivel)
                }
            }
        }
        Template::AtrBreakout => {
            let _a = ind::atr(bars, 14);
            let lookback = p[0].max(5.0) as usize;
            for i in lookback..n {
                let hh = ind::highest(&closes, lookback, i - 1);
                if !hh.is_nan() && bars[i].close > hh {
                    out[i] = Signal::EnterLong;
                } else if i > 1 {
                    let ll = ind::lowest(&closes, lookback, i - 1);
                    if !ll.is_nan() && bars[i].close < ll {
                        out[i] = Signal::ExitLong;
                    }
                }
            }
        }
    }
    out
}
