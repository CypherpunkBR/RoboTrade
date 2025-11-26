//! Entidade Trade (Trade executado/histórico)

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{ExchangeId, OrderId, PositionId, PositionSide, SignalId};

/// ID único de um trade
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TradeId(pub Uuid);

impl TradeId {
    /// Cria um novo ID de trade
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for TradeId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TradeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Motivo do fechamento do trade
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TradeCloseReason {
    /// Stop loss atingido
    StopLoss,
    /// Take profit atingido
    TakeProfit,
    /// Sinal de saída da estratégia
    Signal { signal_id: SignalId },
    /// Fechamento manual pelo usuário
    Manual { nota: Option<String> },
    /// Liquidação (margin call)
    Liquidation,
    /// Tempo limite atingido
    Timeout,
    /// Fechamento de emergência
    Emergency,
}

/// Trade completo (entrada + saída)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    /// ID único do trade
    pub id: TradeId,
    /// Exchange onde foi executado
    pub exchange: ExchangeId,
    /// Símbolo do par
    pub symbol: String,
    /// Lado (long/short)
    pub side: PositionSide,
    /// ID da posição relacionada
    pub position_id: PositionId,
    /// ID do sinal que originou o trade
    pub signal_id: Option<SignalId>,
    /// ID da ordem de entrada
    pub entry_order_id: OrderId,
    /// ID da ordem de saída
    pub exit_order_id: OrderId,
    /// Quantidade negociada
    pub quantity: Decimal,
    /// Preço de entrada
    pub entry_price: Decimal,
    /// Preço de saída
    pub exit_price: Decimal,
    /// P&L bruto (sem taxas)
    pub gross_pnl: Decimal,
    /// Total de taxas pagas
    pub total_fees: Decimal,
    /// P&L líquido (com taxas)
    pub net_pnl: Decimal,
    /// P&L em percentual
    pub pnl_pct: Decimal,
    /// ROI (baseado na margem)
    pub roi_pct: Decimal,
    /// Alavancagem usada
    pub leverage: u32,
    /// Motivo do fechamento
    pub close_reason: TradeCloseReason,
    /// Duração do trade em segundos
    pub duration_seconds: i64,
    /// Timestamp de entrada
    pub entered_at: DateTime<Utc>,
    /// Timestamp de saída
    pub exited_at: DateTime<Utc>,
    /// Metadados adicionais
    pub metadata: Option<serde_json::Value>,
}

impl Trade {
    /// Cria um novo trade a partir de uma posição fechada
    pub fn from_closed_position(
        exchange: ExchangeId,
        symbol: String,
        side: PositionSide,
        position_id: PositionId,
        signal_id: Option<SignalId>,
        entry_order_id: OrderId,
        exit_order_id: OrderId,
        quantity: Decimal,
        entry_price: Decimal,
        exit_price: Decimal,
        total_fees: Decimal,
        leverage: u32,
        close_reason: TradeCloseReason,
        entered_at: DateTime<Utc>,
    ) -> Self {
        let exited_at = Utc::now();
        let duration_seconds = (exited_at - entered_at).num_seconds();

        // Calcula P&L baseado no lado
        let price_diff = match side {
            PositionSide::Long => exit_price - entry_price,
            PositionSide::Short => entry_price - exit_price,
        };

        let gross_pnl = price_diff * quantity;
        let net_pnl = gross_pnl - total_fees;

        // Calcula percentuais
        let pnl_pct = if entry_price != Decimal::ZERO {
            (price_diff / entry_price) * Decimal::from(100)
        } else {
            Decimal::ZERO
        };

        // ROI considera alavancagem
        let roi_pct = pnl_pct * Decimal::from(leverage);

        Self {
            id: TradeId::new(),
            exchange,
            symbol,
            side,
            position_id,
            signal_id,
            entry_order_id,
            exit_order_id,
            quantity,
            entry_price,
            exit_price,
            gross_pnl,
            total_fees,
            net_pnl,
            pnl_pct,
            roi_pct,
            leverage,
            close_reason,
            duration_seconds,
            entered_at,
            exited_at,
            metadata: None,
        }
    }

    /// Verifica se o trade foi lucrativo
    pub fn is_profitable(&self) -> bool {
        self.net_pnl > Decimal::ZERO
    }

    /// Retorna a duração formatada
    pub fn duration_formatted(&self) -> String {
        let hours = self.duration_seconds / 3600;
        let minutes = (self.duration_seconds % 3600) / 60;
        let seconds = self.duration_seconds % 60;

        if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, seconds)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds)
        } else {
            format!("{}s", seconds)
        }
    }
}

/// Resumo estatístico de trades
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TradeStats {
    /// Total de trades
    pub total_trades: u32,
    /// Trades vencedores
    pub winning_trades: u32,
    /// Trades perdedores
    pub losing_trades: u32,
    /// Taxa de acerto (win rate)
    pub win_rate_pct: Decimal,
    /// P&L total
    pub total_pnl: Decimal,
    /// Lucro bruto (soma dos trades positivos)
    pub gross_profit: Decimal,
    /// Perda bruta (soma dos trades negativos)
    pub gross_loss: Decimal,
    /// Profit factor (lucro bruto / perda bruta)
    pub profit_factor: Decimal,
    /// Média de lucro por trade vencedor
    pub avg_win: Decimal,
    /// Média de perda por trade perdedor
    pub avg_loss: Decimal,
    /// Maior lucro
    pub largest_win: Decimal,
    /// Maior perda
    pub largest_loss: Decimal,
    /// Sequência atual de vitórias
    pub current_win_streak: u32,
    /// Sequência atual de derrotas
    pub current_loss_streak: u32,
    /// Maior sequência de vitórias
    pub max_win_streak: u32,
    /// Maior sequência de derrotas
    pub max_loss_streak: u32,
    /// Duração média dos trades (segundos)
    pub avg_duration_seconds: i64,
    /// Total de taxas pagas
    pub total_fees: Decimal,
}

