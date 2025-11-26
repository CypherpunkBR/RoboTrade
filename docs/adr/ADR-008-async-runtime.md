# ADR-008: Runtime Assíncrono

## Status

Aceita

## Contexto

O RoboTrade é uma aplicação I/O-bound com múltiplas operações concorrentes:

1. **HTTP Requests**: Chamadas a APIs de exchanges (REST)
2. **WebSockets**: Streams de dados de mercado em tempo real
3. **Database**: Queries e escritas SQLite
4. **File I/O**: Logs, configuração, cache
5. **Timers**: Schedulers, timeouts, intervalos
6. **IPC**: Comunicação Tauri <-> Frontend

Requisitos:

- Alta concorrência sem overhead de threads por conexão
- Cancelamento de operações em andamento
- Timeouts configuráveis
- Integração com Tauri (que já usa async)
- Performance adequada para trading de baixa latência

## Decisão

Adotamos **Tokio** como runtime assíncrono principal, configurado como multi-threaded com work-stealing.

### Configuração do Runtime

```rust
use tokio::runtime::{Builder, Runtime};

pub fn create_runtime() -> Runtime {
    Builder::new_multi_thread()
        .worker_threads(4)  // Ajustável via config
        .enable_all()       // I/O, time, etc.
        .thread_name("robotrade-worker")
        .on_thread_start(|| {
            tracing::debug!("Tokio worker thread started");
        })
        .on_thread_stop(|| {
            tracing::debug!("Tokio worker thread stopped");
        })
        .build()
        .expect("Failed to create Tokio runtime")
}

// Para Tauri, usamos o runtime que ele já fornece
// ou integramos via tauri::async_runtime
```

### Padrões de Concorrência

```rust
use tokio::sync::{mpsc, broadcast, watch, RwLock, Semaphore};
use tokio::time::{timeout, interval, Duration};

/// Canais para comunicação entre componentes
pub struct Channels {
    // mpsc: múltiplos produtores, um consumidor
    pub orders_tx: mpsc::Sender<OrderRequest>,
    pub orders_rx: mpsc::Receiver<OrderRequest>,

    // broadcast: múltiplos produtores, múltiplos consumidores
    pub market_data_tx: broadcast::Sender<MarketDataEvent>,

    // watch: valor mais recente, múltiplos leitores
    pub status_tx: watch::Sender<SystemStatus>,
    pub status_rx: watch::Receiver<SystemStatus>,
}

/// Rate limiting com semáforo
pub struct RateLimiter {
    semaphore: Semaphore,
    refill_interval: Duration,
}

impl RateLimiter {
    pub fn new(permits: usize, refill_interval: Duration) -> Self {
        Self {
            semaphore: Semaphore::new(permits),
            refill_interval,
        }
    }

    pub async fn acquire(&self) -> Result<(), Error> {
        timeout(Duration::from_secs(30), self.semaphore.acquire())
            .await
            .map_err(|_| Error::RateLimitTimeout)?
            .map_err(|_| Error::RateLimitClosed)?;

        // Libera permit após intervalo
        let semaphore = &self.semaphore;
        let interval = self.refill_interval;
        tokio::spawn(async move {
            tokio::time::sleep(interval).await;
            semaphore.add_permits(1);
        });

        Ok(())
    }
}
```

### Cancelamento Graceful

```rust
use tokio_util::sync::CancellationToken;

pub struct ServiceManager {
    cancellation: CancellationToken,
    tasks: Vec<tokio::task::JoinHandle<()>>,
}

impl ServiceManager {
    pub fn new() -> Self {
        Self {
            cancellation: CancellationToken::new(),
            tasks: Vec::new(),
        }
    }

    pub fn spawn_service<F, Fut>(&mut self, name: &str, service: F)
    where
        F: FnOnce(CancellationToken) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send,
    {
        let token = self.cancellation.child_token();
        let task_name = name.to_string();

        let handle = tokio::spawn(async move {
            tracing::info!(service = %task_name, "Service starting");
            service(token).await;
            tracing::info!(service = %task_name, "Service stopped");
        });

        self.tasks.push(handle);
    }

    pub async fn shutdown(&self) {
        tracing::info!("Initiating graceful shutdown");

        // Sinaliza cancelamento para todos os serviços
        self.cancellation.cancel();

        // Aguarda todos terminarem com timeout
        let shutdown_timeout = Duration::from_secs(30);

        for (i, task) in self.tasks.iter().enumerate() {
            match timeout(shutdown_timeout, task).await {
                Ok(Ok(())) => tracing::debug!(task = i, "Task completed"),
                Ok(Err(e)) => tracing::error!(task = i, error = %e, "Task panicked"),
                Err(_) => tracing::warn!(task = i, "Task shutdown timeout"),
            }
        }

        tracing::info!("Shutdown complete");
    }
}

// Exemplo de serviço cancelável
async fn market_data_service(cancel: CancellationToken) {
    let mut interval = interval(Duration::from_secs(60));

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                tracing::info!("Market data service cancelled");
                break;
            }
            _ = interval.tick() => {
                if let Err(e) = fetch_market_data().await {
                    tracing::error!(error = %e, "Failed to fetch market data");
                }
            }
        }
    }
}
```

### Timeouts e Retries

