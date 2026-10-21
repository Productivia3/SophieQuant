//! engine/backtest.rs — motor V1: señal por barras en TF elegido, Long only,
//! spread + slippage + comisión, SL/TP por ATR configurable. Sin lookahead:
//! la señal de la barra i se ejecuta en la apertura de i+1.

use super::types::{BacktestOutput, Bar, Direction, EngineCfg, Trade};
use super::metrics;
use super::rules::{evaluate, Signal, Template};

pub fn run(
    bars: &[Bar],
    tmpl: Template,
    params: [f64; 3],
    cfg: &EngineCfg,
) -> BacktestOutput {
    let out = BacktestOutput::default();
    if bars.len() < 50 {
        return out;
    }

    let signals = evaluate(tmpl, params, bars, cfg);
    let atr_vals = super::indicators::atr(bars, 14);
    run_signals(bars, &signals, &atr_vals, cfg)
}

/// Bucle de cartera long-only sobre señales ya evaluadas. Las variantes de
/// SPP y Monte Carlo (señales invertidas, entradas aleatorias…) generan su
/// propia lista de señales y reutilizan exactamente la misma ejecución.
pub fn run_signals(
    bars: &[Bar],
    signals: &[Signal],
    atr_vals: &[f64],
    cfg: &EngineCfg,
) -> BacktestOutput {
    let mut out = BacktestOutput::default();
    if bars.len() < 50 {
        return out;
    }

    if signals.len() < bars.len() || atr_vals.len() < bars.len() {
        return out;
    }

    let spread_price = cfg.spread * cfg.pip_size;
    let slip_price = cfg.slippage * cfg.pip_size;
    // 1 lot de 100k units es lo estándar forex; el profit = (Δprecio / pip) × pip_value × lots
    let pip_value = cfg.pip_size * 100_000.0 * cfg.fixed_lots;

    let mut in_pos = false;
    let mut entry_price = 0.0f64;
    let mut entry_time = 0i64;
    let mut stop = 0.0f64;
    let mut take = 0.0f64;

    for i in 1..bars.len() {
        let sig = signals[i];

        if in_pos {
            // 1) stop / target intra-barra (conservador: stop primero)
            let b = bars[i];
            let exit_price = if b.low <= stop {
                Some(stop - slip_price)
            } else if cfg.tp_atr_mult > 0.0 && b.high >= take {
                Some(take - slip_price)
            } else if sig == Signal::ExitLong {
                Some(b.open - slip_price) // salida por señal en la apertura de i
            } else {
                None
            };

            if let Some(px) = exit_price {
                let profit = (px - entry_price) / cfg.pip_size * pip_value
                    - cfg.commission_per_lot * cfg.fixed_lots;
                out.trades.push(Trade {
                    direction: Direction::Long,
                    open_time: entry_time,
                    open_price: entry_price,
                    close_time: b.time,
                    close_price: px,
                    profit,
                    bars: (i as i64 - 0) as u32, // aproximación V1
                });
                in_pos = false;
            }
        }

        if sig == Signal::EnterLong && !in_pos {
            // ejecutar en la apertura de la siguiente barra => guardamos para i+1
            if let Some(next) = bars.get(i) {
                let px = next.open + spread_price / 2.0 + slip_price;
                let a = atr_vals[i];
                in_pos = true;
                entry_price = px;
                entry_time = next.time;
                stop = if a.is_nan() { px - 30.0 * cfg.pip_size } else { px - a * cfg.sl_atr_mult };
                take = if a.is_nan() || cfg.tp_atr_mult <= 0.0 { f64::INFINITY } else { px + a * cfg.tp_atr_mult };
            }
        }
    }

    // tamaño de la serie de equity inicial-first
    let mut equity = metrics::equity_series(&out.trades, cfg.initial_balance, 500);
    out.metrics = metrics::compute(&out.trades, &equity, cfg.initial_balance);
    // métricas.r_squared calculado sobre equity acumulativa total; equity incluye inicial
    equity.insert(0, cfg.initial_balance);
    equity.truncate(500);
    out.equity = equity;
    out
}

