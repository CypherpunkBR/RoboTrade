# Telemetria e Monitoramento

Este documento descreve o sistema de telemetria, monitoramento e diagnóstico do RoboTrade.

## Princípios

1. **Privacidade** - Nenhum dado sensível é coletado ou transmitido
2. **Opt-in** - Telemetria externa requer consentimento explícito
3. **Local First** - Métricas são armazenadas localmente
4. **Transparência** - Usuário pode ver todos os dados coletados
5. **Performance** - Coleta não impacta performance do sistema

## Arquitetura de Telemetria

```
┌─────────────────────────────────────────────────────────────────┐
│                     FONTES DE MÉTRICAS                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │   Market    │  │   Worker    │  │  Exchange   │             │
│  │    Data     │  │   Queue     │  │  Gateways   │             │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘             │
│         │                │                │                      │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │  Analytics  │  │  Database   │  │   System    │             │
│  │             │  │             │  │  Resources  │             │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘             │
│         │                │                │                      │
└─────────┴────────────────┴────────────────┴──────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│                    METRICS COLLECTOR                             │
│                                                                  │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │                    MetricsRegistry                         │ │
│  │                                                             │ │
│  │  - Counters   (total_orders, errors_count, etc.)          │ │
│  │  - Gauges     (queue_size, open_positions, etc.)          │ │
│  │  - Histograms (latency_ms, order_fill_time, etc.)         │ │
│  │                                                             │ │
│  └───────────────────────────────────────────────────────────┘ │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
                           │
           ┌───────────────┼───────────────┐
           │               │               │
           ▼               ▼               ▼
┌─────────────────┐ ┌─────────────┐ ┌─────────────────┐
│  SQLite Store   │ │    Logs     │ │   Dashboard     │
│  (histórico)    │ │  (tracing)  │ │   (UI/API)      │
└─────────────────┘ └─────────────┘ └─────────────────┘
```

## Métricas Coletadas

### Métricas de Market Data

```rust
pub struct MarketDataMetrics {
    /// Total de coletas realizadas
    pub fetch_count: Counter,

    /// Coletas com sucesso
    pub fetch_success: Counter,

    /// Coletas com falha
    pub fetch_errors: Counter,

    /// Latência de coleta (ms)
    pub fetch_latency: Histogram,

    /// Candles processados
    pub candles_processed: Counter,

    /// Bytes recebidos
    pub bytes_received: Counter,

    /// Último timestamp de coleta bem-sucedida
    pub last_successful_fetch: Gauge,
}
```

### Métricas do Worker

```rust
pub struct WorkerMetrics {
    /// Tamanho atual da fila
    pub queue_size: Gauge,

    /// Jobs enfileirados (total)
    pub jobs_enqueued: Counter,

    /// Jobs completados
    pub jobs_completed: Counter,

    /// Jobs falhados
    pub jobs_failed: Counter,

    /// Retries executados
    pub retries: Counter,

    /// Tempo de processamento por job (ms)
    pub job_processing_time: Histogram,

    /// Tempo na fila por job (ms)
    pub job_queue_time: Histogram,

    /// Jobs por tipo
    pub jobs_by_type: HashMap<String, Counter>,

    /// Tempo desde último job processado
    pub idle_time: Gauge,
}
```

### Métricas de Exchange

```rust
pub struct ExchangeMetrics {
    /// Requisições enviadas
    pub requests_sent: Counter,

    /// Requisições bem-sucedidas
    pub requests_success: Counter,

    /// Requisições com erro
    pub requests_error: Counter,

    /// Latência por exchange (ms)
    pub latency_by_exchange: HashMap<String, Histogram>,

    /// Rate limit hits
    pub rate_limit_hits: Counter,

    /// Ordens enviadas
    pub orders_submitted: Counter,

    /// Ordens preenchidas
    pub orders_filled: Counter,

    /// Ordens canceladas
    pub orders_cancelled: Counter,

    /// Ordens rejeitadas
    pub orders_rejected: Counter,

    /// Tempo médio de preenchimento (ms)
    pub order_fill_time: Histogram,
}
```

### Métricas de Trading

