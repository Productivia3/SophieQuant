//! engine/metrics.rs — métricas de rendimiento a partir de trades + equity.

use super::types::{Metrics, Trade};


/// Retornos diarios a partir de las operaciones.
///
/// El Sharpe "por operacion" que se usaba antes **no** es comparable con el de
/// una serie de equity por tiempo (ni con el del tester de MT5). Aqui se
/// construye la curva diaria real y se annualiza con 252 dias, que es lo
/// standard.
pub fn daily_returns(trades: &[Trade], initial: f64) -> Vec<f64> {
    if trades.is_empty() { return vec![] }
    let mut balance = initial;
    let mut por_dia: Vec<(i64, f64)> = Vec::new();
    for t in trades {
        balance += t.profit;
        let dia = t.close_time.div_euclid(86_400);
        match por_dia.last_mut() {
            Some((d, _)) if *d == dia => {
                if let Some(u) = por_dia.last_mut() { u.1 = balance; }
            }
            _ => por_dia.push((dia, balance)),
        }
    }
    let mut rets: Vec<f64> = Vec::with_capacity(por_dia.len());
    let mut anterior = initial;
    for (_, v) in por_dia {
        if anterior.abs() > 1e-9 {
            rets.push((v - anterior) / anterior.abs());
        }
        anterior = v;
    }
    rets
}

/// Sharpe y Sortino anualizados sobre los retornos diarios.
pub fn sharpe_annualized(trades: &[Trade], initial: f64) -> (f64, f64) {
    let rets = daily_returns(trades, initial);
    if rets.len() < 2 { return (0.0, 0.0) }
    let n = rets.len() as f64;
    let media = rets.iter().sum::<f64>() / n;
    let var = rets.iter().map(|r| (r - media) * (r - media)).sum::<f64>() / (n - 1.0);
    let sd = var.max(0.0).sqrt();
    let downside: Vec<f64> = rets.iter().map(|r| (r - media).min(0.0)).collect();
    let var_d = downside.iter().map(|d| d * d).sum::<f64>() / (n - 1.0);
    let sd_d = var_d.max(0.0).sqrt();
    let raiz = 252.0f64.sqrt();
    let sharpe = if sd > 0.0 { media / sd * raiz } else { 0.0 };
    let sortino = if sd_d > 0.0 { media / sd_d * raiz } else { 0.0 };
    ((sharpe * 100.0).round() / 100.0, (sortino * 100.0).round() / 100.0)
}

pub fn compute(trades: &[Trade], equity: &[f64], initial: f64) -> Metrics {
    let mut m = Metrics::default();
    m.n_trades = trades.len() as u32;
    if trades.is_empty() { return m }

    let profits: Vec<f64> = trades.iter().map(|t| t.profit).collect();
    m.gross_profit = profits.iter().filter(|p| **p > 0.0).sum();
    let gross_loss: f64 = profits.iter().filter(|p| **p < 0.0).sum::<f64>().abs();
    m.net_profit = profits.iter().sum::<f64>();
    m.profit_factor = if gross_loss == 0.0 { f64::INFINITY.clamp(0.0, 999.0) } else { m.gross_profit / gross_loss };
    m.win_rate = profits.iter().filter(|p| **p > 0.0).count() as f64 / profits.len() as f64;
    m.avg_trade = m.net_profit / profits.len() as f64;

    // max drawdown % sobre el peak
    let mut peak = initial;
    let mut max_dd_money = 0.0f64;
    for e in equity {
        if *e > peak { peak = *e }
        let dd = peak - *e;
        if dd > max_dd_money { max_dd_money = dd }
    }
    m.max_dd = if peak > 0.0 { max_dd_money / peak * 100.0 } else { 0.0 };
    m.rdd = if max_dd_money > 0.0 { m.net_profit / max_dd_money } else { 0.0 };
    // Sharpe/Sortino anualizados (base diaria): comparables con MT5
    let (sh_a, so_a) = sharpe_annualized(trades, initial);
    m.sharpe_anual = sh_a;
    m.sortino_anual = so_a;

    // sharpe/sortino simplificados con retornos por trade
    let wins: Vec<f64> = profits.iter().filter(|p| **p > 0.0).copied().collect();
    if !wins.is_empty() {
        let mean = profits.iter().sum::<f64>() / profits.len() as f64;
        let var_pos: f64 = profits.iter().map(|p| p.max(0.0) - mean).map(|d| d * d).sum::<f64>() / profits.len() as f64;
        let var: f64 = profits.iter().map(|p| p - mean).map(|d| d * d).sum::<f64>() / profits.len() as f64;
        let sd = var.max(0.0).sqrt();
        let sd_pos = var_pos.max(0.0).sqrt();
        if sd > 0.0 { m.sharpe = mean / sd * (profits.len() as f64).sqrt(); }
        if sd_pos > 0.0 { m.sortino = mean / sd_pos * (profits.len() as f64).sqrt(); }

        // z-score (dependencia serial de wins/losses)
        let n = profits.len() as f64;
        let w = wins.len() as f64;
        let l = n - w;
        if w > 0.0 && l > 0.0 {
            let mut runs = 1;
            for i in 1..profits.len() {
                if (profits[i] > 0.0) != (profits[i - 1] > 0.0) { runs += 1 }
            }
            let exp = 2.0 * w * l / n + 1.0;
            let sd_runs = (2.0 * w * l * (2.0 * w * l - n)) / (n * n * (n - 1.0).max(1.0));
            if sd_runs > 0.0 { m.z_score = (runs as f64 - exp) / sd_runs.max(f64::MIN_POSITIVE).sqrt() * if sd_runs < 1.0 { 1.0 } else { 1.0 }; }
        }
        // r² de la recta de equity (regresión simple)
        let n2 = equity.len();
        if n2 > 2 {
            let xs: Vec<f64> = (0..n2).map(|i| i as f64).collect();
            let mx: f64 = xs.iter().sum::<f64>() / n2 as f64;
            let my: f64 = equity.iter().sum::<f64>() / n2 as f64;
            let sxy: f64 = xs.iter().zip(equity).map(|(x, y)| (x - mx) * (y - my)).sum();
            let sxx: f64 = xs.iter().map(|x| (x - mx) * (x - mx)).sum();
            let syy: f64 = equity.iter().map(|y| (y - my) * (y - my)).sum();
            if sxx > 0.0 && syy > 0.0 {
                let r = sxy / (sxx.sqrt() * syy.sqrt());
                m.r_squared = r * r;
            }
        }
    }
    m
}

/// Serie de equity muestreada (máx 500 puntos) para pintar el gráfico.
pub fn equity_series(trades: &[Trade], initial: f64, max_points: usize) -> Vec<f64> {
    let mut out = vec![initial];
    let mut bal = initial;
    for t in trades {
        bal += t.profit;
        out.push(bal);
    }
    if out.len() > max_points {
        let step = out.len() as f64 / max_points as f64;
        out = (0..max_points).map(|i| out[((i as f64 * step) as usize).min(out.len() - 1)]).collect();
    }
    out
}
