//! Servico de dados de mercado
//!
//! Gerencia dados historicos de precos, alertas e indicadores.

use parking_lot::RwLock;
use robotrade_core::entities::{Candle, PriceAlert, PriceAlertId};
use robotrade_exchange_gateways::BinanceFuturesClient;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

/// Servico de dados de mercado
pub struct MarketDataService {
    /// Cliente Binance para dados publicos
    binance: BinanceFuturesClient,
    /// Cache de candles por simbolo e timeframe
    candles_cache: Arc<RwLock<HashMap<String, Vec<Candle>>>>,
    /// Alertas de preco ativos
    price_alerts: Arc<RwLock<HashMap<String, PriceAlert>>>,
    /// Ultimo preco conhecido por simbolo
    last_prices: Arc<RwLock<HashMap<String, Decimal>>>,
}

impl MarketDataService {
    /// Cria um novo servico
    pub fn new() -> Self {
        // Usa testnet por padrao para dados publicos (nao precisa de API key)
        let binance = BinanceFuturesClient::testnet("", "");

        Self {
            binance,
            candles_cache: Arc::new(RwLock::new(HashMap::new())),
            price_alerts: Arc::new(RwLock::new(HashMap::new())),
            last_prices: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Busca candles recentes
    pub async fn get_klines(
        &self,
        symbol: &str,
        interval: &str,
        limit: Option<u32>,
    ) -> Result<Vec<Candle>, String> {
        self.binance
            .get_klines(symbol, interval, limit)
            .await
            .map_err(|e| format!("Erro ao buscar klines: {}", e))
    }

    /// Busca candles com range de tempo
    pub async fn get_klines_range(
        &self,
        symbol: &str,
        interval: &str,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<u32>,
    ) -> Result<Vec<Candle>, String> {
        self.binance
            .get_klines_range(symbol, interval, start_time, end_time, limit)
            .await
            .map_err(|e| format!("Erro ao buscar klines: {}", e))
    }

    /// Busca todo o historico de klines
    pub async fn get_all_klines(
        &self,
        symbol: &str,
        interval: &str,
        start_time: i64,
        end_time: Option<i64>,
    ) -> Result<Vec<Candle>, String> {
        info!(
            symbol = %symbol,
            interval = %interval,
            start_time = %start_time,
            "Buscando historico completo de klines"
        );

        self.binance
            .get_all_klines(symbol, interval, start_time, end_time)
            .await
            .map_err(|e| format!("Erro ao buscar historico: {}", e))
    }

    /// Atualiza cache de candles
    pub fn update_candles_cache(&self, key: String, candles: Vec<Candle>) {
        self.candles_cache.write().insert(key, candles);
    }

    /// Busca candles do cache
    pub fn get_cached_candles(&self, key: &str) -> Option<Vec<Candle>> {
        self.candles_cache.read().get(key).cloned()
    }

    // =========================================================================
    // Alertas de Preco
    // =========================================================================

    /// Cria um novo alerta de preco
    pub fn create_alert(&self, alert: PriceAlert) -> PriceAlertId {
        let id = alert.id;
        self.price_alerts
            .write()
            .insert(id.to_string(), alert);
        info!(alert_id = %id, "Alerta de preco criado");
        id
    }

    /// Remove um alerta
    pub fn remove_alert(&self, id: &str) -> bool {
        let removed = self.price_alerts.write().remove(id).is_some();
        if removed {
            debug!(alert_id = %id, "Alerta de preco removido");
        }
        removed
    }

    /// Lista todos os alertas
    pub fn list_alerts(&self) -> Vec<PriceAlert> {
        self.price_alerts.read().values().cloned().collect()
    }

    /// Lista alertas por simbolo
    pub fn list_alerts_by_symbol(&self, symbol: &str) -> Vec<PriceAlert> {
        self.price_alerts
            .read()
            .values()
            .filter(|a| a.symbol == symbol)
            .cloned()
            .collect()
    }

    /// Busca um alerta por ID
    pub fn get_alert(&self, id: &str) -> Option<PriceAlert> {
        self.price_alerts.read().get(id).cloned()
    }

    /// Atualiza um alerta
    pub fn update_alert(&self, id: &str, alert: PriceAlert) -> bool {
        let mut alerts = self.price_alerts.write();
        if alerts.contains_key(id) {
            alerts.insert(id.to_string(), alert);
            true
        } else {
            false
        }
    }

    /// Verifica alertas e retorna os que devem disparar
    pub fn check_alerts(&self, symbol: &str, current_price: Decimal) -> Vec<PriceAlert> {
        let mut triggered = Vec::new();
        let previous_price = self.last_prices.read().get(symbol).copied();

        // Atualiza ultimo preco
        self.last_prices
            .write()
            .insert(symbol.to_string(), current_price);

        let mut alerts = self.price_alerts.write();
        for alert in alerts.values_mut() {
            if alert.symbol != symbol {
                continue;
            }

            if alert.should_trigger(current_price, previous_price) {
                alert.trigger();
                triggered.push(alert.clone());
                info!(
                    alert_id = %alert.id,
                    symbol = %symbol,
                    price = %current_price,
                    "Alerta de preco disparado"
                );
            }

            alert.update_last_price(current_price);
        }

        triggered
    }

    /// Desabilita um alerta
    pub fn disable_alert(&self, id: &str) -> bool {
        if let Some(alert) = self.price_alerts.write().get_mut(id) {
            alert.disable();
            true
        } else {
            false
        }
    }

    /// Reativa um alerta
    pub fn reactivate_alert(&self, id: &str) -> bool {
        if let Some(alert) = self.price_alerts.write().get_mut(id) {
            alert.reactivate();
            true
        } else {
            false
        }
    }

    // =========================================================================
    // Indicadores Tecnicos
    // =========================================================================

    /// Calcula SMA (Simple Moving Average)
    pub fn calculate_sma(candles: &[Candle], period: usize) -> Vec<Option<Decimal>> {
        if candles.len() < period {
            return vec![None; candles.len()];
        }

        let mut result = vec![None; period - 1];

        for i in (period - 1)..candles.len() {
            let sum: Decimal = candles[(i + 1 - period)..=i]
                .iter()
                .map(|c| c.close)
                .sum();
            result.push(Some(sum / Decimal::from(period)));
        }

        result
    }

    /// Calcula EMA (Exponential Moving Average)
    pub fn calculate_ema(candles: &[Candle], period: usize) -> Vec<Option<Decimal>> {
        if candles.len() < period {
            return vec![None; candles.len()];
        }

        let multiplier = Decimal::from(2) / Decimal::from(period + 1);
        let mut result = vec![None; period - 1];

        // Primeira EMA e a SMA
        let first_sma: Decimal = candles[..period]
            .iter()
            .map(|c| c.close)
            .sum::<Decimal>()
            / Decimal::from(period);

        result.push(Some(first_sma));
        let mut prev_ema = first_sma;

        for candle in candles.iter().skip(period) {
            let ema = (candle.close - prev_ema) * multiplier + prev_ema;
            result.push(Some(ema));
            prev_ema = ema;
        }

        result
    }

    /// Calcula Bollinger Bands
    pub fn calculate_bollinger_bands(
        candles: &[Candle],
        period: usize,
        std_dev: Decimal,
    ) -> Vec<(Option<Decimal>, Option<Decimal>, Option<Decimal>)> {
        let sma = Self::calculate_sma(candles, period);

        let mut result = vec![(None, None, None); period - 1];

        for i in (period - 1)..candles.len() {
            if let Some(middle) = sma[i] {
                let closes: Vec<Decimal> = candles[(i + 1 - period)..=i]
                    .iter()
                    .map(|c| c.close)
                    .collect();

                let variance: Decimal = closes
                    .iter()
                    .map(|c| {
                        let diff = *c - middle;
                        diff * diff
                    })
                    .sum::<Decimal>()
                    / Decimal::from(period);

                // Aproximacao de raiz quadrada usando Newton-Raphson
                let std = Self::sqrt_approx(variance);

                let upper = middle + std * std_dev;
                let lower = middle - std * std_dev;

                result.push((Some(lower), Some(middle), Some(upper)));
            } else {
                result.push((None, None, None));
            }
        }

        result
    }

    /// Aproximacao de raiz quadrada
    fn sqrt_approx(n: Decimal) -> Decimal {
        if n <= Decimal::ZERO {
            return Decimal::ZERO;
        }

        let mut x = n;
        let two = Decimal::from(2);

        // Newton-Raphson iterations
        for _ in 0..10 {
            x = (x + n / x) / two;
        }

        x
    }
}

impl Default for MarketDataService {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for MarketDataService {
    fn clone(&self) -> Self {
        Self {
            binance: BinanceFuturesClient::testnet("", ""),
            candles_cache: Arc::clone(&self.candles_cache),
            price_alerts: Arc::clone(&self.price_alerts),
            last_prices: Arc::clone(&self.last_prices),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn create_test_candles(prices: &[Decimal]) -> Vec<Candle> {
        prices
            .iter()
            .enumerate()
            .map(|(i, &price)| Candle {
                open: price,
                high: price + dec!(1),
                low: price - dec!(1),
                close: price,
                volume: dec!(100),
                quote_volume: None,
                trade_count: None,
                open_time: chrono::Utc::now(),
                close_time: chrono::Utc::now(),
            })
            .collect()
    }

    #[test]
    fn test_sma_calculation() {
        let candles = create_test_candles(&[
            dec!(10), dec!(11), dec!(12), dec!(13), dec!(14),
            dec!(15), dec!(16), dec!(17), dec!(18), dec!(19),
        ]);

        let sma = MarketDataService::calculate_sma(&candles, 5);

        // Primeiros 4 devem ser None
        assert!(sma[0].is_none());
        assert!(sma[3].is_none());

        // SMA(5) no indice 4 = (10+11+12+13+14)/5 = 12
        assert_eq!(sma[4], Some(dec!(12)));

        // SMA(5) no indice 5 = (11+12+13+14+15)/5 = 13
        assert_eq!(sma[5], Some(dec!(13)));
    }

    #[test]
    fn test_ema_calculation() {
        let candles = create_test_candles(&[
            dec!(10), dec!(11), dec!(12), dec!(13), dec!(14),
            dec!(15), dec!(16), dec!(17), dec!(18), dec!(19),
        ]);

        let ema = MarketDataService::calculate_ema(&candles, 5);

        assert!(ema[0].is_none());
        assert!(ema[3].is_none());
        assert!(ema[4].is_some());
        assert!(ema[9].is_some());
    }
}
