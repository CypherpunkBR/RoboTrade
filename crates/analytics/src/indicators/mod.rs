//! # Indicadores Técnicos
//!
//! Módulo de indicadores técnicos para análise de mercado.
//!
//! ## Indicadores Implementados
//!
//! - **SMA** - Simple Moving Average (Média Móvel Simples)
//! - **EMA** - Exponential Moving Average (Média Móvel Exponencial)
//! - **RSI** - Relative Strength Index (Índice de Força Relativa)
//! - **MACD** - Moving Average Convergence Divergence
//! - **Bollinger Bands** - Bandas de Bollinger
//! - **ATR** - Average True Range

mod sma;
mod ema;
mod rsi;
mod macd;
mod bollinger;
mod atr;

pub use sma::*;
pub use ema::*;
pub use rsi::*;
pub use macd::*;
pub use bollinger::*;
pub use atr::*;

use robotrade_core::entities::Candle;
use robotrade_core::error::{AnalyticsError, AnalyticsResult};
use rust_decimal::Decimal;

/// Trait para indicadores técnicos
pub trait Indicator {
    /// Nome do indicador
    fn name(&self) -> &'static str;

    /// Período usado pelo indicador
    fn period(&self) -> usize;

    /// Calcula o valor do indicador para uma série de preços
    fn calculate(&self, prices: &[Decimal]) -> AnalyticsResult<Vec<Option<Decimal>>>;
}

/// Trait para indicadores baseados em candles (OHLCV)
pub trait CandleIndicator {
    /// Nome do indicador
    fn name(&self) -> &'static str;

    /// Período usado pelo indicador
    fn period(&self) -> usize;

    /// Calcula o valor do indicador para uma série de candles
    fn calculate_from_candles(&self, candles: &[Candle]) -> AnalyticsResult<Vec<Option<Decimal>>>;
}

/// Extrai preços de fechamento de uma lista de candles
pub fn extract_close_prices(candles: &[Candle]) -> Vec<Decimal> {
    candles.iter().map(|c| c.close).collect()
}

/// Extrai preços de abertura de uma lista de candles
pub fn extract_open_prices(candles: &[Candle]) -> Vec<Decimal> {
    candles.iter().map(|c| c.open).collect()
}

/// Extrai preços máximos de uma lista de candles
pub fn extract_high_prices(candles: &[Candle]) -> Vec<Decimal> {
    candles.iter().map(|c| c.high).collect()
}

/// Extrai preços mínimos de uma lista de candles
pub fn extract_low_prices(candles: &[Candle]) -> Vec<Decimal> {
    candles.iter().map(|c| c.low).collect()
}

/// Valida se há dados suficientes para o cálculo
pub fn validate_data_length(data_len: usize, period: usize) -> AnalyticsResult<()> {
    if data_len < period {
        return Err(AnalyticsError::InsufficientDataForCalculation {
            calculation: "indicator".to_string(),
            min_required: period,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use rust_decimal_macros::dec;

    pub fn create_test_candles(prices: &[Decimal]) -> Vec<Candle> {
        prices
            .iter()
            .map(|&price| Candle::new(
                price,
                price + dec!(1),
                price - dec!(1),
                price,
                dec!(1000),
                Utc::now(),
                Utc::now(),
            ))
            .collect()
    }
}
