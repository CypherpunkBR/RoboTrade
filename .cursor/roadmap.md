# Roadmap e Features Não Implementadas

## ✅ Implementado (Pronto pra Usar)

### Core Features
- ✅ Trading manual (ordens market, limit, stop)
- ✅ Multi-exchange (Binance + Kraken Futures)
- ✅ Paper trading
- ✅ Gestão de posições com SL/TP
- ✅ Alavancagem configurável (1-125x)
- ✅ Gráficos interativos com indicadores
- ✅ Sistema de alertas de preço
- ✅ Histórico completo de trades
- ✅ Relatórios visuais (PnL, equity curve, etc)
- ✅ Risk management automático
- ✅ Circuit breaker
- ✅ Fear & Greed Index
- ✅ WebSocket streams (Binance + Kraken)
- ✅ Database SQLite
- ✅ Logging estruturado
- ✅ Configuração TOML
- ✅ System tray
- ✅ Ledger system (contabilidade)
- ✅ P&L calculator
- ✅ Reconciliação
- ✅ Sync histórico

### Indicators
- ✅ SMA, EMA
- ✅ RSI
- ✅ MACD
- ✅ Bollinger Bands
- ✅ ATR

### Estratégias
- ✅ Fear & Greed Strategy

## ⏳ Em Desenvolvimento

### Trading Worker
- ✅ Infraestrutura completa
- ✅ Risk manager
- ✅ Circuit breaker
- ⏳ **Integração end-to-end com estratégias** (código existe, falta ativar)
- ⏳ **Scheduler automático** (estrutura pronta, falta ligar)

### Repositories
- ✅ CandleRepository, FearGreedRepository
- ⏳ **TradeRepository** (trait definido, implementação pendente)
- ⏳ **PositionRepository** (trait definido, implementação pendente)
- ⏳ **OrderRepository** (trait definido, implementação pendente)

### Database
- ⏳ **Monthly P&L aggregation tables** (planejado)
- ⏳ **Account snapshots** (planejado)
- ⏳ **Currency-specific P&L tracking** (parcial)

## ❌ Planejado (Não Iniciado)

### Exchange Integration
- ❌ Binance Spot API
- ❌ Kraken Spot API
- ❌ OKX Futures
- ❌ Bybit Futures

### Estratégias Adicionais
- ❌ RSI Strategy
- ❌ MACD Strategy
- ❌ Bollinger Bands Strategy
- ❌ SMA Crossover Strategy (código exemplo existe)
- ❌ Grid Trading
- ❌ DCA (Dollar Cost Average)

### Backtesting
- ✅ Engine completo
- ❌ **Interface no frontend** (não conectado)
- ❌ **Walk-forward analysis**
- ❌ **Parameter optimization**
- ❌ **Multi-strategy comparison**
- ❌ **Portfolio backtesting**

### Notifications
- ❌ Desktop notifications (OS native)
- ❌ Telegram bot
- ❌ Discord webhook
- ❌ Email alerts
- ❌ SMS (Twilio)

### Reporting
- ✅ Gráficos básicos
- ❌ **Exportar pra PDF**
- ❌ **Exportar pra Excel**
- ❌ **Tax reports** (relatório de impostos)
- ❌ **Performance attribution**

### Advanced Features
- ❌ Machine Learning strategies
- ❌ Sentiment analysis (Twitter, Reddit)
- ❌ On-chain metrics integration
- ❌ Multi-timeframe analysis
- ❌ Portfolio optimization
- ❌ Copy trading
- ❌ Social trading features

### UI/UX
- ✅ Dark theme
- ❌ Light theme
- ❌ Customizable layouts
- ❌ Drag-and-drop panels
- ❌ Chart templates
- ❌ Hotkeys/shortcuts

### Mobile
- ❌ React Native app
- ❌ Mobile-responsive web
- ❌ Push notifications mobile

---

## Prioridades (Tier 1)

Próximas features a implementar:

1. **Ativar Trading Worker End-to-End** ⏳
   - Ligar scheduler com estratégias
   - Testar fluxo completo
   - Adicionar logs detalhados

2. **Implementar TradeRepository** ⏳
   - Queries otimizadas
   - Agregação por mês/símbolo
   - Performance metrics

3. **Monthly P&L Aggregation** ⏳
   - Tabela monthly_pnl
   - Triggers automáticos
   - API pra consulta

4. **Interface de Backtest no Frontend** ❌
   - Página de backtest
   - Form de config
   - Visualização de resultados

5. **Desktop Notifications** ❌
   - Alertas de preço
   - Sinais de trading
   - Ordens executadas

## Estimativas

| Feature | Complexidade | Tempo Estimado |
|---------|--------------|----------------|
| Trade Repository | Média | 2-3 dias |
| Monthly P&L | Baixa | 1-2 dias |
| Backtest UI | Média | 3-5 dias |
| Desktop Notifications | Baixa | 1 dia |
| Telegram Bot | Média | 2-3 dias |
| RSI Strategy | Baixa | 1 dia |
| Binance Spot | Média | 3-5 dias |

---

**Roadmap vivo! Prioridades mudam baseado em feedback. 🗺️**
