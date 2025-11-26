# Changelog

Todas as mudanças notáveis neste projeto serão documentadas neste arquivo.

O formato é baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.0.0/),
e este projeto adere ao [Versionamento Semântico](https://semver.org/lang/pt-BR/).

## [Não Lançado]

### Adicionado
- Estrutura inicial do workspace Rust com 7 crates
- Integração com Binance Futures (REST API)
- Provider Fear & Greed Index (alternative.me)
- Engine de backtest com simulação de posições
- Estratégia baseada em Fear & Greed Index
- Scheduler para coleta periódica de dados
- Aplicação desktop Tauri com system tray
- Banco de dados SQLite com schema completo
- Sistema de configuração via TOML
- Logging estruturado com tracing

### Infraestrutura
- Configuração de ferramentas de qualidade (rustfmt, clippy, cargo-deny)
- Husky com hooks pre-commit e commit-msg
- Detecção de código duplicado com jscpd
- Conventional Commits com commitlint

## [0.1.0] - 2024-11-26

### Adicionado

#### Core (`robotrade-core`)
- Entidades fundamentais: `Asset`, `TradingPair`, `ExchangeId`
- Estruturas de mercado: `Candle`, `TimeFrame` (15 intervalos), `Ticker`
- Gerenciamento de ordens: `Order`, `OrderRequest`, `OrderStatus`, `OrderType`
- Controle de posições: `Position`, `PositionSide`, `PositionStatus`
- Sistema de sinais: `Signal`, `SignalStrength`, `TradeDirection`
- Histórico de trades: `Trade`, `TradeCloseReason`, `TradeStats`
- Dados Fear & Greed: `FearGreedData`, `FearGreedClassification`, `FearGreedHistory`
- Hierarquia de erros com 7 tipos específicos de domínio
- 15 traits para abstrações (Repository, Provider, Gateway, Strategy)
- DTOs para comunicação frontend-backend

#### Exchange Gateways (`robotrade-exchange-gateways`)
- Cliente Binance Futures completo
- Suporte a testnet e mainnet
- Assinatura HMAC-SHA256 para requests autenticados
- Métodos: submit_order, cancel_order, get_positions, get_balances, set_leverage

#### Market Data (`robotrade-market-data`)
- Provider Fear & Greed Index (alternative.me API)
- Métodos: fetch_current, fetch_history, health_check

#### Analytics (`robotrade-analytics`)
- Engine de backtest com simulação completa
- Configuração: capital inicial, tamanho de posição, fees, slippage, alavancagem
- Tracking de posições simuladas e equity curve
- Cálculo de métricas: P&L, max drawdown, win rate, profit factor, Sharpe ratio
- Estratégia Fear & Greed com thresholds configuráveis

#### Trading Worker (`robotrade-trading-worker`)
- Scheduler event-driven com broadcast channel
- Eventos: CollectFearGreed, UpdatePrices, CheckPositions, RunStrategies
- Intervalos configuráveis para cada tipo de evento
- DataCollector para coleta periódica de dados de mercado

#### Infra (`robotrade-infra`)
- Sistema de configuração TOML com 6 seções
- Banco de dados SQLite com 8+ tabelas
- Repositórios: SqliteCandleRepository, SqliteFearGreedRepository
- Pool de conexões com health check
- Logging estruturado com tracing, JSON output, file appenders

#### Aplicação Tauri (`src-tauri`)
- 15 comandos Tauri para interface desktop
- AppState com modo de trading, status de conexão, posições
- System tray com ações show/hide e quit
- Plugins: shell, notification

### Dependências
- tokio 1.35 (runtime assíncrono)
- sqlx 0.7 (SQLite)
- reqwest 0.12 (HTTP client)
- rust_decimal 1.33 (precisão financeira)
- tracing 0.1 (logging estruturado)
- tauri 2.0 (desktop app)