```rust
pub struct TradingMetrics {
    /// Posições abertas
    pub open_positions: Gauge,

    /// Total de trades
    pub total_trades: Counter,

    /// Trades vencedores
    pub winning_trades: Counter,

    /// Trades perdedores
    pub losing_trades: Counter,

    /// P&L total
    pub total_pnl: Gauge,

    /// P&L do dia
    pub daily_pnl: Gauge,

    /// Win rate (rolling 30 dias)
    pub win_rate_30d: Gauge,

    /// Drawdown atual
    pub current_drawdown: Gauge,

    /// Máximo drawdown
    pub max_drawdown: Gauge,

    /// Sinais gerados
    pub signals_generated: Counter,

    /// Sinais executados
    pub signals_executed: Counter,
}
```

### Métricas de Sistema

```rust
pub struct SystemMetrics {
    /// Uso de CPU (%)
    pub cpu_usage: Gauge,

    /// Uso de memória (bytes)
    pub memory_usage: Gauge,

    /// Tamanho do banco de dados (bytes)
    pub database_size: Gauge,

    /// Conexões de banco ativas
    pub db_active_connections: Gauge,

    /// Tempo de uptime (segundos)
    pub uptime: Gauge,

    /// Versão do app
    pub app_version: String,
}
```

## Implementação

### MetricsRegistry

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use parking_lot::RwLock;

pub struct MetricsRegistry {
    market_data: Arc<MarketDataMetrics>,
    worker: Arc<WorkerMetrics>,
    exchange: Arc<ExchangeMetrics>,
    trading: Arc<TradingMetrics>,
    system: Arc<SystemMetrics>,
}

impl MetricsRegistry {
    pub fn new() -> Self {
        Self {
            market_data: Arc::new(MarketDataMetrics::default()),
            worker: Arc::new(WorkerMetrics::default()),
            exchange: Arc::new(ExchangeMetrics::default()),
            trading: Arc::new(TradingMetrics::default()),
            system: Arc::new(SystemMetrics::default()),
        }
    }

    pub fn market_data(&self) -> &MarketDataMetrics {
        &self.market_data
    }

    pub fn worker(&self) -> &WorkerMetrics {
        &self.worker
    }

    // ... etc

    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            timestamp: Utc::now(),
            market_data: self.market_data.snapshot(),
            worker: self.worker.snapshot(),
            exchange: self.exchange.snapshot(),
            trading: self.trading.snapshot(),
            system: self.system.snapshot(),
        }
    }
}

// Counter simples
pub struct Counter {
    value: AtomicU64,
}

impl Counter {
    pub fn new() -> Self {
        Self { value: AtomicU64::new(0) }
    }

    pub fn inc(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    pub fn add(&self, n: u64) {
        self.value.fetch_add(n, Ordering::Relaxed);
    }

    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }
}

// Gauge (valor que pode subir e descer)
pub struct Gauge {
    value: AtomicU64,
}

impl Gauge {
    pub fn set(&self, value: u64) {
        self.value.store(value, Ordering::Relaxed);
    }

    pub fn set_f64(&self, value: f64) {
        self.value.store(value.to_bits(), Ordering::Relaxed);
    }

    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }

    pub fn get_f64(&self) -> f64 {
        f64::from_bits(self.value.load(Ordering::Relaxed))
    }
}

// Histogram para distribuição de valores
pub struct Histogram {
    buckets: RwLock<Vec<f64>>,
    sum: AtomicU64,
    count: AtomicU64,
}

