//! Teste de conectividade das APIs das exchanges
//!
//! Este exemplo testa as APIs públicas das exchanges sem necessidade de credenciais.
//!
//! Executar com:
//! ```
//! cargo run --example test_apis -p robotrade-exchange-gateways
//! ```

use robotrade_exchange_gateways::{BinanceFuturesClient, KrakenFuturesClient};
use robotrade_exchange_gateways::kraken::CandleInterval;
use robotrade_core::ExchangeGateway;

#[tokio::main]
async fn main() {
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║           TESTE DE CONECTIVIDADE DAS EXCHANGES                ║");
    println!("╚═══════════════════════════════════════════════════════════════╝\n");

    // Testar Binance Futures
    test_binance_futures().await;

    println!("\n{}\n", "─".repeat(65));

    // Testar Kraken Futures
    test_kraken_futures().await;

    println!("\n╔═══════════════════════════════════════════════════════════════╗");
    println!("║                    TESTES CONCLUÍDOS                          ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
}

async fn test_binance_futures() {
    println!("🟡 BINANCE FUTURES (Testnet)");
    println!("   URL: https://testnet.binancefuture.com\n");

    // Criar cliente sem credenciais (para endpoints públicos)
    let client = BinanceFuturesClient::testnet("", "");

    // Teste 1: Ping
    print!("   [1/4] Ping API... ");
    match client.ping().await {
        Ok(()) => println!("✅ OK"),
        Err(e) => println!("❌ Erro: {}", e),
    }

    // Teste 2: Obter preço atual do BTCUSDT
    print!("   [2/4] Preço BTCUSDT... ");
    match client.get_price("BTCUSDT").await {
        Ok(price) => println!("✅ ${}", price.price),
        Err(e) => println!("❌ Erro: {}", e),
    }

    // Teste 3: Obter klines/candles
    print!("   [3/4] Klines BTCUSDT (5 candles, 1h)... ");
    match client.get_klines("BTCUSDT", "1h", Some(5)).await {
        Ok(candles) => {
            println!("✅ {} candles recebidos", candles.len());
            if let Some(last) = candles.last() {
                println!("         Último candle: O={} H={} L={} C={}",
                    last.open, last.high, last.low, last.close);
            }
        }
        Err(e) => println!("❌ Erro: {}", e),
    }

    // Teste 4: Ticker 24h
    print!("   [4/4] Ticker 24h ETHUSDT... ");
    match client.get_ticker_24h("ETHUSDT").await {
        Ok(ticker) => {
            println!("✅");
            println!("         Preço: ${}", ticker.last_price);
            println!("         Variação 24h: {}%", ticker.price_change_percent);
            println!("         Volume 24h: {} USDT", ticker.quote_volume);
        }
        Err(e) => println!("❌ Erro: {}", e),
    }

    // Testar Mainnet também
    println!("\n🟢 BINANCE FUTURES (Mainnet)");
    println!("   URL: https://fapi.binance.com\n");

    let mainnet_client = BinanceFuturesClient::mainnet("", "");

    print!("   [1/2] Ping API... ");
    match mainnet_client.ping().await {
        Ok(()) => println!("✅ OK"),
        Err(e) => println!("❌ Erro: {}", e),
    }

    print!("   [2/2] Preço BTCUSDT... ");
    match mainnet_client.get_price("BTCUSDT").await {
        Ok(price) => println!("✅ ${}", price.price),
        Err(e) => println!("❌ Erro: {}", e),
    }
}

async fn test_kraken_futures() {
    println!("🟡 KRAKEN FUTURES (Demo)");
    println!("   URL: https://demo-futures.kraken.com\n");

    // Criar cliente sem credenciais (para endpoints públicos)
    let client = KrakenFuturesClient::demo("", "");

    // Teste 1: Listar instrumentos (primeiro para verificar parsing)
    print!("   [1/4] Listar instrumentos... ");
    match client.get_instruments().await {
        Ok(instruments) => {
            println!("✅ {} instrumentos encontrados", instruments.len());
            // Mostrar alguns instrumentos
            let btc_instruments: Vec<_> = instruments
                .iter()
                .filter(|i| i.symbol.contains("XBT") || i.symbol.contains("BTC"))
                .take(3)
                .collect();
            for inst in btc_instruments {
                println!("         - {} ({})", inst.symbol, inst.instrument_type);
            }
        }
        Err(e) => println!("❌ Erro: {}", e),
    }

    // Teste 2: Ping (usa get_instruments internamente)
    print!("   [2/4] Ping API... ");
    match client.ping().await {
        Ok(()) => println!("✅ OK"),
        Err(e) => println!("❌ Erro: {}", e),
    }

    // Teste 3: Obter tickers
    print!("   [3/4] Tickers (PI_XBTUSD)... ");
    match client.get_ticker("PI_XBTUSD").await {
        Ok(Some(ticker)) => {
            println!("✅");
            println!("         Bid: ${:.2}", ticker.bid.unwrap_or(0.0));
            println!("         Ask: ${:.2}", ticker.ask.unwrap_or(0.0));
            println!("         Last: ${:.2}", ticker.last.unwrap_or(0.0));
        }
        Ok(None) => println!("⚠️ Ticker não encontrado"),
        Err(e) => println!("❌ Erro: {}", e),
    }

    // Teste 4: Obter candles
    print!("   [4/4] Candles PI_XBTUSD (1h)... ");
    match client.get_candles("PI_XBTUSD", CandleInterval::H1, None, None).await {
        Ok(candles) => {
            println!("✅ {} candles recebidos", candles.len());
            if let Some(last) = candles.last() {
                println!("         Último candle: O={} H={} L={} C={}",
                    last.open, last.high, last.low, last.close);
            }
        }
        Err(e) => println!("❌ Erro: {}", e),
    }

    // Testar Mainnet também
    println!("\n🟢 KRAKEN FUTURES (Mainnet)");
    println!("   URL: https://futures.kraken.com\n");

    let mainnet_client = KrakenFuturesClient::mainnet("", "");

    print!("   [1/2] Ping API... ");
    match mainnet_client.ping().await {
        Ok(()) => println!("✅ OK"),
        Err(e) => println!("❌ Erro: {}", e),
    }

    print!("   [2/2] Ticker PI_XBTUSD... ");
    match mainnet_client.get_ticker("PI_XBTUSD").await {
        Ok(Some(ticker)) => println!("✅ Last: ${:.2}", ticker.last.unwrap_or(0.0)),
        Ok(None) => println!("⚠️ Ticker não encontrado"),
        Err(e) => println!("❌ Erro: {}", e),
    }
}
