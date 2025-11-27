//! # Position Manager
//!
//! Gerenciador de posições responsável por:
//! - Abrir e fechar posições
//! - Calcular P&L em tempo real
//! - Gerenciar Stop Loss e Take Profit
//! - Trailing stop

use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

use robotrade_core::entities::{
    OrderId, Position, PositionId, PositionSide, PositionStatus, Trade, TradeCloseReason,
};

/// Evento emitido pelo Position Manager
#[derive(Debug, Clone)]
pub enum PositionEvent {
    /// Posição foi aberta
    Opened(Position),
    /// Posição foi atualizada (P&L, preço)
    Updated(Position),
    /// Posição foi fechada
    Closed { position: Position, trade: Trade },
    /// Stop Loss foi acionado
    StopLossTriggered(PositionId),
    /// Take Profit foi acionado
    TakeProfitTriggered(PositionId),
    /// Trailing Stop foi atualizado
    TrailingStopUpdated {
        position_id: PositionId,
        new_stop: Decimal,
    },
    /// Erro ao gerenciar posição
    Error { position_id: PositionId, message: String },
}

/// Configuração do Position Manager
#[derive(Debug, Clone)]
pub struct PositionManagerConfig {
    /// Ativar trailing stop automático
    pub enable_trailing_stop: bool,
    /// Distância do trailing stop em percentual
    pub trailing_stop_pct: Decimal,
    /// Ativar break-even automático
    pub enable_break_even: bool,
    /// Lucro mínimo para ativar break-even (em percentual)
    pub break_even_trigger_pct: Decimal,
    /// Distância do break-even do entry price
    pub break_even_offset_pct: Decimal,
}

impl Default for PositionManagerConfig {
    fn default() -> Self {
        Self {
            enable_trailing_stop: false,
            trailing_stop_pct: dec!(2.0),
            enable_break_even: true,
            break_even_trigger_pct: dec!(1.5),
            break_even_offset_pct: dec!(0.1),
        }
    }
}

/// Estado interno de uma posição gerenciada
#[derive(Debug, Clone)]
struct ManagedPosition {
    position: Position,
    highest_price: Decimal,  // Para trailing stop em long
    lowest_price: Decimal,   // Para trailing stop em short
    break_even_activated: bool,
    original_stop_loss: Option<Decimal>,
}

/// Position Manager
pub struct PositionManager {
    config: PositionManagerConfig,
    positions: Arc<RwLock<HashMap<PositionId, ManagedPosition>>>,
    event_tx: tokio::sync::broadcast::Sender<PositionEvent>,
}

impl PositionManager {
    /// Cria um novo Position Manager
    pub fn new(config: PositionManagerConfig) -> Self {
        let (event_tx, _) = tokio::sync::broadcast::channel(100);
        Self {
            config,
            positions: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
        }
    }

    /// Retorna um receiver para eventos
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<PositionEvent> {
        self.event_tx.subscribe()
    }

    /// Abre uma nova posição
    pub async fn open_position(&self, position: Position) -> Result<(), String> {
        let position_id = position.id.clone();

        let managed = ManagedPosition {
            highest_price: position.current_price,
            lowest_price: position.current_price,
            break_even_activated: false,
            original_stop_loss: position.stop_loss_price,
            position: position.clone(),
        };

        {
            let mut positions = self.positions.write().await;
            if positions.contains_key(&position_id) {
                return Err(format!("Posição {} já existe", position_id));
            }
            positions.insert(position_id.clone(), managed);
        }

        info!("Posição aberta: {} {} {} @ {}",
            position_id,
            position.side,
            position.symbol,
            position.entry_price
        );

        let _ = self.event_tx.send(PositionEvent::Opened(position));
        Ok(())
    }

    /// Atualiza o preço de mercado de uma posição
    pub async fn update_price(&self, position_id: &PositionId, current_price: Decimal) -> Option<PositionEvent> {
        let mut positions = self.positions.write().await;

        let managed = positions.get_mut(position_id)?;
        managed.position.current_price = current_price;

        // Atualiza highest/lowest para trailing stop
        if current_price > managed.highest_price {
            managed.highest_price = current_price;
        }
        if current_price < managed.lowest_price {
            managed.lowest_price = current_price;
        }

        // Recalcula P&L
        let pnl = self.calculate_pnl(&managed.position);
        managed.position.unrealized_pnl = pnl;

        // Verifica Stop Loss
        if let Some(stop_loss) = managed.position.stop_loss_price {
            let triggered = match managed.position.side {
                PositionSide::Long => current_price <= stop_loss,
                PositionSide::Short => current_price >= stop_loss,
            };

            if triggered {
                debug!("Stop Loss acionado para posição {}", position_id);
                return Some(PositionEvent::StopLossTriggered(position_id.clone()));
            }
        }

        // Verifica Take Profit
        if let Some(take_profit) = managed.position.take_profit_price {
            let triggered = match managed.position.side {
                PositionSide::Long => current_price >= take_profit,
                PositionSide::Short => current_price <= take_profit,
            };

            if triggered {
                debug!("Take Profit acionado para posição {}", position_id);
                return Some(PositionEvent::TakeProfitTriggered(position_id.clone()));
            }
        }

        // Atualiza trailing stop se configurado
        if self.config.enable_trailing_stop {
            if let Some(event) = self.update_trailing_stop(&mut managed.position, managed.highest_price, managed.lowest_price) {
                return Some(event);
            }
        }

        // Ativa break-even se configurado
        if self.config.enable_break_even && !managed.break_even_activated {
            let pnl_pct = self.calculate_pnl_pct(&managed.position);
            if pnl_pct >= self.config.break_even_trigger_pct {
                self.activate_break_even(managed);
            }
        }

        Some(PositionEvent::Updated(managed.position.clone()))
    }