impl Histogram {
    pub fn observe(&self, value: f64) {
        let mut buckets = self.buckets.write();
        buckets.push(value);

        // Manter apenas últimas 1000 observações
        if buckets.len() > 1000 {
            buckets.remove(0);
        }

        self.sum.fetch_add(value.to_bits(), Ordering::Relaxed);
        self.count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn percentile(&self, p: f64) -> f64 {
        let buckets = self.buckets.read();
        if buckets.is_empty() {
            return 0.0;
        }

        let mut sorted = buckets.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let index = ((p / 100.0) * (sorted.len() - 1) as f64) as usize;
        sorted[index]
    }

    pub fn mean(&self) -> f64 {
        let count = self.count.load(Ordering::Relaxed);
        if count == 0 {
            return 0.0;
        }

        let sum = f64::from_bits(self.sum.load(Ordering::Relaxed));
        sum / count as f64
    }
}
```

### Persistência de Métricas

```rust
pub struct MetricsPersistence {
    db: DbPool,
}

impl MetricsPersistence {
    /// Salva snapshot de métricas
    pub async fn save_snapshot(&self, snapshot: &MetricsSnapshot) -> InfraResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO metrics_snapshots (timestamp, data)
            VALUES (?, ?)
            "#,
            snapshot.timestamp.to_rfc3339(),
            serde_json::to_string(snapshot)?,
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Busca histórico de métricas
    pub async fn get_history(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        resolution: Duration,
    ) -> InfraResult<Vec<MetricsSnapshot>> {
        let rows = sqlx::query!(
            r#"
            SELECT timestamp, data FROM metrics_snapshots
            WHERE timestamp >= ? AND timestamp <= ?
            ORDER BY timestamp ASC
            "#,
            start.to_rfc3339(),
            end.to_rfc3339(),
        )
        .fetch_all(&self.db)
        .await?;

        let snapshots: Vec<MetricsSnapshot> = rows
            .into_iter()
            .filter_map(|r| serde_json::from_str(&r.data).ok())
            .collect();

        // Agregar por resolução se necessário
        Ok(self.aggregate_by_resolution(snapshots, resolution))
    }

    /// Remove métricas antigas
    pub async fn cleanup(&self, older_than: DateTime<Utc>) -> InfraResult<u64> {
        let result = sqlx::query!(
            "DELETE FROM metrics_snapshots WHERE timestamp < ?",
            older_than.to_rfc3339(),
        )
        .execute(&self.db)
        .await?;

        Ok(result.rows_affected())
    }
}
```

## Logging Estruturado

### Setup com Tracing

```rust
use tracing::{Level, Subscriber};
use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
    Layer,
};
use tracing_appender::rolling::{RollingFileAppender, Rotation};

pub struct LoggingGuard {
    _guard: tracing_appender::non_blocking::WorkerGuard,
}

pub fn init_logging(config: &LoggingConfig) -> LoggingGuard {
    // Configurar filtro de nível
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.level));

    // Configurar rolling file appender
    let file_appender = RollingFileAppender::new(
        match config.rotation {
            LogRotation::Daily => Rotation::DAILY,
            LogRotation::Hourly => Rotation::HOURLY,
            LogRotation::Never => Rotation::NEVER,
        },
        logs_path(),
        "robotrade.log",
    );

    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // Configurar layers
    let file_layer = fmt::layer()
        .json()
        .with_writer(non_blocking)
        .with_filter(filter.clone());

    let console_layer = if config.console_enabled {
        Some(
            fmt::layer()
                .pretty()
                .with_filter(filter),
        )
    } else {
        None
    };

    // Registrar subscriber
    tracing_subscriber::registry()
        .with(file_layer)
        .with(console_layer)
        .init();

    tracing::info!(
        level = %config.level,
        format = ?config.format,
        rotation = ?config.rotation,
        "Logging initialized"
    );

    LoggingGuard { _guard: guard }
}
```

### Logs Estruturados

```rust
// Exemplo de uso de logs estruturados
impl TradingWorker {
    async fn process_job(&self, job: &Job) -> WorkerResult<()> {
        let span = tracing::info_span!(
            "process_job",
            job_id = %job.id,
            job_type = %job.job_type,
            attempt = job.retries + 1,
        );

        let _enter = span.enter();

        tracing::info!("Starting job processing");

        let start = Instant::now();

        match self.execute(job).await {
            Ok(result) => {
                let duration = start.elapsed();

                tracing::info!(
                    duration_ms = duration.as_millis(),
                    result = ?result,
                    "Job completed successfully"
                );

                self.metrics.worker().job_processing_time.observe(duration.as_millis() as f64);
                self.metrics.worker().jobs_completed.inc();

                Ok(())
            }
            Err(e) => {
                let duration = start.elapsed();

                tracing::error!(
                    duration_ms = duration.as_millis(),
                    error = %e,
                    error_chain = ?error_chain(&e),
                    "Job failed"
                );

                self.metrics.worker().jobs_failed.inc();

                Err(e)
            }
        }
    }
}

