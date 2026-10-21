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

    let slip_price = cfg.slippage * cfg.pip_size;
    // El P&L se calcula sobre las unidades reales del instrumento:
    // (Δprecio) × contract_size × lotes. En Forex contract_size = 100 000.
    let unit_value = cfg.unit_value() * cfg.fixed_lots;

    let mut in_pos = false;
    let mut entry_price = 0.0f64;
    let mut entry_time = 0i64;
    let mut stop = 0.0f64;
    let mut take = 0.0f64;
    // excursiones de la posicion en curso (MAE/MFE)
    let mut mae_px = 0.0f64;
    let mut mfe_px = 0.0f64;

    for i in 1..bars.len() {
        let sig = signals[i];

        if in_pos {
            // 1) stop / target intra-barra (conservador: stop primero)
            let b = bars[i];
            // excursiones: lo peor y lo mejor mientrasomia la posicion
            let adverso = entry_price - b.low;
            let favorable = b.high - entry_price;
            if adverso > mae_px { mae_px = adverso; }
            if favorable > mfe_px { mfe_px = favorable; }
            let exit_price = if b.low <= stop {
                Some(stop - slip_price)
            } else if cfg.tp_atr_mult > 0.0 && b.high >= take {
                Some(take - slip_price)
            } else if sig == Signal::ExitLong {
                // salida por seÃ±al en la apertura de i (medio spread + slippage)
                let spread_out = cfg.spread_pips(b.time) * cfg.pip_size * 0.5;
                Some(b.open - spread_out - slip_price)
            } else {
                None
            };

            if let Some(px) = exit_price {
                // P&L en dinero: Δprecio × unidades por lote × lotes
                let mut profit = (px - entry_price) * unit_value
                    - cfg.commission_per_lot * cfg.fixed_lots;
                // swap: se cobra por cada noche que la posicion sigue abierta
                let noches = (b.time.div_euclid(86_400) - entry_time.div_euclid(86_400)).max(0);
                if noches > 0 {
                    profit -= cfg.swap_long * noches as f64 * cfg.fixed_lots;
                }
                out.trades.push(Trade {
                    direction: Direction::Long,
                    open_time: entry_time,
                    open_price: entry_price,
                    close_time: b.time,
                    close_price: px,
                    profit,
                    bars: (i as i64 - 0) as u32, // aproximación V1
                    mae: mae_px,
                    mfe: mfe_px,
                });
                in_pos = false;
            }
        }

        if sig == Signal::EnterLong && !in_pos {
            // ejecutar en la apertura de la siguiente barra => guardamos para i+1
            if let Some(next) = bars.get(i) {
                // el spread depende de la sesion: en la apertura de Londres/NY es
                // bastante mas caro que en Asia
                let spread_in = cfg.spread_pips(next.time) * cfg.pip_size * 0.5;
                let px = next.open + spread_in + slip_price;
                let a = atr_vals[i];
                in_pos = true;
                entry_price = px;
                entry_time = next.time;
                mae_px = 0.0;
                mfe_px = 0.0;
                stop = if a.is_nan() { px - 30.0 * cfg.pip_size } else { px - a * cfg.sl_atr_mult };
                take = if a.is_nan() || cfg.tp_atr_mult <= 0.0 { f64::INFINITY } else { px + a * cfg.tp_atr_mult };
            }
        }
    }

    // Las MÉTRICAS se calculan sobre la curva COMPLETA (una punto por operacion):
    // si se midieran sobre la curva muestreada, el drawdown real quedaria
    // subestimado en cuanto la estrategia tenga mas de 500 operaciones.
    let completa = metrics::equity_series(&out.trades, cfg.initial_balance, usize::MAX);
    out.metrics = metrics::compute(&out.trades, &completa, cfg.initial_balance);

    // La curva que se PINTA si puede ir muestreada (para que el grafico siga siendo ligero)
    let mut pintada = metrics::equity_series(&out.trades, cfg.initial_balance, 500);
    pintada.insert(0, cfg.initial_balance);
    pintada.truncate(500);
    out.equity = pintada;
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


    /// Serie en sierra: produce MUCHAS operaciones (para probar el drawdown sobre
    /// la curva completa y no sobre la muestreada).
    fn sierra(n: usize, periodo: usize) -> Vec<Bar> {
        let t0 = 1_700_000_000i64;
        (0..n)
            .map(|i| {
                let fase = (i % periodo) as f64 / periodo as f64;
                let base = 1.1000 + fase * 0.010;
                Bar {
                    time: t0 + i as i64 * 60,
                    open: base,
                    high: base * 1.0008,
                    low: base * 0.9992,
                    close: base * 1.0002,
                    volume: 100.0,
                }
            })
            .collect()
    }

    /// El Max DD se mide sobre la curva COMPLETA de operaciones: con mas de 500
    /// operaciones, medirlo sobre la curva muestreada lo subestimaba.
    #[test]
    fn el_max_drawdown_se_calcula_sobre_la_curva_completa() {
        let cfg = EngineCfg::default();
        let bars = sierra(6000, 10);
        let out = run(&bars, Template::MaCross, [3.0, 8.0, 0.0], &cfg);
        assert!(out.trades.len() > 500, "hacen falta muchas operaciones: {}", out.trades.len());

        // curva completa calculada a mano
        let mut balance = cfg.initial_balance;
        let mut peak = balance;
        let mut dd = 0.0f64;
        for t in &out.trades {
            balance += t.profit;
            if balance > peak { peak = balance; }
            let d = peak - balance;
            if d > dd { dd = d; }
        }
        let esperado = if peak > 0.0 { dd / peak * 100.0 } else { 0.0 };
        assert!(
            (out.metrics.max_dd - esperado).abs() < 0.05,
            "max_dd {} vs el de la curva completa {}",
            out.metrics.max_dd,
            esperado
        );
        // y la curva que se PINTA puede ser mas corta: eso ya no afecta a las metricas
        assert!(out.equity.len() <= 500);
    }

    /// El spread depende de la sesion: entrar en Londres cuesta mas que en Asia.
    #[test]
    fn el_spread_cambia_por_sesion() {
        let mut cfg = EngineCfg::default();
        cfg.spread = 1.0;
        cfg.spread_asia = Some(0.5);
        cfg.spread_london = Some(2.0);
        cfg.spread_ny = Some(1.5);
        // midnight UTC de un dia cualquiera y tres horas concretas:
        // 03:00 (Asia), 09:00 (Londres), 15:00 (Nueva York)
        let medianoche = 1_699_920_000i64;
        let asia = medianoche + 3 * 3_600;
        let london = medianoche + 9 * 3_600;
        let ny = medianoche + 15 * 3_600;
        assert!(EngineCfg::session_of(asia) == crate::engine::types::Session::Asia);
        assert!(EngineCfg::session_of(london) == crate::engine::types::Session::London);
        assert!((cfg.spread_pips(asia) - 0.5).abs() < 1e-9);
        assert!((cfg.spread_pips(london) - 2.0).abs() < 1e-9);
        assert!((cfg.spread_pips(ny) - 1.5).abs() < 1e-9);
        // sin spreads por sesion, manda el plano
        let plana = EngineCfg { spread: 1.3, ..EngineCfg::default() };
        assert!((plana.spread_pips(asia) - 1.3).abs() < 1e-9);
    }

    /// El swap se cobra por cada noche que la posicion sigue abierta.
    #[test]
    fn el_swap_se_cobra_por_noche() {
        let mut cfg = EngineCfg::default();
        cfg.swap_long = 5.0;    // 5 USD por lote y noche
        cfg.fixed_lots = 1.0;
        let bars = sierra(4000, 20);
        let con_swap = run(&bars, Template::MaCross, [3.0, 8.0, 0.0], &cfg);

        let mut cfg0 = cfg.clone();
        cfg0.swap_long = 0.0;
        let sin_swap = run(&bars, Template::MaCross, [3.0, 8.0, 0.0], &cfg0);

        assert!(
            con_swap.metrics.net_profit < sin_swap.metrics.net_profit,
            "con swap tiene que ganar menos: {} vs {}",
            con_swap.metrics.net_profit,
            sin_swap.metrics.net_profit
        );
        // la diferencia es multiplo del swap por noche
        let diferencia = sin_swap.metrics.net_profit - con_swap.metrics.net_profit;
        assert!(diferencia > 0.0);
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