impl TradeStats {
    /// Calcula estatísticas a partir de uma lista de trades
    pub fn from_trades(trades: &[Trade]) -> Self {
        if trades.is_empty() {
            return Self::default();
        }

        let total_trades = trades.len() as u32;
        let mut winning_trades = 0u32;
        let mut losing_trades = 0u32;
        let mut gross_profit = Decimal::ZERO;
        let mut gross_loss = Decimal::ZERO;
        let mut largest_win = Decimal::ZERO;
        let mut largest_loss = Decimal::ZERO;
        let mut total_duration = 0i64;
        let mut total_fees = Decimal::ZERO;

        let mut current_win_streak = 0u32;
        let mut current_loss_streak = 0u32;
        let mut max_win_streak = 0u32;
        let mut max_loss_streak = 0u32;

        for trade in trades {
            total_duration += trade.duration_seconds;
            total_fees += trade.total_fees;

            if trade.is_profitable() {
                winning_trades += 1;
                gross_profit += trade.net_pnl;

                if trade.net_pnl > largest_win {
                    largest_win = trade.net_pnl;
                }

                current_win_streak += 1;
                current_loss_streak = 0;

                if current_win_streak > max_win_streak {
                    max_win_streak = current_win_streak;
                }
            } else {
                losing_trades += 1;
                gross_loss += trade.net_pnl.abs();

                if trade.net_pnl.abs() > largest_loss {
                    largest_loss = trade.net_pnl.abs();
                }

                current_loss_streak += 1;
                current_win_streak = 0;

                if current_loss_streak > max_loss_streak {
                    max_loss_streak = current_loss_streak;
                }
            }
        }

        let total_pnl = gross_profit - gross_loss;

        let win_rate_pct = if total_trades > 0 {
            Decimal::from(winning_trades) / Decimal::from(total_trades) * Decimal::from(100)
        } else {
            Decimal::ZERO
        };

        let profit_factor = if gross_loss > Decimal::ZERO {
            gross_profit / gross_loss
        } else if gross_profit > Decimal::ZERO {
            Decimal::from(999) // Sem perdas = profit factor muito alto
        } else {
            Decimal::ZERO
        };

        let avg_win = if winning_trades > 0 {
            gross_profit / Decimal::from(winning_trades)
        } else {
            Decimal::ZERO
        };

        let avg_loss = if losing_trades > 0 {
            gross_loss / Decimal::from(losing_trades)
        } else {
            Decimal::ZERO
        };

        let avg_duration_seconds = if total_trades > 0 {
            total_duration / total_trades as i64
        } else {
            0
        };

        Self {
            total_trades,
            winning_trades,
            losing_trades,
            win_rate_pct,
            total_pnl,
            gross_profit,
            gross_loss,
            profit_factor,
            avg_win,
            avg_loss,
            largest_win,
            largest_loss,
            current_win_streak,
            current_loss_streak,
            max_win_streak,
            max_loss_streak,
            avg_duration_seconds,
            total_fees,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn create_test_trade(net_pnl: Decimal) -> Trade {
        let is_winning = net_pnl > Decimal::ZERO;

        Trade {
            id: TradeId::new(),
            exchange: ExchangeId::Paper,
            symbol: "BTCUSDT".to_string(),
            side: PositionSide::Long,
            position_id: PositionId::new(),
            signal_id: None,
            entry_order_id: OrderId::new(),
            exit_order_id: OrderId::new(),
            quantity: dec!(0.1),
            entry_price: dec!(40000),
            exit_price: if is_winning {
                dec!(41000)
            } else {
                dec!(39000)
            },
            gross_pnl: net_pnl + dec!(5),
            total_fees: dec!(5),
            net_pnl,
            pnl_pct: dec!(2.5),
            roi_pct: dec!(12.5),
            leverage: 5,
            close_reason: TradeCloseReason::TakeProfit,
            duration_seconds: 3600,
            entered_at: Utc::now() - chrono::Duration::hours(1),
            exited_at: Utc::now(),
            metadata: None,
        }
    }

    #[test]
    fn test_trade_stats() {
        let trades = vec![
            create_test_trade(dec!(100)),
            create_test_trade(dec!(50)),
            create_test_trade(dec!(-30)),
            create_test_trade(dec!(75)),
            create_test_trade(dec!(-25)),
        ];

        let stats = TradeStats::from_trades(&trades);

        assert_eq!(stats.total_trades, 5);
        assert_eq!(stats.winning_trades, 3);
        assert_eq!(stats.losing_trades, 2);
        assert_eq!(stats.win_rate_pct, dec!(60));
        assert_eq!(stats.total_pnl, dec!(170)); // 225 - 55
        assert_eq!(stats.gross_profit, dec!(225));
        assert_eq!(stats.gross_loss, dec!(55));
    }
}