// Log de evento de trading
fn log_trade_event(trade: &Trade) {
    tracing::info!(
        event = "trade_closed",
        trade_id = %trade.id,
        symbol = %trade.symbol,
        side = %trade.side,
        entry_price = %trade.entry_price,
        exit_price = %trade.exit_price,
        quantity = %trade.quantity,
        pnl = %trade.net_pnl,
        return_pct = %trade.return_pct,
        duration_seconds = trade.duration_seconds,
        "Trade closed"
    );
}
```

## Dashboard de Métricas

### Comandos Tauri

```rust
#[tauri::command]
pub async fn get_metrics_snapshot(
    state: tauri::State<'_, AppState>,
) -> Result<MetricsSnapshotDto, String> {
    let snapshot = state.metrics.snapshot();
    Ok(MetricsSnapshotDto::from(snapshot))
}

#[tauri::command]
pub async fn get_metrics_history(
    state: tauri::State<'_, AppState>,
    start: String,
    end: String,
    resolution_minutes: u32,
) -> Result<Vec<MetricsSnapshotDto>, String> {
    let start = DateTime::parse_from_rfc3339(&start)
        .map_err(|e| e.to_string())?
        .with_timezone(&Utc);
    let end = DateTime::parse_from_rfc3339(&end)
        .map_err(|e| e.to_string())?
        .with_timezone(&Utc);

    let history = state.metrics_persistence
        .get_history(start, end, Duration::minutes(resolution_minutes as i64))
        .await
        .map_err(|e| e.to_string())?;

    Ok(history.into_iter().map(MetricsSnapshotDto::from).collect())
}

#[tauri::command]
pub async fn get_worker_status(
    state: tauri::State<'_, AppState>,
) -> Result<WorkerStatusDto, String> {
    Ok(WorkerStatusDto {
        running: state.worker.is_running(),
        queue_size: state.metrics.worker().queue_size.get() as u32,
        jobs_completed: state.metrics.worker().jobs_completed.get(),
        jobs_failed: state.metrics.worker().jobs_failed.get(),
        avg_processing_time_ms: state.metrics.worker().job_processing_time.mean(),
        p99_processing_time_ms: state.metrics.worker().job_processing_time.percentile(99.0),
    })
}

