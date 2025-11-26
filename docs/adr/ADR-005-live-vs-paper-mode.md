# ADR-005: Modo Live vs Paper Trading

## Status

Aceita

## Contexto

O RoboTrade opera em dois modos fundamentalmente diferentes:

1. **Paper Trading**: Simulação sem dinheiro real
   - Para testar estratégias em condições de mercado real
   - Sem risco financeiro
   - Execução instantânea simulada

2. **Live Trading**: Operações reais com capital
   - Requer credenciais de API da exchange
   - Risco financeiro real
   - Sujeito a latência, slippage, rejeições

A transição entre esses modos é crítica:
- Acidentes podem causar perdas financeiras significativas
- Usuários devem estar cientes do modo atual
- Código não deve "acidentalmente" executar ordens reais

## Decisão

Implementamos uma **separação arquitetural forte** entre os modos, com múltiplas camadas de proteção.

### Tipos Distintos (Type-Level Safety)

```rust
use std::marker::PhantomData;

/// Marker traits para modos de trading
pub trait TradingMode: Send + Sync + 'static {
    const IS_LIVE: bool;
    const MODE_NAME: &'static str;
}

#[derive(Debug, Clone, Copy)]
pub struct LiveMode;

impl TradingMode for LiveMode {
    const IS_LIVE: bool = true;
    const MODE_NAME: &'static str = "LIVE";
}

#[derive(Debug, Clone, Copy)]
pub struct PaperMode;

impl TradingMode for PaperMode {
    const IS_LIVE: bool = false;
    const MODE_NAME: &'static str = "PAPER";
}

/// Order executor parametrizado pelo modo
pub struct OrderExecutor<M: TradingMode> {
    gateway: Box<dyn ExchangeGateway>,
    _mode: PhantomData<M>,
}

impl<M: TradingMode> OrderExecutor<M> {
    pub async fn execute(&self, order: Order) -> Result<OrderResult, Error> {
        tracing::info!(
            mode = M::MODE_NAME,
            symbol = %order.symbol,
            side = ?order.side,
            quantity = %order.quantity,
            "Executing order"
        );

        // Lógica comum...
        self.gateway.place_order(order).await
    }
}

// Métodos específicos para LiveMode
impl OrderExecutor<LiveMode> {
    /// Verificação adicional obrigatória para ordens live
    pub async fn execute_with_confirmation(
        &self,
        order: Order,
        confirmation_token: ConfirmationToken,
    ) -> Result<OrderResult, Error> {
        confirmation_token.verify()?;
        self.execute(order).await
    }
}
```

### Gateway Implementations

```rust
/// Gateway para paper trading - simulação local
pub struct PaperGateway {
    positions: RwLock<HashMap<String, PaperPosition>>,
    order_book: RwLock<Vec<PaperOrder>>,
    market_data: Arc<MarketDataService>,
}

impl ExchangeGateway for PaperGateway {
    async fn place_order(&self, order: OrderRequest) -> Result<OrderResponse, Error> {
        // Simula execução instantânea ao preço atual
        let current_price = self.market_data
            .get_current_price(&order.symbol)
            .await?;

        let fill_price = match order.order_type {
            OrderType::Market => current_price,
            OrderType::Limit { price } => {
                // Simula: limit só executa se preço favorável
                if self.would_fill(order.side, price, current_price) {
                    price
                } else {
                    // Adiciona ao order book simulado
                    self.add_pending_order(order).await;
                    return Ok(OrderResponse::pending(order.client_id));
                }
            }
        };

        // Atualiza posição simulada
        self.update_paper_position(&order.symbol, order.side, order.quantity, fill_price).await;

        Ok(OrderResponse {
            order_id: format!("paper_{}", Uuid::new_v4()),
            status: OrderStatus::Filled,
            filled_quantity: order.quantity,
            average_price: fill_price,
            // ...
        })
    }

    async fn get_balance(&self) -> Result<Balance, Error> {
        // Retorna balanço simulado
        let positions = self.positions.read().await;
        // ... calcular balanço baseado em posições paper
    }
}

/// Gateway para trading real - conecta à Binance
pub struct BinanceGateway {
    client: BinanceClient,
    rate_limiter: RateLimiter,
}

impl ExchangeGateway for BinanceGateway {
    async fn place_order(&self, order: OrderRequest) -> Result<OrderResponse, Error> {
        // Rate limiting
        self.rate_limiter.acquire().await?;

        // Chamada real à API
        let response = self.client
            .post("/api/v3/order")
            .sign()  // HMAC-SHA256
            .json(&order.to_binance_format())
            .send()
            .await?;

        // Parse resposta real
        OrderResponse::from_binance(response.json().await?)
    }
}
```

### Configuração e Inicialização

