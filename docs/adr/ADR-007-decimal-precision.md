# ADR-007: Precisão Decimal para Valores Financeiros

## Status

Aceita

## Contexto

O RoboTrade trabalha com valores financeiros que exigem precisão exata:

1. **Preços**: BTC pode ter preços como 67,234.12345678
2. **Quantidades**: Ordens podem especificar 0.00123456 BTC
3. **Taxas**: Fees de 0.075% precisam ser calculadas corretamente
4. **P&L**: Lucro/prejuízo precisa ser exato para contabilidade
5. **Backtesting**: Erros de arredondamento acumulam ao longo de milhares de trades

Problemas com floating point (f64):
- `0.1 + 0.2 != 0.3` em IEEE 754
- Erros acumulam em operações repetidas
- Comparações de igualdade são não-confiáveis
- Diferentes plataformas podem dar resultados diferentes

## Decisão

Adotamos a crate **rust_decimal** como tipo padrão para todos os valores financeiros.

### Tipo Base

```rust
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

// Constantes compiladas com precisão exata
const ZERO: Decimal = dec!(0);
const ONE_HUNDRED: Decimal = dec!(100);
const DEFAULT_FEE_RATE: Decimal = dec!(0.001); // 0.1%

/// Wrapper com semântica financeira
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Price(Decimal);

impl Price {
    pub fn new(value: Decimal) -> Result<Self, Error> {
        if value < ZERO {
            return Err(Error::NegativePrice);
        }
        Ok(Self(value))
    }

    pub fn from_str(s: &str) -> Result<Self, Error> {
        let value = Decimal::from_str(s)
            .map_err(|_| Error::InvalidPriceFormat)?;
        Self::new(value)
    }

    pub fn inner(&self) -> Decimal {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Quantity(Decimal);

impl Quantity {
    pub fn new(value: Decimal) -> Result<Self, Error> {
        if value <= ZERO {
            return Err(Error::NonPositiveQuantity);
        }
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Money {
    pub amount: Decimal,
    pub currency: Currency,
}
```

### Operações com Precisão Controlada

```rust
use rust_decimal::prelude::*;

impl Price {
    /// Calcula valor total com precisão adequada
    pub fn total_value(&self, quantity: Quantity) -> Money {
        // Multiplicação exata, depois arredonda para moeda
        let raw = self.0 * quantity.0;
        let rounded = raw.round_dp(8); // 8 casas decimais para crypto
        Money {
            amount: rounded,
            currency: Currency::USD, // ou derivar do contexto
        }
    }
}

/// Calcula fee com arredondamento para cima (conservador)
pub fn calculate_fee(value: Decimal, fee_rate: Decimal) -> Decimal {
    let raw_fee = value * fee_rate;
    // Arredonda para cima para não subestimar custos
    raw_fee.round_dp_with_strategy(8, RoundingStrategy::AwayFromZero)
}

/// Calcula P&L considerando fees
pub fn calculate_pnl(
    entry_price: Price,
    exit_price: Price,
    quantity: Quantity,
    entry_fee_rate: Decimal,
    exit_fee_rate: Decimal,
) -> Money {
    let entry_value = entry_price.0 * quantity.0;
    let exit_value = exit_price.0 * quantity.0;

    let entry_fee = calculate_fee(entry_value, entry_fee_rate);
    let exit_fee = calculate_fee(exit_value, exit_fee_rate);

    let gross_pnl = exit_value - entry_value;
    let net_pnl = gross_pnl - entry_fee - exit_fee;

    Money {
        amount: net_pnl.round_dp(8),
        currency: Currency::USD,
    }
}
```

### Serialização

```rust
use serde::{Deserialize, Serialize};

// Serializa como string para preservar precisão em JSON
#[derive(Debug, Serialize, Deserialize)]
pub struct OrderResponse {
    #[serde(with = "rust_decimal::serde::str")]
    pub price: Decimal,

    #[serde(with = "rust_decimal::serde::str")]
    pub quantity: Decimal,

    #[serde(with = "rust_decimal::serde::str")]
    pub filled_quantity: Decimal,
}

// Ou usar o formato opcional de float para compatibilidade
#[derive(Debug, Serialize, Deserialize)]
pub struct LegacyFormat {
    #[serde(with = "rust_decimal::serde::float")]
    pub price: Decimal,
}
```

### Persistência em SQLite

```rust
// SQLite não tem tipo Decimal nativo, então armazenamos como TEXT
impl sqlx::Type<sqlx::Sqlite> for Price {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <String as sqlx::Type<sqlx::Sqlite>>::type_info()
    }
}

impl sqlx::Encode<'_, sqlx::Sqlite> for Price {
    fn encode_by_ref(
        &self,
        buf: &mut Vec<sqlx::sqlite::SqliteArgumentValue<'_>>,
    ) -> sqlx::encode::IsNull {
        self.0.to_string().encode_by_ref(buf)
    }
}

impl sqlx::Decode<'_, sqlx::Sqlite> for Price {
    fn decode(
        value: sqlx::sqlite::SqliteValueRef<'_>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<sqlx::Sqlite>>::decode(value)?;
        let decimal = Decimal::from_str(&s)?;
        Ok(Price(decimal))
    }
}
```

### Validação de Precisão por Exchange

