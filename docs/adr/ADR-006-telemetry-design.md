# ADR-006: Design de Telemetria

## Status

Aceita

## Contexto

O RoboTrade precisa de observabilidade para:

1. **Debugging**: Entender o que aconteceu quando algo deu errado
2. **Performance**: Identificar gargalos e otimizar
3. **Auditoria**: Rastrear decisões de trading
4. **Alertas**: Notificar sobre condições anormais
5. **Analytics**: Entender padrões de uso e comportamento do sistema

Requisitos:

- Baixo overhead em runtime
- Logs estruturados (não apenas strings)
- Métricas exportáveis (Prometheus format opcional)
- Traces para debugging de fluxos complexos
- Funcionar offline (aplicação desktop)
- Histórico consultável

## Decisão

Adotamos o ecossistema **tracing** como foundation, com storage local em SQLite para histórico.

### Arquitetura

```
┌─────────────────────────────────────────────────────────────────┐
│                       Application Code                          │
│  tracing::info!(), tracing::span!(), metrics!()                │
└─────────────────────────────────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────┐
│                     tracing Subscriber                          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐ │
│  │   Console   │  │    File     │  │      SQLite Layer       │ │
│  │   Layer     │  │   Layer     │  │  (structured events)    │ │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                               │
              ┌────────────────┼────────────────┐
              │                │                │
              ▼                ▼                ▼
         ┌─────────┐    ┌──────────┐    ┌──────────────┐
         │ stdout  │    │ .log     │    │  events.db   │
         │ (dev)   │    │ files    │    │  (SQLite)    │
         └─────────┘    └──────────┘    └──────────────┘
```

### Configuração do Subscriber

```rust
use tracing_subscriber::{
    fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer,
};

pub fn init_telemetry(config: &TelemetryConfig) -> Result<(), Error> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.default_level));

    // Console layer (desenvolvimento)
    let console_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_filter(env_filter.clone());

    // File layer (produção)
    let file_appender = tracing_appender::rolling::daily(&config.log_dir, "robotrade.log");
    let file_layer = fmt::layer()
        .json()
        .with_writer(file_appender)
        .with_filter(env_filter.clone());

    // SQLite layer (eventos estruturados para UI)
    let sqlite_layer = SqliteLayer::new(&config.events_db_path)?;

    tracing_subscriber::registry()
        .with(console_layer)
        .with(file_layer)
        .with(sqlite_layer)
        .init();

    Ok(())
}
```

### SQLite Layer Customizado

```rust
use tracing::{Event, Subscriber};
use tracing_subscriber::layer::Context;

pub struct SqliteLayer {
    db: SqlitePool,
    tx: mpsc::Sender<EventRecord>,
}

#[derive(Debug)]
struct EventRecord {
    timestamp: DateTime<Utc>,
    level: String,
    target: String,
    message: String,
    fields: serde_json::Value,
    span_id: Option<String>,
}

impl<S: Subscriber> tracing_subscriber::Layer<S> for SqliteLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut visitor = JsonVisitor::default();
        event.record(&mut visitor);

        let record = EventRecord {
            timestamp: Utc::now(),
            level: event.metadata().level().to_string(),
            target: event.metadata().target().to_string(),
            message: visitor.message,
            fields: visitor.fields,
            span_id: None, // TODO: extrair do contexto
        };

        // Enviar para writer assíncrono (não bloqueia)
        let _ = self.tx.try_send(record);
    }
}

impl SqliteLayer {
    pub fn new(db_path: &str) -> Result<Self, Error> {
        let db = SqlitePool::connect(db_path).await?;

        // Criar tabela se não existir
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS telemetry_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                level TEXT NOT NULL,
                target TEXT NOT NULL,
                message TEXT,
                fields_json TEXT,
                span_id TEXT
            )
        "#).execute(&db).await?;

        // Canal para escritas assíncronas
        let (tx, rx) = mpsc::channel(10000);

        // Spawn writer background task
        let db_clone = db.clone();
        tokio::spawn(async move {
            Self::writer_loop(db_clone, rx).await;
        });

        Ok(Self { db, tx })
    }

    async fn writer_loop(db: SqlitePool, mut rx: mpsc::Receiver<EventRecord>) {
        let mut batch = Vec::with_capacity(100);
        let mut interval = tokio::time::interval(Duration::from_secs(1));

        loop {
            tokio::select! {
                Some(record) = rx.recv() => {
                    batch.push(record);
                    if batch.len() >= 100 {
                        Self::flush_batch(&db, &mut batch).await;
                    }
                }
                _ = interval.tick() => {
                    if !batch.is_empty() {
                        Self::flush_batch(&db, &mut batch).await;
                    }
                }
            }
        }
    }

    async fn flush_batch(db: &SqlitePool, batch: &mut Vec<EventRecord>) {
        // Batch insert para performance
        let mut query = String::from(
            "INSERT INTO telemetry_events (timestamp, level, target, message, fields_json, span_id) VALUES "
        );

        for (i, record) in batch.iter().enumerate() {
            if i > 0 { query.push_str(", "); }
            query.push_str(&format!(
                "('{}', '{}', '{}', '{}', '{}', {})",
                record.timestamp.to_rfc3339(),
                record.level,
                record.target,
                record.message.replace('\'', "''"),
                serde_json::to_string(&record.fields).unwrap().replace('\'', "''"),
                record.span_id.as_ref().map(|s| format!("'{}'", s)).unwrap_or("NULL".to_string()),
            ));
        }

        sqlx::query(&query).execute(db).await.ok();
        batch.clear();
    }
}
```

### Métricas

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref METRICS: MetricsRegistry = MetricsRegistry::new();
}

