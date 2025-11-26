# RoboTrade - Roadmap

## Visão Geral

Este documento descreve o roadmap de desenvolvimento do RoboTrade, um robô de trading automatizado focado em criptomoedas.

---

## Fase 1: Fundação (Atual)

### ✅ Concluído
- [x] Estrutura do workspace Rust com múltiplos crates
- [x] Crate `core` com entidades, erros e traits
- [x] Crate `infra` com configuração, logging e SQLite
- [x] Provider Fear & Greed Index (alternative.me)

### 🔄 Em Progresso
- [ ] Provider Binance Futures (testnet)
- [ ] Gateway de exchange com abstração

### 📋 Pendente
- [ ] Setup Tauri com System Tray
- [ ] Scheduler de coleta de dados
- [ ] Motor de backtest
- [ ] Estratégia Fear & Greed
- [ ] Frontend React + Tailwind + shadcn/ui

---

## Fase 2: Exchanges e Histórico

### Binance Futures
- [ ] Conexão com API REST (testnet e produção)
- [ ] WebSocket para dados em tempo real
- [ ] **Importação de histórico completo de trades**
  - Puxar todos os trades históricos da conta
  - Calcular P&L por período (dia, semana, mês, ano)
  - Visualização de perdas e ganhos por símbolo
  - Gráfico de curva de equity histórica
- [ ] Gestão de posições abertas
- [ ] Envio e cancelamento de ordens
- [ ] Suporte a hedge mode

### Kraken Pro Futures
- [ ] Conexão com API REST
- [ ] WebSocket para dados em tempo real
- [ ] **Importação de histórico completo de trades**
  - Sincronização com histórico de trades
  - Análise de performance por par
  - Relatório de fees pagos
  - Identificação de trades perdedores
- [ ] Gestão de posições
- [ ] Suporte a ordens condicionais

### Análise de Histórico
- [ ] Dashboard unificado de P&L (todas as exchanges)
- [ ] Filtros por período, símbolo, lado (long/short)
- [ ] Exportação para CSV/Excel
- [ ] Cálculo de impostos (lucro líquido)
- [ ] Métricas de performance:
  - Win rate por exchange
  - Drawdown máximo
  - Profit factor
  - Sharpe ratio
  - Trades mais lucrativos vs mais perdedores

---

## Fase 3: Estratégias Avançadas

### Indicadores Técnicos
- [ ] RSI (Relative Strength Index)
- [ ] MACD (Moving Average Convergence Divergence)
- [ ] Bollinger Bands
- [ ] EMA/SMA (Médias Móveis)
- [ ] ATR (Average True Range)
- [ ] Volume Profile

### Estratégias
- [ ] Fear & Greed contrarian (compra medo, vende ganância)
- [ ] RSI oversold/overbought
- [ ] Cruzamento de médias
- [ ] Breakout de Bollinger
- [ ] DCA (Dollar Cost Averaging) automatizado
- [ ] Grid trading

### Backtesting
- [ ] Motor de backtest com dados históricos
- [ ] Simulação de slippage e fees
- [ ] Otimização de parâmetros
- [ ] Walk-forward analysis
- [ ] Monte Carlo simulation
- [ ] Comparação entre estratégias

---

## Fase 4: Risk Management

### Gestão de Risco
- [ ] Limite de posição por símbolo
- [ ] Limite de risco diário/semanal
- [ ] Trailing stop automático
- [ ] Break-even automático
- [ ] Scaling in/out de posições
- [ ] Correlação entre posições

### Circuit Breakers
- [ ] Pausa após N perdas consecutivas
- [ ] Pausa após drawdown percentual
- [ ] Limite de ordens por minuto
- [ ] Detecção de volatilidade extrema
- [ ] Alerta de liquidação próxima

---

## Fase 5: Interface e UX

### System Tray
- [ ] Ícone com status (conectado/desconectado)
- [ ] Menu com posições abertas
- [ ] P&L do dia em tempo real
- [ ] Quick actions (pausar trading, fechar posições)
- [ ] Notificações nativas

### Dashboard Principal
- [ ] Overview de conta (saldo, P&L, posições)
- [ ] Gráficos de performance
- [ ] Lista de sinais ativos
- [ ] Histórico de trades recentes
- [ ] Fear & Greed Index atual

### Páginas
- [ ] Estratégias (ativar/desativar, configurar)
- [ ] Backtest (rodar, comparar resultados)
- [ ] Histórico (trades, sinais, ordens)
- [ ] Análise (perdas, ganhos, métricas)
- [ ] Configurações (API keys, preferências)

---

## Fase 6: Recursos Avançados

### Multi-Exchange
- [ ] Arbitragem entre exchanges
- [ ] Agregação de orderbook
- [ ] Smart order routing
- [ ] Balanceamento de margem

### Automação
- [ ] Agendamento de estratégias
- [ ] Alertas customizados
- [ ] Webhooks para integração externa
- [ ] Telegram bot para notificações

### Análise Avançada
- [ ] Machine Learning para predição
- [ ] Sentiment analysis (Twitter, Reddit)
- [ ] On-chain metrics (whale movements)
- [ ] Funding rate analysis

---

## Fase 7: Produção e Segurança

### Segurança
- [ ] Criptografia de API keys (keyring)
- [ ] Triple-lock para modo live
- [ ] Audit log de todas as ações
- [ ] Rate limiting interno
- [ ] Validação de assinaturas

### Confiabilidade
- [ ] Reconexão automática
- [ ] Sincronização de estado
- [ ] Backup de configurações
- [ ] Recuperação de falhas
- [ ] Health checks contínuos

### Monitoramento
- [ ] Métricas de sistema
- [ ] Logs estruturados
- [ ] Alertas de anomalias
- [ ] Dashboard de operações

---

## Notas Técnicas

### Exchanges Suportadas
| Exchange | Status | Tipo | Prioridade |
|----------|--------|------|------------|
| Binance Futures | 🔄 Em desenvolvimento | Perpetual | Alta |
| Kraken Pro Futures | 📋 Planejado | Perpetual | Alta |
| Bybit | 📋 Planejado | Perpetual | Média |
| OKX | 📋 Planejado | Perpetual | Baixa |

### Stack Tecnológico
- **Backend**: Rust (100% da lógica de negócio)
- **Frontend**: React + TypeScript + Tailwind + shadcn/ui
- **Desktop**: Tauri 2.0
- **Database**: SQLite (local)
- **Async**: Tokio

---

## Contribuindo

Para contribuir com o projeto, veja [CONTRIBUTING.md](./CONTRIBUTING.md).

## Licença

MIT