```rust
/// Regras de precisão específicas da Binance
pub struct BinancePrecision {
    pub price_precision: u32,    // Casas decimais permitidas
    pub quantity_precision: u32,
    pub min_quantity: Decimal,
    pub min_notional: Decimal,   // Valor mínimo da ordem em quote
}

impl BinancePrecision {
    /// Ajusta quantidade para precisão da exchange
    pub fn normalize_quantity(&self, qty: Decimal) -> Decimal {
        qty.round_dp(self.quantity_precision)
    }

    /// Ajusta preço para precisão da exchange
    pub fn normalize_price(&self, price: Decimal) -> Decimal {
        price.round_dp(self.price_precision)
    }

    /// Valida ordem contra regras da exchange
    pub fn validate_order(
        &self,
        price: Decimal,
        quantity: Decimal,
    ) -> Result<(), ValidationError> {
        if quantity < self.min_quantity {
            return Err(ValidationError::QuantityTooSmall {
                min: self.min_quantity,
                got: quantity,
            });
        }

        let notional = price * quantity;
        if notional < self.min_notional {
            return Err(ValidationError::NotionalTooSmall {
                min: self.min_notional,
                got: notional,
            });
        }

        Ok(())
    }
}

// Exemplo de configurações para BTCUSDT
const BTCUSDT_PRECISION: BinancePrecision = BinancePrecision {
    price_precision: 2,      // 67234.12
    quantity_precision: 5,   // 0.00123
    min_quantity: dec!(0.00001),
    min_notional: dec!(10),  // Mínimo $10 por ordem
};
```

### Comparações Seguras

```rust
impl Price {
    /// Compara preços com tolerância para evitar problemas de precisão
    pub fn approximately_equal(&self, other: &Price, tolerance: Decimal) -> bool {
        (self.0 - other.0).abs() <= tolerance
    }

    /// Verifica se preço atingiu target
    pub fn reached_target(&self, target: Price, side: Side) -> bool {
        match side {
            Side::Buy => self.0 <= target.0,  // Compra quando preço cai
            Side::Sell => self.0 >= target.0, // Vende quando preço sobe
        }
    }
}

/// Calcula variação percentual com precisão
pub fn percent_change(old: Decimal, new: Decimal) -> Decimal {
    if old.is_zero() {
        return Decimal::ZERO;
    }
    ((new - old) / old) * dec!(100)
}
```

## Consequências

### Positivas

- **Exatidão**: Cálculos financeiros são deterministicos e corretos
- **Portabilidade**: Mesmos resultados em qualquer plataforma
- **Auditabilidade**: Valores podem ser verificados manualmente
- **Comparações confiáveis**: `==` funciona como esperado
- **Overflow seguro**: Decimal tem range muito maior que f64
- **Integração**: Suporte nativo a serde e sqlx

### Negativas

- **Performance**: ~10x mais lento que f64 para operações básicas
- **Memória**: 16 bytes vs 8 bytes de f64
- **Complexidade**: Precisa de conversão de/para APIs que usam float

### Neutras

- A maioria das exchanges retorna preços como strings (já compatível)
- Indicadores técnicos podem usar f64 internamente (não crítico)

## Mitigação de Performance

```rust
/// Para cálculos de indicadores não-críticos, usar f64 internamente
pub struct IndicatorCalculator {
    // Valores internos em f64 para performance
    sma_buffer: Vec<f64>,
}

impl IndicatorCalculator {
    pub fn calculate_sma(&mut self, prices: &[Decimal]) -> Decimal {
        // Converter para f64 para cálculo rápido
        let floats: Vec<f64> = prices.iter()
            .map(|d| d.to_f64().unwrap())
            .collect();

        let sum: f64 = floats.iter().sum();
        let avg = sum / floats.len() as f64;

        // Converter resultado de volta para Decimal
        Decimal::from_f64(avg).unwrap().round_dp(8)
    }
}
```

## Alternativas Consideradas

### Alternativa 1: f64 com Epsilon

- **Descrição**: Usar float com comparações tolerantes
- **Prós**: Performance máxima, tipo nativo
- **Contras**: Erros acumulam, não adequado para contabilidade
- **Motivo da rejeição**: Inaceitável para valores financeiros

### Alternativa 2: Integer com Escala Fixa

- **Descrição**: Armazenar centavos/satoshis como i64
- **Prós**: Performance, sem overhead de parsing
- **Contras**: Escala fixa não funciona para crypto (precisões variadas)
- **Motivo da rejeição**: Inflexível para diferentes precisões

### Alternativa 3: BigDecimal

- **Descrição**: Crate bigdecimal para precisão arbitrária
- **Prós**: Precisão ilimitada
- **Contras**: Muito mais lento, alocações heap
- **Motivo da rejeição**: rust_decimal suficiente e mais performático

## Referências

- [rust_decimal Documentation](https://docs.rs/rust_decimal/)
- [IEEE 754 Floating Point](https://en.wikipedia.org/wiki/IEEE_754)
- [What Every Computer Scientist Should Know About Floating-Point](https://docs.oracle.com/cd/E19957-01/806-3568/ncg_goldberg.html)
- [Binance API - Filters](https://binance-docs.github.io/apidocs/spot/en/#filters)