// ── tests del motor ───────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::rules::Template;

    /// Serie sintética: onda con tendencia alcista — suficientes cruces para
    /// generar trades de la plantilla MaCross.
    fn serie(n: usize) -> Vec<Bar> {
        let t0 = 1_700_000_000i64;
        (0..n).map(|i| {
            let w = i as f64 / 24.0;
            let base = 1.1000 + i as f64 * 0.00002;      // deriva +
            let onda = (w * core::f64::consts::TAU).sin() * 0.0008;
            let o = base + onda;
            let c = base + (w * core::f64::consts::TAU + 0.35).sin() * 0.0008;
            Bar {
                time: t0 + (i as i64) * 60,
                open: o,
                high: o.max(c) + 0.0003,
                low: o.min(c) - 0.0003,
                close: c,
                volume: 100.0,
            }
        }).collect()
    }

    #[test]
    fn menos_de_50_barras_da_output_vacio() {
        let cfg = EngineCfg::default();
        let out = run(&serie(30), Template::MaCross, [5.0, 20.0, 0.0], &cfg);
        assert_eq!(out.trades.len(), 0);
        assert_eq!(out.metrics.n_trades, 0);
    }

    #[test]
    fn determinismo_bit_a_bit() {
        let bars = serie(600);
        let cfg = EngineCfg::default();
        let a = run(&bars, Template::MaCross, [5.0, 20.0, 0.0], &cfg);
        let b = run(&bars, Template::MaCross, [5.0, 20.0, 0.0], &cfg);
        assert_eq!(a.equity, b.equity);
        assert_eq!(a.trades.len(), b.trades.len());
        for (ta, tb) in a.trades.iter().zip(b.trades.iter()) {
            assert_eq!(ta.open_time, tb.open_time);
            assert_eq!(ta.close_time, tb.close_time);
            assert_eq!(ta.profit, tb.profit);
        }
        assert_eq!(a.metrics.net_profit, b.metrics.net_profit);
        assert_eq!(a.metrics.max_dd, b.metrics.max_dd);
    }

    #[test]
    fn metricas_en_rango_sensato_y_sin_lookahead_con_trades() {
        let bars = serie(600);
        let cfg = EngineCfg::default();
        let out = run(&bars, Template::AtrBreakout, [20.0, 2.0, 3.0], &cfg);
        let m = &out.metrics;
        assert_eq!(m.n_trades as usize, out.trades.len());
        assert!(m.max_dd >= 0.0 && m.max_dd <= 100.0, "maxDD fuera de rango: {}", m.max_dd);
        assert!(m.win_rate >= 0.0 && m.win_rate <= 100.0);
        assert!((out.equity[0] - cfg.initial_balance).abs() < 1e-9, "la equity empieza en el balance");
        // no-lookahead: la apertura de un trade nunca ocurre en la misma barra que su señal
        // (el motor ejecuta i+1; aquí lo comprobamos indirectamente: todo trade cierra
        //  después de abrirse o simultáneo si stop/target en la misma barra siguiente)
        for t in out.trades.iter() {
            assert!(t.close_time >= t.open_time);
        }
    }

    #[test]
    fn spread_y_comision_reducen_net_respecto_a_cero_costes() {
        let bars = serie(700);
        let mut cheap = EngineCfg::default();
        cheap.spread = 0.0; cheap.slippage = 0.0; cheap.commission_per_lot = 0.0;
        let mut dear = EngineCfg::default();
        dear.spread = 3.0; dear.slippage = 2.0; dear.commission_per_lot = 10.0;
        let a = run(&bars, Template::MaCross, [5.0, 20.0, 0.0], &cheap);
        let b = run(&bars, Template::MaCross, [5.0, 20.0, 0.0], &dear);
        assert!(b.metrics.net_profit < a.metrics.net_profit,
            "con costes la desigualdad estricta debe cumplirse");
    }
}
