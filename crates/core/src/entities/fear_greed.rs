//! Entidade Fear & Greed Index
//!
//! Dados do índice de medo e ganância do mercado crypto.
//! Fonte: alternative.me/crypto/fear-and-greed-index/

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

/// Classificação do índice Fear & Greed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FearGreedClassification {
    /// Medo extremo (0-24)
    ExtremeFear,
    /// Medo (25-44)
    Fear,
    /// Neutro (45-55)
    Neutral,
    /// Ganância (56-74)
    Greed,
    /// Ganância extrema (75-100)
    ExtremeGreed,
}

impl FearGreedClassification {
    /// Classifica um valor do índice
    pub fn from_value(value: u8) -> Self {
        match value {
            0..=24 => FearGreedClassification::ExtremeFear,
            25..=44 => FearGreedClassification::Fear,
            45..=55 => FearGreedClassification::Neutral,
            56..=74 => FearGreedClassification::Greed,
            75..=100 => FearGreedClassification::ExtremeGreed,
            _ => FearGreedClassification::ExtremeGreed, // Valores > 100 são tratados como ganância extrema
        }
    }

    /// Retorna se é um momento de medo (potencial compra)
    pub fn is_fearful(&self) -> bool {
        matches!(
            self,
            FearGreedClassification::ExtremeFear | FearGreedClassification::Fear
        )
    }

    /// Retorna se é um momento de ganância (potencial venda)
    pub fn is_greedy(&self) -> bool {
        matches!(
            self,
            FearGreedClassification::ExtremeGreed | FearGreedClassification::Greed
        )
    }

    /// Retorna a descrição em português
    pub fn description_pt(&self) -> &'static str {
        match self {
            FearGreedClassification::ExtremeFear => "Medo Extremo",
            FearGreedClassification::Fear => "Medo",
            FearGreedClassification::Neutral => "Neutro",
            FearGreedClassification::Greed => "Ganância",
            FearGreedClassification::ExtremeGreed => "Ganância Extrema",
        }
    }
}

impl std::fmt::Display for FearGreedClassification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description_pt())
    }
}

/// Dados de um ponto do Fear & Greed Index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FearGreedData {
    /// Valor do índice (0-100)
    pub value: u8,
    /// Classificação textual
    pub classification: FearGreedClassification,
    /// Data do índice
    pub date: NaiveDate,
    /// Timestamp de quando foi coletado
    pub collected_at: DateTime<Utc>,
}

impl FearGreedData {
    /// Cria um novo dado de Fear & Greed
    pub fn new(value: u8, date: NaiveDate) -> Self {
        Self {
            value,
            classification: FearGreedClassification::from_value(value),
            date,
            collected_at: Utc::now(),
        }
    }

    /// Verifica se o índice indica medo extremo (zona de compra agressiva)
    pub fn is_extreme_fear(&self) -> bool {
        self.classification == FearGreedClassification::ExtremeFear
    }

    /// Verifica se o índice indica ganância extrema (zona de venda agressiva)
    pub fn is_extreme_greed(&self) -> bool {
        self.classification == FearGreedClassification::ExtremeGreed
    }

    /// Calcula a distância do valor atual até o limiar de medo extremo
    pub fn distance_to_extreme_fear(&self) -> i16 {
        self.value as i16 - 25
    }

    /// Calcula a distância do valor atual até o limiar de ganância extrema
    pub fn distance_to_extreme_greed(&self) -> i16 {
        75 - self.value as i16
    }
}

/// Histórico de Fear & Greed com estatísticas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FearGreedHistory {
    /// Lista de dados históricos (mais recente primeiro)
    pub data: Vec<FearGreedData>,
    /// Valor atual
    pub current: Option<FearGreedData>,
    /// Média dos últimos 7 dias
    pub avg_7d: Option<f64>,
    /// Média dos últimos 30 dias
    pub avg_30d: Option<f64>,
    /// Valor mínimo no período
    pub min_value: Option<u8>,
    /// Valor máximo no período
    pub max_value: Option<u8>,
}

