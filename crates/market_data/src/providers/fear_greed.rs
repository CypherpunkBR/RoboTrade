//! Provedor de dados do Fear & Greed Index
//!
//! Fonte: https://alternative.me/crypto/fear-and-greed-index/
//! API: https://api.alternative.me/fng/

use async_trait::async_trait;
use reqwest::Client;
use robotrade_core::entities::FearGreedData;
use robotrade_core::error::{MarketDataError, MarketDataResult};
use robotrade_core::traits::FearGreedProvider;
use serde::Deserialize;
use std::time::Duration;
use tracing::{debug, info, warn};

/// URL base da API
const API_BASE_URL: &str = "https://api.alternative.me/fng/";

/// Timeout padrão para requisições (segundos)
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Implementação do provedor Fear & Greed usando alternative.me
pub struct AlternativeMeFearGreedProvider {
  client: Client,
  #[allow(dead_code)]
  timeout: Duration,
}

impl AlternativeMeFearGreedProvider {
  /// Cria uma nova instância do provedor
  pub fn new() -> Self {
    Self::with_timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
  }

  /// Cria uma instância com timeout customizado
  pub fn with_timeout(timeout: Duration) -> Self {
    let client = Client::builder()
      .timeout(timeout)
      .build()
      .expect("Erro ao criar cliente HTTP");

    Self { client, timeout }
  }

  /// Faz uma requisição à API
  async fn fetch(&self, limit: Option<u32>) -> MarketDataResult<ApiResponse> {
    let url = match limit {
      Some(l) => format!("{}?limit={}", API_BASE_URL, l),
      None => API_BASE_URL.to_string(),
    };

    debug!(url = %url, "Requisitando Fear & Greed Index");

    let response = self.client.get(&url).send().await.map_err(|e| {
      if e.is_timeout() {
        MarketDataError::Timeout {
          provider: "alternative.me".into(),
          reason: e.to_string(),
        }
      } else {
        MarketDataError::ProviderUnavailable {
          provider: "alternative.me".into(),
          reason: e.to_string(),
        }
      }
    })?;

    if !response.status().is_success() {
      let status = response.status();
      let body = response.text().await.unwrap_or_default();

      return Err(MarketDataError::ProviderUnavailable {
        provider: "alternative.me".into(),
        reason: format!("HTTP {}: {}", status, body),
      });
    }

    let api_response: ApiResponse = response
      .json()
      .await
      .map_err(|e| MarketDataError::ParseError(format!("Erro ao parsear resposta: {}", e)))?;

    if api_response.metadata.error.is_some() {
      return Err(MarketDataError::ProviderUnavailable {
        provider: "alternative.me".into(),
        reason: api_response
          .metadata
          .error
          .unwrap_or("Erro desconhecido".into()),
      });
    }

    Ok(api_response)
  }
}

impl Default for AlternativeMeFearGreedProvider {
  fn default() -> Self {
    Self::new()
  }
}

#[async_trait]
impl FearGreedProvider for AlternativeMeFearGreedProvider {
  fn provider_id(&self) -> &str {
    "alternative.me"
  }

  async fn fetch_current(&self) -> MarketDataResult<FearGreedData> {
    let response = self.fetch(Some(1)).await?;

    let item = response
      .data
      .first()
      .ok_or_else(|| MarketDataError::ParseError("Resposta vazia".into()))?;

    let data = parse_api_item(item)?;

    info!(
        value = %data.value,
        classification = %data.classification,
        "Fear & Greed Index atual"
    );

    Ok(data)
  }

  async fn fetch_history(&self, days: u32) -> MarketDataResult<Vec<FearGreedData>> {
    let response = self.fetch(Some(days)).await?;

    let mut data: Vec<FearGreedData> = response
      .data
      .iter()
      .filter_map(|item| match parse_api_item(item) {
        Ok(d) => Some(d),
        Err(e) => {
          warn!(error = %e, "Erro ao parsear item do histórico");
          None
        }
      })
      .collect();

    // Ordena por data (mais recente primeiro)
    data.sort_by(|a, b| b.date.cmp(&a.date));

    info!(
        count = %data.len(),
        days = %days,
        "Histórico Fear & Greed carregado"
    );

    Ok(data)
  }

  async fn health_check(&self) -> MarketDataResult<bool> {
    let response = self.fetch(Some(1)).await?;
    Ok(!response.data.is_empty())
  }
}

/// Resposta da API
#[derive(Debug, Deserialize)]
struct ApiResponse {
  data: Vec<ApiDataItem>,
  metadata: ApiMetadata,
}

/// Item de dados da API
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ApiDataItem {
  value: String,
  value_classification: String,
  timestamp: String,
  time_until_update: Option<String>,
}

/// Metadados da API
#[derive(Debug, Deserialize)]
struct ApiMetadata {
  error: Option<String>,
}

/// Converte um item da API para FearGreedData
fn parse_api_item(item: &ApiDataItem) -> MarketDataResult<FearGreedData> {
  let value: u8 = item
    .value
    .parse()
    .map_err(|e| MarketDataError::ParseError(format!("Valor inválido: {}", e)))?;

  // O timestamp da API está em segundos Unix
  let timestamp: i64 = item
    .timestamp
    .parse()
    .map_err(|e| MarketDataError::ParseError(format!("Timestamp inválido: {}", e)))?;

  let datetime = chrono::DateTime::from_timestamp(timestamp, 0)
    .ok_or_else(|| MarketDataError::ParseError("Timestamp inválido".into()))?;

  let date = datetime.date_naive();

  Ok(FearGreedData::new(value, date))
}

#[cfg(test)]
mod tests {
  use super::*;
  use chrono::NaiveDate;

  #[test]
  fn test_parse_api_item() {
    let item = ApiDataItem {
      value: "25".into(),
      value_classification: "Extreme Fear".into(),
      timestamp: "1704067200".into(), // 2024-01-01 00:00:00 UTC
      time_until_update: None,
    };

    let data = parse_api_item(&item).unwrap();
    assert_eq!(data.value, 25);
    assert_eq!(data.date, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
  }

  // Testes de integração (requerem conexão com internet)
  // Descomente para executar manualmente
  /*
  #[tokio::test]
  async fn test_fetch_current() {
      let provider = AlternativeMeFearGreedProvider::new();
      let data = provider.fetch_current().await.unwrap();

      assert!(data.value <= 100);
      println!("Fear & Greed atual: {} - {:?}", data.value, data.classification);
  }

  #[tokio::test]
  async fn test_fetch_history() {
      let provider = AlternativeMeFearGreedProvider::new();
      let history = provider.fetch_history(7).await.unwrap();

      assert!(!history.is_empty());
      println!("Histórico (7 dias): {} registros", history.len());
  }
  */
}