#[tauri::command]
pub async fn export_diagnostics(
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let report = DiagnosticsReport {
        generated_at: Utc::now(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        metrics: state.metrics.snapshot(),
        config: state.config.to_safe_export(),  // Sem dados sensíveis
        recent_errors: state.get_recent_errors(100).await,
        system_info: get_system_info(),
    };

    let json = serde_json::to_string_pretty(&report)
        .map_err(|e| e.to_string())?;

    Ok(json)
}
```

### DTOs para Frontend

```rust
#[derive(Debug, Clone, Serialize)]
pub struct MetricsSnapshotDto {
    pub timestamp: String,

    // Market Data
    pub market_data_fetch_count: u64,
    pub market_data_errors: u64,
    pub market_data_latency_avg_ms: f64,

    // Worker
    pub worker_queue_size: u64,
    pub worker_jobs_completed: u64,
    pub worker_jobs_failed: u64,
    pub worker_avg_processing_ms: f64,

    // Exchange
    pub exchange_requests_total: u64,
    pub exchange_errors: u64,
    pub exchange_latency_avg_ms: f64,

    // Trading
    pub open_positions: u64,
    pub total_trades: u64,
    pub win_rate: f64,
    pub total_pnl: String,
    pub daily_pnl: String,
    pub current_drawdown: String,

    // System
    pub cpu_usage: f64,
    pub memory_usage_mb: f64,
    pub database_size_mb: f64,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkerStatusDto {
    pub running: bool,
    pub queue_size: u32,
    pub jobs_completed: u64,
    pub jobs_failed: u64,
    pub avg_processing_time_ms: f64,
    pub p99_processing_time_ms: f64,
}
```

## Scheduler de Coleta

```rust
pub struct MetricsCollector {
    registry: Arc<MetricsRegistry>,
    persistence: Arc<MetricsPersistence>,
    interval: Duration,
    running: AtomicBool,
}

impl MetricsCollector {
    pub async fn start(&self) {
        self.running.store(true, Ordering::SeqCst);

        tracing::info!(
            interval_seconds = self.interval.as_secs(),
            "Metrics collector started"
        );

        while self.running.load(Ordering::SeqCst) {
            // Coletar métricas de sistema
            self.collect_system_metrics().await;

            // Salvar snapshot
            let snapshot = self.registry.snapshot();
            if let Err(e) = self.persistence.save_snapshot(&snapshot).await {
                tracing::error!(error = %e, "Failed to save metrics snapshot");
            }

            tokio::time::sleep(self.interval).await;
        }

        tracing::info!("Metrics collector stopped");
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    async fn collect_system_metrics(&self) {
        // CPU e memória (usando sysinfo crate)
        let sys = sysinfo::System::new_all();

        self.registry.system().cpu_usage.set_f64(
            sys.global_cpu_info().cpu_usage() as f64
        );

        self.registry.system().memory_usage.set(
            sys.used_memory()
        );

        // Tamanho do banco de dados
        if let Ok(metadata) = tokio::fs::metadata(database_path()).await {
            self.registry.system().database_size.set(metadata.len());
        }
    }
}
```

## Alertas

```rust
pub struct AlertManager {
    config: AlertConfig,
    notifier: Arc<dyn NotificationService>,
}

pub struct AlertConfig {
    pub error_rate_threshold: f64,      // Taxa de erro para alertar
    pub queue_size_threshold: u64,       // Tamanho máximo da fila
    pub latency_threshold_ms: f64,       // Latência máxima
    pub drawdown_threshold: Decimal,     // Drawdown para alertar
}

impl AlertManager {
    pub async fn check_alerts(&self, snapshot: &MetricsSnapshot) {
        // Check error rate
        let error_rate = self.calculate_error_rate(snapshot);
        if error_rate > self.config.error_rate_threshold {
            self.send_alert(Alert::HighErrorRate {
                current: error_rate,
                threshold: self.config.error_rate_threshold,
            }).await;
        }

        // Check queue size
        if snapshot.worker.queue_size > self.config.queue_size_threshold {
            self.send_alert(Alert::QueueBacklog {
                size: snapshot.worker.queue_size,
                threshold: self.config.queue_size_threshold,
            }).await;
        }

        // Check latency
        if snapshot.exchange.latency_avg > self.config.latency_threshold_ms {
            self.send_alert(Alert::HighLatency {
                current_ms: snapshot.exchange.latency_avg,
                threshold_ms: self.config.latency_threshold_ms,
            }).await;
        }

        // Check drawdown
        if snapshot.trading.current_drawdown > self.config.drawdown_threshold {
            self.send_alert(Alert::HighDrawdown {
                current: snapshot.trading.current_drawdown,
                threshold: self.config.drawdown_threshold,
            }).await;
        }
    }

    async fn send_alert(&self, alert: Alert) {
        tracing::warn!(alert = ?alert, "Alert triggered");

        let notification = Notification {
            title: alert.title(),
            body: alert.body(),
            level: NotificationLevel::Warning,
            data: Some(serde_json::to_value(&alert).unwrap()),
        };

        if let Err(e) = self.notifier.send(notification).await {
            tracing::error!(error = %e, "Failed to send alert notification");
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum Alert {
    HighErrorRate { current: f64, threshold: f64 },
    QueueBacklog { size: u64, threshold: u64 },
    HighLatency { current_ms: f64, threshold_ms: f64 },
    HighDrawdown { current: Decimal, threshold: Decimal },
}

impl Alert {
    fn title(&self) -> String {
        match self {
            Alert::HighErrorRate { .. } => "Taxa de Erro Alta".to_string(),
            Alert::QueueBacklog { .. } => "Fila Acumulada".to_string(),
            Alert::HighLatency { .. } => "Latência Alta".to_string(),
            Alert::HighDrawdown { .. } => "Drawdown Elevado".to_string(),
        }
    }

    fn body(&self) -> String {
        match self {
            Alert::HighErrorRate { current, threshold } =>
                format!("Taxa de erro {:.1}% excede limite de {:.1}%", current * 100.0, threshold * 100.0),
            Alert::QueueBacklog { size, threshold } =>
                format!("Fila com {} jobs (limite: {})", size, threshold),
            Alert::HighLatency { current_ms, threshold_ms } =>
                format!("Latência média {:.0}ms excede limite de {:.0}ms", current_ms, threshold_ms),
            Alert::HighDrawdown { current, threshold } =>
                format!("Drawdown de {}% excede limite de {}%", current, threshold),
        }
    }
}
```

---

**Próximo**: [Documentação de Negócio](../business/market-data.md)