impl FearGreedHistory {
    /// Cria um novo histórico vazio
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            current: None,
            avg_7d: None,
            avg_30d: None,
            min_value: None,
            max_value: None,
        }
    }

    /// Cria um histórico a partir de uma lista de dados
    pub fn from_data(mut data: Vec<FearGreedData>) -> Self {
        if data.is_empty() {
            return Self::new();
        }

        // Ordena por data (mais recente primeiro)
        data.sort_by(|a, b| b.date.cmp(&a.date));

        let current = data.first().cloned();

        // Calcula médias
        let avg_7d = if data.len() >= 7 {
            let sum: u32 = data.iter().take(7).map(|d| d.value as u32).sum();
            Some(sum as f64 / 7.0)
        } else if !data.is_empty() {
            let sum: u32 = data.iter().map(|d| d.value as u32).sum();
            Some(sum as f64 / data.len() as f64)
        } else {
            None
        };

        let avg_30d = if data.len() >= 30 {
            let sum: u32 = data.iter().take(30).map(|d| d.value as u32).sum();
            Some(sum as f64 / 30.0)
        } else if !data.is_empty() {
            let sum: u32 = data.iter().map(|d| d.value as u32).sum();
            Some(sum as f64 / data.len() as f64)
        } else {
            None
        };

        let min_value = data.iter().map(|d| d.value).min();
        let max_value = data.iter().map(|d| d.value).max();

        Self {
            data,
            current,
            avg_7d,
            avg_30d,
            min_value,
            max_value,
        }
    }

    /// Adiciona um novo dado ao histórico
    pub fn add(&mut self, data: FearGreedData) {
        // Insere no início (mais recente primeiro)
        self.data.insert(0, data.clone());
        self.current = Some(data);

        // Recalcula estatísticas
        self.recalculate_stats();
    }

    /// Recalcula estatísticas do histórico
    fn recalculate_stats(&mut self) {
        if self.data.is_empty() {
            self.avg_7d = None;
            self.avg_30d = None;
            self.min_value = None;
            self.max_value = None;
            return;
        }

        self.avg_7d = if self.data.len() >= 7 {
            let sum: u32 = self.data.iter().take(7).map(|d| d.value as u32).sum();
            Some(sum as f64 / 7.0)
        } else {
            let sum: u32 = self.data.iter().map(|d| d.value as u32).sum();
            Some(sum as f64 / self.data.len() as f64)
        };

        self.avg_30d = if self.data.len() >= 30 {
            let sum: u32 = self.data.iter().take(30).map(|d| d.value as u32).sum();
            Some(sum as f64 / 30.0)
        } else {
            let sum: u32 = self.data.iter().map(|d| d.value as u32).sum();
            Some(sum as f64 / self.data.len() as f64)
        };

        self.min_value = self.data.iter().map(|d| d.value).min();
        self.max_value = self.data.iter().map(|d| d.value).max();
    }

    /// Retorna a tendência do índice (subindo, descendo, estável)
    pub fn trend(&self) -> Option<FearGreedTrend> {
        if self.data.len() < 3 {
            return None;
        }

        let recent: Vec<u8> = self.data.iter().take(3).map(|d| d.value).collect();
        let avg_recent = recent.iter().map(|v| *v as f64).sum::<f64>() / 3.0;

        let older: Vec<u8> = self.data.iter().skip(3).take(3).map(|d| d.value).collect();
        if older.is_empty() {
            return None;
        }
        let avg_older = older.iter().map(|v| *v as f64).sum::<f64>() / older.len() as f64;

        let diff = avg_recent - avg_older;

        if diff > 5.0 {
            Some(FearGreedTrend::Rising)
        } else if diff < -5.0 {
            Some(FearGreedTrend::Falling)
        } else {
            Some(FearGreedTrend::Stable)
        }
    }
}

impl Default for FearGreedHistory {
    fn default() -> Self {
        Self::new()
    }
}

/// Tendência do Fear & Greed Index
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FearGreedTrend {
    /// Índice subindo (mercado ficando mais ganancioso)
    Rising,
    /// Índice caindo (mercado ficando mais medroso)
    Falling,
    /// Índice estável
    Stable,
}

impl std::fmt::Display for FearGreedTrend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FearGreedTrend::Rising => write!(f, "Subindo"),
            FearGreedTrend::Falling => write!(f, "Caindo"),
            FearGreedTrend::Stable => write!(f, "Estável"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classification_from_value() {
        assert_eq!(
            FearGreedClassification::from_value(10),
            FearGreedClassification::ExtremeFear
        );
        assert_eq!(
            FearGreedClassification::from_value(30),
            FearGreedClassification::Fear
        );
        assert_eq!(
            FearGreedClassification::from_value(50),
            FearGreedClassification::Neutral
        );
        assert_eq!(
            FearGreedClassification::from_value(65),
            FearGreedClassification::Greed
        );
        assert_eq!(
            FearGreedClassification::from_value(85),
            FearGreedClassification::ExtremeGreed
        );
    }

    #[test]
    fn test_fear_greed_data() {
        let data = FearGreedData::new(20, NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());

        assert!(data.is_extreme_fear());
        assert!(!data.is_extreme_greed());
        assert!(data.classification.is_fearful());
    }

    #[test]
    fn test_history_averages() {
        let data: Vec<FearGreedData> = (0..10)
            .map(|i| {
                FearGreedData::new(
                    30 + i * 5,
                    NaiveDate::from_ymd_opt(2024, 1, 1).unwrap() + chrono::Duration::days(i as i64),
                )
            })
            .collect();

        let history = FearGreedHistory::from_data(data);

        assert!(history.avg_7d.is_some());
        assert!(history.current.is_some());
        assert_eq!(history.min_value, Some(30));
        assert_eq!(history.max_value, Some(75));
    }
}