    /// Fecha uma posição
    pub async fn close_position(
        &self,
        position_id: &PositionId,
        exit_price: Decimal,
        reason: TradeCloseReason,
    ) -> Option<Trade> {
        let mut positions = self.positions.write().await;
        let managed = positions.remove(position_id)?;

        let mut position = managed.position;
        position.current_price = exit_price;
        position.status = PositionStatus::Closed;
        position.closed_at = Some(Utc::now());

        // Calcula P&L final
        let fees = position.quantity * exit_price * dec!(0.001); // 0.1% fee estimado

        // Usa o método do core para criar o Trade corretamente
        let trade = Trade::from_closed_position(
            position.exchange,
            position.symbol.clone(),
            position.side,
            position_id.clone(),
            None, // signal_id
            position.entry_order_id.clone(),
            OrderId::new(), // exit_order_id - gerado aqui
            position.quantity,
            position.entry_price,
            exit_price,
            fees,
            position.leverage,
            reason.clone(),
            position.opened_at,
        );

        info!(
            "Posição fechada: {} - P&L: {} ({:?})",
            position_id, trade.net_pnl, reason
        );

        let _ = self.event_tx.send(PositionEvent::Closed {
            position: position.clone(),
            trade: trade.clone(),
        });

        Some(trade)
    }

    /// Retorna todas as posições abertas
    pub async fn get_open_positions(&self) -> Vec<Position> {
        let positions = self.positions.read().await;
        positions.values().map(|m| m.position.clone()).collect()
    }

    /// Retorna uma posição específica
    pub async fn get_position(&self, position_id: &PositionId) -> Option<Position> {
        let positions = self.positions.read().await;
        positions.get(position_id).map(|m| m.position.clone())
    }

    /// Atualiza o Stop Loss de uma posição
    pub async fn update_stop_loss(&self, position_id: &PositionId, new_stop: Decimal) -> bool {
        let mut positions = self.positions.write().await;
        if let Some(managed) = positions.get_mut(position_id) {
            managed.position.stop_loss_price = Some(new_stop);
            info!("Stop Loss atualizado para posição {}: {}", position_id, new_stop);
            true
        } else {
            false
        }
    }

    /// Atualiza o Take Profit de uma posição
    pub async fn update_take_profit(&self, position_id: &PositionId, new_tp: Decimal) -> bool {
        let mut positions = self.positions.write().await;
        if let Some(managed) = positions.get_mut(position_id) {
            managed.position.take_profit_price = Some(new_tp);
            info!("Take Profit atualizado para posição {}: {}", position_id, new_tp);
            true
        } else {
            false
        }
    }

    /// Calcula o P&L de uma posição
    fn calculate_pnl(&self, position: &Position) -> Decimal {
        let price_diff = position.current_price - position.entry_price;
        let direction = match position.side {
            PositionSide::Long => Decimal::ONE,
            PositionSide::Short => -Decimal::ONE,
        };
        price_diff * position.quantity * direction * Decimal::from(position.leverage)
    }

    /// Calcula o P&L em percentual
    fn calculate_pnl_pct(&self, position: &Position) -> Decimal {
        if position.entry_price == Decimal::ZERO {
            return Decimal::ZERO;
        }
        let price_diff = position.current_price - position.entry_price;
        let direction = match position.side {
            PositionSide::Long => Decimal::ONE,
            PositionSide::Short => -Decimal::ONE,
        };
        (price_diff / position.entry_price) * dec!(100) * direction * Decimal::from(position.leverage)
    }

    /// Atualiza o trailing stop
    fn update_trailing_stop(
        &self,
        position: &mut Position,
        highest: Decimal,
        lowest: Decimal,
    ) -> Option<PositionEvent> {
        let trailing_distance = match position.side {
            PositionSide::Long => highest * self.config.trailing_stop_pct / dec!(100),
            PositionSide::Short => lowest * self.config.trailing_stop_pct / dec!(100),
        };

        let new_stop = match position.side {
            PositionSide::Long => highest - trailing_distance,
            PositionSide::Short => lowest + trailing_distance,
        };

        // Só atualiza se o novo stop for melhor que o atual
        let should_update = match position.stop_loss_price {
            Some(current_stop) => match position.side {
                PositionSide::Long => new_stop > current_stop,
                PositionSide::Short => new_stop < current_stop,
            },
            None => true,
        };

        if should_update {
            position.stop_loss_price = Some(new_stop);
            return Some(PositionEvent::TrailingStopUpdated {
                position_id: position.id.clone(),
                new_stop,
            });
        }

        None
    }