pub struct MetricsRegistry {
    // Contadores
    pub orders_placed: AtomicU64,
    pub orders_filled: AtomicU64,
    pub orders_cancelled: AtomicU64,
    pub orders_rejected: AtomicU64,

    // Gauges
    pub active_positions: AtomicU64,
    pub pending_orders: AtomicU64,
    pub queue_size: AtomicU64,

    // Histogramas (usando HdrHistogram ou similar)
    pub order_latency_ms: Histogram,
    pub strategy_eval_time_us: Histogram,
    pub api_response_time_ms: Histogram,
}

impl MetricsRegistry {
    pub fn increment_orders_placed(&self) {
        let count = self.orders_placed.fetch_add(1, Ordering::Relaxed) + 1;
        tracing::debug!(metric = "orders_placed", value = count);
    }

    pub fn record_order_latency(&self, latency_ms: u64) {
        self.order_latency_ms.record(latency_ms);
        tracing::debug!(metric = "order_latency_ms", value = latency_ms);
    }

    /// Exporta métricas em formato Prometheus (opcional)
    pub fn export_prometheus(&self) -> String {
        format!(
            r#"# HELP robotrade_orders_placed Total orders placed
# TYPE robotrade_orders_placed counter
robotrade_orders_placed {}

# HELP robotrade_orders_filled Total orders filled
# TYPE robotrade_orders_filled counter
robotrade_orders_filled {}

# HELP robotrade_active_positions Current active positions
# TYPE robotrade_active_positions gauge
robotrade_active_positions {}
"#,
            self.orders_placed.load(Ordering::Relaxed),
            self.orders_filled.load(Ordering::Relaxed),
            self.active_positions.load(Ordering::Relaxed),
        )
    }
}
```

### Spans para Tracing

```rust
impl TradingWorker {
    #[tracing::instrument(
        skip(self),
        fields(
            strategy_id = %strategy_id,
            symbol = tracing::field::Empty,
        )
    )]
    async fn execute_strategy(&self, strategy_id: Uuid) -> Result<(), Error> {
        let span = tracing::Span::current();

        let strategy = self.load_strategy(strategy_id).await?;
        span.record("symbol", &strategy.symbol);

        // Sub-span para coleta de dados
        let candles = {
            let _guard = tracing::info_span!("fetch_candles").entered();
            self.market_data.get_candles(&strategy.symbol, 100).await?
        };

        // Sub-span para avaliação
        let signal = {
            let _guard = tracing::info_span!("evaluate_signal").entered();
            strategy.evaluate(&candles)?
        };

        tracing::info!(
            signal = ?signal,
            "Strategy evaluation complete"
        );

        if let Signal::Entry { side, strength } = signal {
            let _guard = tracing::info_span!(
                "place_order",
                side = ?side,
                strength = %strength,
            ).entered();

            self.place_order(/* ... */).await?;
        }

        Ok(())
    }
}
```

### Query de Eventos na UI

```rust
#[tauri::command]
pub async fn get_telemetry_events(
    state: State<'_, AppState>,
    filter: TelemetryFilter,
) -> Result<Vec<TelemetryEvent>, String> {
    let events = sqlx::query_as!(
        TelemetryEvent,
        r#"
        SELECT * FROM telemetry_events
        WHERE ($1 IS NULL OR level = $1)
          AND ($2 IS NULL OR target LIKE $2)
          AND ($3 IS NULL OR timestamp >= $3)
          AND ($4 IS NULL OR timestamp <= $4)
        ORDER BY timestamp DESC
        LIMIT $5
        "#,
        filter.level,
        filter.target_pattern,
        filter.from,
        filter.to,
        filter.limit.unwrap_or(1000) as i32,
    )
    .fetch_all(&state.telemetry_db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(events)
}
```

## Consequências

### Positivas

- **Estruturação**: Logs são JSON parseável, não strings livres
- **Performance**: Batching e async write minimizam overhead
- **Flexibilidade**: Múltiplas saídas (console, file, SQLite)
- **Queryable**: Eventos no SQLite podem ser filtrados na UI
- **Ecosystem**: tracing é o padrão do ecossistema Rust async
- **Context propagation**: Spans automaticamente propagam contexto

### Negativas

- **Storage**: Eventos em SQLite crescem rapidamente (requer cleanup)
- **Complexidade**: Setup inicial do subscriber é verboso
- **Overhead mínimo**: Ainda há custo, mesmo que pequeno

### Neutras

- Logs de arquivo são rotacionados diariamente
- Eventos mais antigos que N dias são automaticamente removidos

## Alternativas Consideradas

### Alternativa 1: log + env_logger

- **Descrição**: Crate `log` padrão com env_logger
- **Prós**: Simples, bem conhecido
- **Contras**: Sem spans, sem estruturação nativa
- **Motivo da rejeição**: tracing oferece mais features sem overhead significativo

### Alternativa 2: slog

- **Descrição**: Structured logging alternativo
- **Prós**: Muito estruturado, type-safe
- **Contras**: Menos integração com ecossistema async
- **Motivo da rejeição**: tracing é mais amplamente adotado

### Alternativa 3: OpenTelemetry

- **Descrição**: Stack completa de observabilidade
- **Prós**: Padrão da indústria, integração com Jaeger/Zipkin
- **Contras**: Overkill para app desktop, requer backend
- **Motivo da rejeição**: Não faz sentido para aplicação local

## Referências

- [tracing Documentation](https://tracing.rs/)
- [tracing-subscriber](https://docs.rs/tracing-subscriber/)
- [Structured Logging Best Practices](https://www.honeycomb.io/blog/structured-logging-done-right)
- [Prometheus Exposition Format](https://prometheus.io/docs/instrumenting/exposition_formats/)
