//! Coletor de dados de mercado
//!
//! Responsável por buscar dados de diferentes fontes e armazenar no banco

use std::sync::Arc;
use robotrade_core::error::RoboTradeError;
use robotrade_core::traits::{FearGreedProvider, FearGreedRepository, CandleRepository};
use robotrade_core::entities::{FearGreedData, TimeFrame};
use tracing::{debug, error, info};

/// Coletor de dados de mercado
pub struct DataCollector<F, FR, CR>
where
    F: FearGreedProvider,
    FR: FearGreedRepository,
    CR: CandleRepository,
{
    fear_greed_provider: Arc<F>,
    fear_greed_repo: Arc<FR>,
    candle_repo: Arc<CR>,
}

impl<F, FR, CR> DataCollector<F, FR, CR>
where
    F: FearGreedProvider + Send + Sync,
    FR: FearGreedRepository + Send + Sync,
    CR: CandleRepository + Send + Sync,
{
    /// Cria um novo coletor de dados
    pub fn new(
        fear_greed_provider: Arc<F>,
        fear_greed_repo: Arc<FR>,
        candle_repo: Arc<CR>,
    ) -> Self {
        Self {
            fear_greed_provider,
            fear_greed_repo,
            candle_repo,
        }
    }

    /// Coleta e armazena Fear & Greed Index atual
    pub async fn collect_fear_greed(&self) -> Result<FearGreedData, RoboTradeError> {
        info!("Coletando Fear & Greed Index...");

        // Busca do provider
        let data = self.fear_greed_provider.fetch_current().await?;
        debug!(
            value = %data.value,
            classification = ?data.classification,
            "Fear & Greed coletado"
        );

        // Verifica se já existe para hoje
        let today = chrono::Utc::now().date_naive();
        if let Ok(Some(existing)) = self.fear_greed_repo.find_by_date(today).await {
            debug!("Fear & Greed de hoje já existe, atualizando...");
            // Poderia atualizar se o valor mudou, mas geralmente é o mesmo
            if existing.value != data.value {
                info!(
                    old = %existing.value,
                    new = %data.value,
                    "Fear & Greed atualizado"
                );
                self.fear_greed_repo.save(&data).await?;
            }
        } else {
            // Salva no banco
            self.fear_greed_repo.save(&data).await?;
            info!("Fear & Greed salvo: {} ({:?})", data.value, data.classification);
        }

        Ok(data)
    }

    /// Coleta histórico do Fear & Greed Index
    pub async fn collect_fear_greed_history(&self, days: u32) -> Result<Vec<FearGreedData>, RoboTradeError> {
        info!(days = %days, "Coletando histórico Fear & Greed...");

        let history = self.fear_greed_provider.fetch_history(days).await?;

        // Salva cada registro
        let mut saved_count = 0;
        for data in &history {
            let date = data.collected_at.date_naive();

            // Verifica se já existe
            match self.fear_greed_repo.find_by_date(date).await {
                Ok(Some(_)) => {
                    debug!("Fear & Greed de {} já existe", date);
                }
                Ok(None) => {
                    self.fear_greed_repo.save(data).await?;
                    saved_count += 1;
                }
                Err(e) => {
                    error!("Erro ao verificar Fear & Greed: {}", e);
                }
            }
        }

        info!(
            total = %history.len(),
            saved = %saved_count,
            "Histórico Fear & Greed coletado"
        );

        Ok(history)
    }

    /// Retorna referência ao repositório de Fear & Greed
    pub fn fear_greed_repo(&self) -> &Arc<FR> {
        &self.fear_greed_repo
    }

    /// Retorna referência ao repositório de candles
    pub fn candle_repo(&self) -> &Arc<CR> {
        &self.candle_repo
    }
}

/// Resultado de coleta de candles
#[derive(Debug)]
pub struct CandleCollectionResult {
    /// Símbolo coletado
    pub symbol: String,
    /// Timeframe
    pub timeframe: TimeFrame,
    /// Número de candles coletados
    pub count: usize,
    /// Se houve erro
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    // Tests requerem mocks dos providers e repositories
    // Serão implementados quando tivermos os mocks
}