    /// Ativa o break-even
    fn activate_break_even(&self, managed: &mut ManagedPosition) {
        let offset = managed.position.entry_price * self.config.break_even_offset_pct / dec!(100);

        let break_even_stop = match managed.position.side {
            PositionSide::Long => managed.position.entry_price + offset,
            PositionSide::Short => managed.position.entry_price - offset,
        };

        // Só atualiza se for melhor que o stop atual
        let should_update = match managed.position.stop_loss_price {
            Some(current_stop) => match managed.position.side {
                PositionSide::Long => break_even_stop > current_stop,
                PositionSide::Short => break_even_stop < current_stop,
            },
            None => true,
        };

        if should_update {
            managed.position.stop_loss_price = Some(break_even_stop);
            managed.break_even_activated = true;
            info!(
                "Break-even ativado para posição {}: stop movido para {}",
                managed.position.id, break_even_stop
            );
        }
    }

    /// Número de posições abertas
    pub async fn position_count(&self) -> usize {
        let positions = self.positions.read().await;
        positions.len()
    }

    /// Exposição total (soma do valor nocional de todas as posições)
    pub async fn total_exposure(&self) -> Decimal {
        let positions = self.positions.read().await;
        positions.values()
            .map(|m| m.position.quantity * m.position.current_price * Decimal::from(m.position.leverage))
            .sum()
    }
}

impl Default for PositionManager {
    fn default() -> Self {
        Self::new(PositionManagerConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use robotrade_core::entities::ExchangeId;

    fn create_test_position(side: PositionSide, entry_price: Decimal) -> Position {
        Position::from_entry_order(
            ExchangeId::BinanceFutures,
            "BTCUSDT".to_string(),
            side,
            dec!(0.1),
            entry_price,
            10, // leverage
            OrderId::new(),
        )
    }

    #[tokio::test]
    async fn test_open_and_close_position() {
        let manager = PositionManager::default();
        let position = create_test_position(PositionSide::Long, dec!(50000));
        let position_id = position.id.clone();

        manager.open_position(position).await.unwrap();

        assert_eq!(manager.position_count().await, 1);

        let trade = manager.close_position(
            &position_id,
            dec!(51000),
            TradeCloseReason::Manual { nota: None }
        ).await;

        assert!(trade.is_some());
        let trade = trade.unwrap();
        assert!(trade.net_pnl > Decimal::ZERO);
        assert_eq!(manager.position_count().await, 0);
    }

    #[tokio::test]
    async fn test_pnl_calculation_long() {
        let manager = PositionManager::default();
        let position = create_test_position(PositionSide::Long, dec!(50000));
        let position_id = position.id.clone();

        manager.open_position(position).await.unwrap();

        // Preço sobe 2%
        manager.update_price(&position_id, dec!(51000)).await;

        let pos = manager.get_position(&position_id).await.unwrap();
        // 0.1 BTC * 1000 USD diff * 10x leverage = 1000 USD profit
        assert!(pos.unrealized_pnl > Decimal::ZERO);
    }

    #[tokio::test]
    async fn test_pnl_calculation_short() {
        let manager = PositionManager::default();
        let position = create_test_position(PositionSide::Short, dec!(50000));
        let position_id = position.id.clone();

        manager.open_position(position).await.unwrap();

        // Preço cai 2%
        manager.update_price(&position_id, dec!(49000)).await;

        let pos = manager.get_position(&position_id).await.unwrap();
        // Short lucra quando preço cai
        assert!(pos.unrealized_pnl > Decimal::ZERO);
    }

    #[tokio::test]
    async fn test_stop_loss_trigger() {
        let manager = PositionManager::default();
        let mut position = create_test_position(PositionSide::Long, dec!(50000));
        position.stop_loss_price = Some(dec!(49000));
        let position_id = position.id.clone();

        manager.open_position(position).await.unwrap();

        // Preço cai abaixo do stop loss
        let event = manager.update_price(&position_id, dec!(48500)).await;

        assert!(matches!(event, Some(PositionEvent::StopLossTriggered(_))));
    }

    #[tokio::test]
    async fn test_take_profit_trigger() {
        let manager = PositionManager::default();
        let mut position = create_test_position(PositionSide::Long, dec!(50000));
        position.take_profit_price = Some(dec!(52000));
        let position_id = position.id.clone();

        manager.open_position(position).await.unwrap();

        // Preço sobe acima do take profit
        let event = manager.update_price(&position_id, dec!(52500)).await;

        assert!(matches!(event, Some(PositionEvent::TakeProfitTriggered(_))));
    }
}