```rust
pub struct TradingConfig {
    pub mode: TradingModeConfig,
    pub live_safeguards: LiveSafeguards,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TradingModeConfig {
    Paper {
        initial_balance: Decimal,
        simulated_latency_ms: u64,
        simulated_slippage_bps: u32,
    },
    Live {
        exchange: ExchangeConfig,
        require_confirmation: bool,
        max_order_value_usd: Decimal,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveSafeguards {
    /// Requer digitação de "LIVE" para ativar
    pub activation_phrase: String,
    /// Limite diário de perdas antes de pausar
    pub daily_loss_limit_percent: Decimal,
    /// Número máximo de ordens por hora
    pub max_orders_per_hour: u32,
    /// Cooldown após erro de API (segundos)
    pub error_cooldown_seconds: u64,
}

pub fn create_executor(config: &TradingConfig) -> Box<dyn OrderExecutorTrait> {
    match &config.mode {
        TradingModeConfig::Paper { initial_balance, .. } => {
            Box::new(OrderExecutor::<PaperMode>::new(
                Box::new(PaperGateway::new(*initial_balance))
            ))
        }
        TradingModeConfig::Live { exchange, .. } => {
            // Verificações adicionais antes de criar executor live
            assert!(
                config.live_safeguards.activation_phrase == "LIVE",
                "Live trading requires explicit activation"
            );

            Box::new(OrderExecutor::<LiveMode>::new(
                Box::new(BinanceGateway::new(exchange.clone()))
            ))
        }
    }
}
```

### UI Safeguards

```rust
// Comando Tauri com verificação de modo
#[tauri::command]
pub async fn place_order(
    state: State<'_, AppState>,
    order: OrderRequest,
    mode_confirmation: Option<String>,
) -> Result<OrderResponse, String> {
    let is_live = state.config.is_live_mode();

    if is_live {
        // Exige confirmação explícita
        match mode_confirmation.as_deref() {
            Some("CONFIRM_LIVE_ORDER") => {}
            _ => return Err("Live orders require explicit confirmation".into()),
        }

        // Log especial para ordens live
        tracing::warn!(
            symbol = %order.symbol,
            side = ?order.side,
            quantity = %order.quantity,
            "LIVE ORDER PLACED"
        );
    }

    state.executor.execute(order).await.map_err(|e| e.to_string())
}
```

### Indicadores Visuais (Frontend)

```typescript
// React component para indicar modo atual
const TradingModeIndicator: React.FC<{ mode: 'paper' | 'live' }> = ({ mode }) => {
  if (mode === 'live') {
    return (
      <div className="fixed top-0 left-0 right-0 bg-red-600 text-white text-center py-1 z-50">
        ⚠️ LIVE TRADING ATIVO - OPERAÇÕES REAIS ⚠️
      </div>
    );
  }

  return (
    <div className="fixed top-0 left-0 right-0 bg-blue-600 text-white text-center py-1 z-50">
      📝 Paper Trading - Simulação
    </div>
  );
};
```

## Consequências

### Positivas

- **Segurança em compile-time**: Tipos diferentes previnem mistura acidental
- **Clareza visual**: Usuário sempre sabe o modo atual
- **Proteção em camadas**: Múltiplas verificações antes de ordem live
- **Auditoria**: Logs distintos para operações reais
- **Simulação realista**: Paper pode incluir latência e slippage simulados

### Negativas

- **Duplicação de código**: Alguns paths paralelos para live/paper
- **Complexidade**: Sistema de tipos mais elaborado
- **UX fricção**: Confirmações extras podem ser tediosas

### Neutras

- Usuários avançados podem desabilitar algumas confirmações
- Métricas de paper e live são armazenadas separadamente

## Alternativas Consideradas

### Alternativa 1: Flag Booleano Simples

- **Descrição**: `is_live: bool` em runtime
- **Prós**: Simples de implementar
- **Contras**: Fácil de ignorar, sem garantias de compile-time
- **Motivo da rejeição**: Risco de erros acidentais muito alto

### Alternativa 2: Ambientes Separados

- **Descrição**: Builds diferentes para paper e live
- **Prós**: Isolamento total
- **Contras**: Duplica infraestrutura, difícil alternar
- **Motivo da rejeição**: UX ruim, manutenção duplicada

### Alternativa 3: Exchange Testnet

- **Descrição**: Usar testnet da Binance para paper
- **Prós**: API real, comportamento mais realista
- **Contras**: Testnet instável, dados de mercado diferentes
- **Motivo da rejeição**: Não adequado para teste com dados reais

## Checklist de Segurança para Live Trading

- [ ] API keys armazenadas com keyring (não em arquivo)
- [ ] Permissões de API mínimas (apenas trading, não withdrawal)
- [ ] IP whitelist configurado na exchange
- [ ] Daily loss limit configurado
- [ ] Notificações configuradas para ordens executadas
- [ ] Backup da config antes de mudanças
- [ ] Teste em paper com mesma estratégia primeiro

## Referências

- [Rust Typestate Pattern](https://cliffle.com/blog/rust-typestate/)
- [Binance API Security Best Practices](https://www.binance.com/en/support/faq)
- [Defense in Depth](https://en.wikipedia.org/wiki/Defense_in_depth_(computing))