```rust
use backoff::{ExponentialBackoff, future::retry};

/// Wrapper para chamadas com timeout
pub async fn with_timeout<T, F, Fut>(
    duration: Duration,
    operation_name: &str,
    f: F,
) -> Result<T, Error>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, Error>>,
{
    match timeout(duration, f()).await {
        Ok(result) => result,
        Err(_) => {
            tracing::warn!(
                operation = operation_name,
                timeout_ms = duration.as_millis(),
                "Operation timed out"
            );
            Err(Error::Timeout(operation_name.to_string()))
        }
    }
}

/// Retry com backoff exponencial
pub async fn retry_with_backoff<T, F, Fut>(
    operation_name: &str,
    f: F,
) -> Result<T, Error>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, backoff::Error<Error>>>,
{
    let backoff = ExponentialBackoff {
        initial_interval: Duration::from_millis(100),
        max_interval: Duration::from_secs(10),
        max_elapsed_time: Some(Duration::from_secs(60)),
        ..Default::default()
    };

    retry(backoff, || async {
        let result = f().await;
        if let Err(ref e) = result {
            tracing::warn!(
                operation = operation_name,
                error = %e,
                "Operation failed, retrying"
            );
        }
        result
    })
    .await
    .map_err(|e| Error::RetryExhausted(operation_name.to_string(), Box::new(e)))
}
```

### Integração com Tauri

```rust
// main.rs
fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Tauri já roda em contexto async
            let handle = app.handle().clone();

            // Spawn serviços em background
            tauri::async_runtime::spawn(async move {
                let services = setup_services(&handle).await;
                services.run().await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_candles,
            place_order,
            // ...
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Comandos Tauri são automaticamente async
#[tauri::command]
async fn get_candles(
    state: State<'_, AppState>,
    symbol: String,
    limit: u32,
) -> Result<Vec<Candlestick>, String> {
    // Pode usar .await normalmente
    state.market_data
        .get_candles(&symbol, limit)
        .await
        .map_err(|e| e.to_string())
}
```

### WebSocket Management

```rust
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures::{StreamExt, SinkExt};

pub struct WebSocketManager {
    url: String,
    reconnect_delay: Duration,
    cancel: CancellationToken,
}

impl WebSocketManager {
    pub async fn connect_with_reconnect<F, Fut>(
        &self,
        on_message: F,
    ) where
        F: Fn(Message) -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = ()> + Send,
    {
        loop {
            tokio::select! {
                _ = self.cancel.cancelled() => {
                    tracing::info!("WebSocket manager cancelled");
                    break;
                }
                result = self.connect_once(&on_message) => {
                    match result {
                        Ok(()) => {
                            tracing::info!("WebSocket closed normally");
                        }
                        Err(e) => {
                            tracing::error!(error = %e, "WebSocket error");
                        }
                    }

                    // Reconectar após delay
                    tracing::info!(
                        delay_ms = self.reconnect_delay.as_millis(),
                        "Reconnecting WebSocket"
                    );
                    tokio::time::sleep(self.reconnect_delay).await;
                }
            }
        }
    }

    async fn connect_once<F, Fut>(&self, on_message: &F) -> Result<(), Error>
    where
        F: Fn(Message) -> Fut + Send + Sync,
        Fut: std::future::Future<Output = ()> + Send,
    {
        let (ws_stream, _) = connect_async(&self.url).await?;
        let (mut write, mut read) = ws_stream.split();

        // Ping periódico para manter conexão
        let ping_interval = interval(Duration::from_secs(30));
        tokio::pin!(ping_interval);

        loop {
            tokio::select! {
                _ = self.cancel.cancelled() => break,

                _ = ping_interval.tick() => {
                    write.send(Message::Ping(vec![])).await?;
                }

                Some(msg) = read.next() => {
                    match msg? {
                        Message::Text(text) => {
                            on_message(Message::Text(text)).await;
                        }
                        Message::Pong(_) => {
                            // Conexão OK
                        }
                        Message::Close(_) => {
                            tracing::info!("WebSocket received close frame");
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }
}
```

## Consequências

### Positivas

- **Eficiência**: Milhares de conexões com poucos threads
- **Composabilidade**: async/await torna código legível
- **Cancelamento**: CancellationToken permite shutdown limpo
- **Ecossistema**: Tokio tem excelente suporte (reqwest, sqlx, etc.)
- **Performance**: Zero-cost abstractions, sem GC
- **Integração Tauri**: Funciona nativamente com Tauri 2.0

### Negativas

- **Complexidade**: async Rust tem curva de aprendizado
- **Colored functions**: Funções async não mixam facilmente com sync
- **Debugging**: Stack traces podem ser confusos
- **Send bounds**: Lifetimes e Send/Sync podem ser trabalhosos

### Neutras

- Runtime overhead é mínimo (~1ms startup)
- Tokio é a escolha de fato no ecossistema Rust async

## Alternativas Consideradas

### Alternativa 1: async-std

- **Descrição**: Runtime alternativo ao Tokio
- **Prós**: API mais próxima da std
- **Contras**: Ecossistema menor, menos crates compatíveis
- **Motivo da rejeição**: Tokio tem melhor suporte de crates que usamos

### Alternativa 2: smol

- **Descrição**: Runtime minimalista
- **Prós**: Muito pequeno, componível
- **Contras**: Menos features, menos testado em produção
- **Motivo da rejeição**: Preferimos solução mais completa

### Alternativa 3: Threads OS + Canais

- **Descrição**: std::thread com crossbeam channels
- **Prós**: Sem complexidade async, debugging mais fácil
- **Contras**: Não escala para muitas conexões, mais overhead
- **Motivo da rejeição**: Inadequado para workload I/O-bound

## Referências

- [Tokio Documentation](https://tokio.rs/)
- [Async Rust Book](https://rust-lang.github.io/async-book/)
- [tokio-util CancellationToken](https://docs.rs/tokio-util/latest/tokio_util/sync/struct.CancellationToken.html)
- [Tauri Async Runtime](https://docs.rs/tauri/latest/tauri/async_runtime/)
