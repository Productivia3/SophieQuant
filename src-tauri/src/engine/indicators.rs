//! engine/indicators.rs — indicadores técnicos estándar escritos desde cero.
//! Todos devuelven Vec<f64> alineado con las barras (los primeros valores = NaN).

use super::types::Bar;

fn _nan_vec(n: usize) -> Vec<f64> { vec![f64::NAN; n] }

pub fn sma(values: &[f64], period: usize) -> Vec<f64> {
    let mut out = _nan_vec(values.len());
    if period == 0 || values.len() < period { return out }
    let mut sum: f64 = values[..period].iter().sum();
    out[period - 1] = sum / period as f64;
    for i in period..values.len() {
        sum += values[i] - values[i - period];
        out[i] = sum / period as f64;
    }
    out
}

pub fn ema(values: &[f64], period: usize) -> Vec<f64> {
    let mut out = _nan_vec(values.len());
    if period == 0 || values.len() < period { return out }
    let k = 2.0 / (period as f64 + 1.0);
    let mut avg: f64 = values[..period].iter().sum::<f64>() / period as f64;
    out[period - 1] = avg;
    for i in period..values.len() {
        avg = values[i] * k + avg * (1.0 - k);
        out[i] = avg;
    }
    out
}

pub fn rsi(values: &[f64], period: usize) -> Vec<f64> {
    let mut out = _nan_vec(values.len());
    if period == 0 || values.len() <= period { return out }
    let mut gain = 0.0; let mut loss = 0.0;
    for i in 1..=period {
        let d = values[i] - values[i - 1];
        if d > 0.0 { gain += d } else { loss -= d }
    }
    let mut ag = gain / period as f64;
    let mut al = loss / period as f64;
    out[period] = 100.0 - 100.0 / (1.0 + if al == 0.0 { f64::INFINITY } else { ag / al });
    for i in period + 1..values.len() {
        let d = values[i] - values[i - 1];
        ag = (ag * (period as f64 - 1.0) + d.max(0.0)) / period as f64;
        al = (al * (period as f64 - 1.0) + (-d).max(0.0)) / period as f64;
        out[i] = 100.0 - 100.0 / (1.0 + if al == 0.0 { f64::INFINITY } else { ag / al });
    }
    out
}

pub fn atr(bars: &[Bar], period: usize) -> Vec<f64> {
    let mut out = _nan_vec(bars.len());
    if period == 0 || bars.len() <= period { return out }
    let mut sum = 0.0;
    for i in 1..=period {
        sum += true_range(bars[i - 1], bars[i]);
    }
    let mut a = sum / period as f64;
    out[period] = a;
    for i in period + 1..bars.len() {
        a = (a * (period as f64 - 1.0) + true_range(bars[i - 1], bars[i])) / period as f64;
        out[i] = a;
    }
    out
}

fn true_range(prev: Bar, cur: Bar) -> f64 {
    let hl = cur.high - cur.low;
    let hc = (cur.high - prev.close).abs();
    let lc = (cur.low - prev.close).abs();
    hl.max(hc).max(lc)
}

pub fn highest(values: &[f64], period: usize, from: usize) -> f64 {
    if period == 0 || from < period { return f64::NAN }
    let start = from - period + 1;
    let mut it = values[start..=from].iter();
    let mut mx = *it.next().unwrap();
    for v in it { if *v > mx { mx = *v } }
    mx
}

pub fn lowest(values: &[f64], period: usize, from: usize) -> f64 {
    if period == 0 || from < period { return f64::NAN }
    let start = from - period + 1;
    let mut it = values[start..=from].iter();
    let mut mn = *it.next().unwrap();
    for v in it { if *v < mn { mn = *v } }
    mn
}

/// cruces con lookback de 2 barras
pub fn crosses_above(value: &Vec<f64>, level_fn: &dyn Fn(usize) -> f64, i: usize) -> bool {
    if i < 1 { return false }
    let (a, b, l) = (value[i], value[i - 1], level_fn(i - 1));
    !(a.is_nan() || b.is_nan() || l.is_nan()) && b <= l && a > l
}
