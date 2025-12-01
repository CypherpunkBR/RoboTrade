# ❓ FAQ - Perguntas Frequentes

Respostas pras dúvidas mais comuns sobre o RoboTrade.

---

## 🎯 Geral

### O que é o RoboTrade?

É uma **plataforma desktop de trading automatizado** pra futuros de criptomoedas. Tu pode tradear manualmente ou deixar o robô operar sozinho com tuas estratégias.

### É grátis?

Sim! O software é **open source** e gratuito. Tu só paga as taxas normais das exchanges (Binance, Kraken).

### Funciona em qual sistema operacional?

- ✅ **macOS** (testado)
- ✅ **Linux** (testado)
- ✅ **Windows** (deve funcionar, mas menos testado)

### Quais exchanges são suportadas?

- ✅ **Binance Futures** (completo)
- ✅ **Kraken Futures** (completo)
- ⏳ **Binance Spot** (planejado)
- ⏳ **Kraken Spot** (planejado)

---

## 🔑 Setup e Configuração

### Como consigo API keys?

**Binance:**
1. Entra em [binance.com](https://www.binance.com)
2. Vai em **API Management**
3. Cria nova API
4. Habilita **Trading** (NÃO habilita Withdraw!)
5. Anota Key + Secret

**Kraken:**
1. Entra em [futures.kraken.com](https://futures.kraken.com)
2. Vai em **Settings → API**
3. Cria nova API
4. Habilita **Read** e **Trade**
5. Anota Key + Secret

### Minhas credenciais tão seguras?

Sim! O sistema:
- ✅ Armazena localmente (nunca envia pra servidor externo)
- ✅ Usa `.env` que tá no `.gitignore`
- ✅ Não pede permissão de Withdraw
- ✅ Recomenda usar IP Whitelist nas exchanges

**Dica:** Nas exchanges, adiciona teu IP na whitelist pra extra segurança.

### Posso testar sem gastar dinheiro?

**SIM!** De dois jeitos:

1. **Paper Trading** (simulação local)
   - Ativa em Settings → Trading Mode
   - Simula trades sem conectar na exchange
   - Bom pra testar a UI

2. **Testnets das Exchanges**
   - Binance Testnet: [testnet.binancefuture.com](https://testnet.binancefuture.com)
   - Kraken Demo: habilita Demo mode em Settings
   - Dinheiro fake, mas API real!
   - **RECOMENDADO pra aprender!**

### Onde ficam os dados?

- **Config:** `~/.config/robotrade/config.toml`
- **Database:** `~/.config/robotrade/robotrade.db`
- **Logs:** `~/.local/share/robotrade/logs/`

No Windows:
- Config: `%APPDATA%\robotrade\config.toml`
- Database: `%APPDATA%\robotrade\robotrade.db`

---

## 💹 Trading

### Qual alavancagem devo usar?

Depende da tua experiência:

- **Iniciante:** 1-3x (seguro)
- **Intermediário:** 5-10x (razoável)
- **Avançado:** 10-20x (arriscado)
- **Maluco:** 50-125x (vai perder tudo 💀)

**Regra:** Quanto maior a alavancagem, **mais rápido tu perde tudo**.

### O que é melhor: Market ou Limit Order?

**Market Order:**
- ✅ Executa NA HORA
- ✅ Garante que entra
- ❌ Pode ter slippage (preço diferente do esperado)
- **Usa quando:** Quer entrar/sair AGORA

**Limit Order:**
- ✅ Executa no preço EXATO que tu quer
- ✅ Sem slippage
- ❌ Pode não executar nunca
- **Usa quando:** Tem paciência e quer preço específico

### Preciso usar Stop Loss?

**SIM! SEMPRE!**

Tipo, sério. Todo trade deveria ter SL. É tua proteção contra:
- Mercado indo contra ti
- Liquidação
- Perder mais que planejou

**Exceção:** Day trader experiente que monitora 24/7. Mas mesmo assim é arriscado.

### O que é Trailing Stop?

É um **Stop Loss que se move** quando tá no lucro.

**Exemplo:**
1. Compra BTC a $50k com SL a $49k (2% abaixo)
2. Sobe pra $52k → SL move pra $51k (ainda 2% abaixo)
3. Sobe pra $54k → SL move pra $53k
4. Cai pra $53k → Fecha automaticamente com $3k de lucro!

**Vantagem:** Protege lucro e deixa correr quando tá ganhando.

**Desvantagem:** Pode fechar "cedo" em volatilidade.

### Quanto devo arriscar por trade?

**Regra de ouro:** Não arrisca mais que **1-2% do capital total** por trade.

**Exemplo:**
- Capital: $10.000
- Risco por trade: 1% = $100
- Se teu SL é 2% do preço, compra $5.000 de posição
- Se perder (bate SL), perde $100 (1% do capital)

**Com isso:**
- Tu aguenta 50 trades perdendo seguidos sem quebrar
- Dá tempo de aprender e ajustar
- Não explode a conta num dia ruim

### Quais são os horários de maior volume?

**Bitcoin:**
- 🌅 **8h-12h UTC:** Abertura da Ásia
- 🌆 **13h-17h UTC:** Abertura da Europa
- 🌃 **13h-21h UTC:** Abertura dos EUA (maior volume!)

**Final de semana:** Volume BEM menor (spread maior, mais volatilidade).

### Quanto custa fazer um trade?

**Binance Futures:**
- Maker: 0.02% (tu coloca liquidez com limit order)
- Taker: 0.04% (tu tira liquidez com market order)

**Exemplo:**
- Ordem de $1.000 com taker fee
- Taxa: $1.000 × 0.04% = $0.40

**Kraken Futures:**
- Similar, varia por tier

**Dica:** Usa Limit Orders quando possível (taxa menor!).

---

## 🤖 Trading Automático

### O robô garante lucro?

**NÃO!** Nenhum robô garante lucro.

O robô:
- ✅ Executa a estratégia que TU definiu
- ✅ Não fica emocional
- ✅ Roda 24/7
- ❌ NÃO faz milagre
- ❌ NÃO prevê o futuro

**Lucro depende:**
- Qualidade da tua estratégia
- Condições de mercado
- Gestão de risco
- Sorte também (sim, trading tem sorte)

### Como sei se minha estratégia é boa?

**Backtest primeiro!**

1. Pega 1 ano de dados históricos
2. Roda backtest
3. Analisa métricas:
   - Sharpe Ratio >= 1.5
   - Max Drawdown <= 20%
   - Win Rate >= 40%
   - Profit Factor >= 1.5
   - Pelo menos 30 trades

4. Se passar nos critérios, testa em **Paper Trading** por 2-4 semanas
5. Se continuar lucrando, move pra Live com **capital pequeno**
6. Escala aos poucos conforme funciona

**Red Flag:** Se performa PERFEITO no backtest (>90% win rate), provável overfitting!

### O que é overfitting?

É quando a estratégia funciona **MUITO BEM** em dados históricos mas **FALHA** no futuro.

**Como acontece:**
- Tu otimiza demais os parâmetros pro passado
- Estratégia "decora" o histórico ao invés de aprender padrões
- No futuro, não funciona

**Como evitar:**
- Não otimiza demais
- Testa em out-of-sample data (dados que não usou pra otimizar)
- Walk-forward analysis
- Se ganhar 100% dos trades no backtest, tá overfitted!

### Posso deixar rodando sozinho?

**Pode, MAS:**

1. **Monitora diariamente**
   - Vê o PnL diário
   - Confere se tá dentro dos limites
   - Checa logs de erro

2. **Define limites rigorosos**
   - Perda diária máxima: 3-5%
   - Circuit Breaker: SEMPRE LIGADO
   - Max posições: 3-5

3. **Tem um plano de emergência**
   - Se perder X%, para tudo
   - Se bugs aparecerem, desliga
   - Sabe como fechar posições manualmente

4. **Não deixa sem supervisão por dias**
   - Mercado muda
   - Bugs podem aparecer
   - Exchange pode ter problema

**Recomendação:** Checa pelo menos 1x por dia.

---

## 📊 Métricas e Análise

### O que é um bom Win Rate?

Depende do teu Risk/Reward!

- **Risk/Reward 1:1** → precisa >50% win rate pra lucrar
- **Risk/Reward 1:2** → precisa >33% win rate
- **Risk/Reward 1:3** → precisa >25% win rate

**Exemplo:**
- Tu arrisca $100 pra ganhar $300 (1:3)
- Ganha 3 de cada 10 trades (30% win rate)
- Resultado: 3 × $300 - 7 × $100 = $900 - $700 = +$200 lucro!

**Conclusão:** Win rate alto é bom, mas Risk/Reward importa mais!

### O que é Profit Factor?

É **Lucro Bruto / Prejuízo Bruto**.

**Interpretação:**
- < 1.0 = Perdendo dinheiro 🔴
- 1.0 = Breakeven (nem ganha nem perde)
- 1.0 - 1.5 = Lucrando, mas fraco
- 1.5 - 2.0 = Bom! 🟢
- \> 2.0 = Excelente! 🚀
- \> 3.0 = Muito bom (ou overfitting 🤔)

**Exemplo:**
- Ganhou $5.000 em trades vencedores
- Perdeu $2.000 em trades perdedores
- Profit Factor = 5000 / 2000 = **2.5** (excelente!)

### O que é Sharpe Ratio?

É **retorno ajustado por risco**. Tipo, quanto tu ganha por unidade de risco.

**Interpretação:**
- < 0 = Perde dinheiro
- 0 - 1.0 = Ruim
- 1.0 - 2.0 = Bom
- 2.0 - 3.0 = Muito bom
- \> 3.0 = Excelente (ou overfitting)

**Por que importa:**
- Ganhar 50% com drawdown de 5% = Sharpe alto (bom!)
- Ganhar 50% com drawdown de 40% = Sharpe baixo (arriscado!)

### O que é Max Drawdown?

É a **maior queda** do pico até o fundo.

**Exemplo:**
- Capital chegou em $15.000 (pico)
- Depois caiu pra $12.000 (fundo)
- Drawdown = $3.000 = 20%

**Por que importa:**
- Mostra quanto tu "sofre" antes de recuperar
- Drawdown alto = estresse alto
- Ideal: <= 15%
- Aceitável: <= 25%
- Perigoso: > 30%

**Psicológico:**
- Drawdown de 50% precisa de 100% de ganho pra recuperar!
- É difícil psicologicamente aguentar
- Melhor ter drawdown baixo e retorno moderado

---

## ⚠️ Erros e Problemas

### "Error: Invalid API Key"

**Causa:**
- API key tá errada no `.env`
- API key foi deletada na exchange
- API key expirou

**Solução:**
1. Copia/cola de novo (sem espaços extras!)
2. Confere se não deletou na exchange
3. Cria nova API key se necessário

### "Error: Timestamp out of sync"

**Causa:** Relógio do computador tá errado.

**Solução:**
```bash
# macOS/Linux
sudo ntpdate -u time.apple.com

# Ou ajusta manualmente em Configurações do Sistema
```

Binance não aceita requests com timestamp >1000ms de diferença.

### "Error: Insufficient margin"

**Causa:** Não tem margin suficiente pra posição.

**Solução:**
- Diminui a quantidade
- Diminui a alavancagem
- Fecha posições existentes
- Adiciona mais fundos na exchange

### "Error: Rate limit exceeded"

**Causa:** Fez requests demais na API.

**Limites:**
- Binance: 1200 req/min
- Kraken: Variável

**Solução:**
- Espera 1 minuto
- Sistema já tem rate limiting (não deveria acontecer)
- Se acontecer muito, reporta um bug!

### Database locked

**Causa:** Múltiplos processos tentando acessar SQLite.

**Solução:**
1. Fecha todas instâncias do app
2. Espera 5 segundos
3. Abre de novo

Se persistir:
```bash
# Deleta o lock (CUIDADO!)
rm ~/.config/robotrade/robotrade.db-shm
rm ~/.config/robotrade/robotrade.db-wal
```

### App não abre / Tela preta

**Causa:** Erro no startup do Tauri.

**Solução:**
1. Vê os logs no terminal
2. Confere se `.env` existe
3. Confere se dependências tão instaladas:
   ```bash
   npm install
   cargo build
   ```

4. Tenta rodar em modo dev:
   ```bash
   npm run tauri dev
   ```

### "Builds" mas não executa

**Causa:** Falta dependências do sistema.

**macOS:**
```bash
# Instala Homebrew se não tiver
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

**Linux (Ubuntu/Debian):**
```bash
sudo apt update
sudo apt install libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
```

**Linux (Fedora):**
```bash
sudo dnf install webkit2gtk4.0-devel \
    openssl-devel \
    curl \
    wget \
    file \
    gtk3-devel \
    libappindicator-gtk3-devel \
    librsvg2-devel
```

---

## 📈 Trading e Estratégias

### Por que minha ordem não executou?

**Se é Market Order:**
- Deveria executar NA HORA
- Se não executou, vê o erro no console
- Pode ser saldo insuficiente, símbolo inválido, etc

**Se é Limit Order:**
- Só executa quando preço BATER o valor que tu definiu
- Se mercado não chegou lá, fica pendente
- **Normal!** Não é bug.

### Minha estratégia deu prejuízo, e agora?

**Calma! Analisa antes de desistir:**

1. **Foi só um dia ruim?**
   - Trading tem variance
   - Um dia ruim acontece
   - Vê a performance de 1 mês

2. **Tá dentro da expectativa do backtest?**
   - Se backtest mostrou drawdown de 15%, um dia de -5% é normal
   - Se tá MUITO pior que backtest, pode ser:
     - Overfitting (backtest mentiu)
     - Mercado mudou
     - Bug na implementação

3. **Checklist:**
   - ✅ Tá usando Stop Loss?
   - ✅ Risk management configurado?
   - ✅ Alavancagem razoável?
   - ✅ Testou em paper antes?

4. **Se tá perdendo consistente:**
   - PARA!
   - Volta pro paper trading
   - Revisa a estratégia
   - Faz backtest de novo com dados mais recentes

### Como otimizo minha estratégia?

**Cuidado com otimização! Pode virar overfitting.**

**Jeito certo:**

1. **Define hipótese**
   - "Acho que RSI < 30 é bom pra comprar"
   - Não: "Vou testar 1000 parâmetros até achar o melhor"

2. **Testa no backtest**
   - Período: 1 ano
   - Sem mexer nos parâmetros

3. **Walk-forward analysis**
   - Divide em períodos (treino + teste)
   - Otimiza nos dados de treino
   - Valida nos dados de teste
   - Repete pra frente

4. **Testa no paper**
   - Pelo menos 1 mês
   - Se funcionar, move pra live

**Jeito errado:**
- Ficar mexendo nos parâmetros até backtest dar 100% win rate
- Isso é overfitting e não vai funcionar no futuro!

### Quantos trades por dia é normal?

Depende da estratégia:

- **Scalping:** 10-50+ trades/dia
- **Day Trading:** 2-10 trades/dia
- **Swing Trading:** 1-3 trades/semana
- **Position Trading:** 1-2 trades/mês

**Mais trades ≠ mais lucro!**

Na verdade:
- Mais trades = mais taxas
- Mais trades = mais slippage
- Overtrading é um problema comum

**Foco em qualidade, não quantidade!**

---

## 🔧 Técnico

### Como adiciono uma nova exchange?

Vê o [GUIA-DEV.md](./GUIA-DEV.md) → seção "Como Adicionar Features".

Resumão:
1. Cria módulo em `crates/exchange_gateways/src/nova_exchange/`
2. Implementa autenticação
3. Implementa trait `ExchangeGateway`
4. Adiciona testes
5. Registra em `ExchangeId`

### Como adiciono um novo indicador?

```rust
// crates/analytics/src/indicators/meu_indicador.rs
use rust_decimal::Decimal;

pub fn meu_indicador(prices: &[Decimal], period: usize) -> Vec<Decimal> {
    // Tua lógica aqui
    vec![]
}
```

Adiciona ao `mod.rs` e usa!

### Posso contribuir?

**SIM! Contributions são bem-vindas!**

1. Fork o repo
2. Cria branch (`git checkout -b feature/minha-feature`)
3. Implementa + testa
4. Commita (`git commit -m 'feat: add minha feature'`)
5. Push (`git push origin feature/minha-feature`)
6. Abre Pull Request

**Áreas que precisam de ajuda:**
- Mais estratégias
- Mais indicadores
- Testes
- Documentação
- UI/UX
- Performance

### Como reporto um bug?

**GitHub Issues:**
1. Vai em [github.com/USER/RoboTrade/issues](https://github.com)
2. Clica "New Issue"
3. Descreve:
   - O que tu tentou fazer
   - O que esperava que acontecesse
   - O que aconteceu de fato
   - Logs de erro (se tiver)
   - Teu OS (macOS, Linux, Windows)

**Dica:** Quanto mais detalhes, mais fácil de resolver!

---

## 💰 Financeiro

### Quanto posso ganhar com isso?

**Depende TOTALMENTE:**
- Da tua estratégia
- Do mercado
- Da tua gestão de risco
- Da sorte

**Realista:**
- Trader experiente: 5-15% ao mês
- Iniciante: -10% a +5% (aprendendo)
- Robô com estratégia boa: 3-10% ao mês

**Expectativas:**
- Se tu espera dobrar a conta em 1 mês, vai se frustrar
- Crescimento composto de 5-10% ao mês é EXCELENTE
- Preservar capital é mais importante que ganhar

### Quanto devo começar?

**Recomendação:**

1. **Paper Trading:** $0 (grátis)
2. **Testnet:** $0 (dinheiro fake)
3. **Live (primeira vez):** $100-500
4. **Depois de 3 meses lucro:** Escala pra $1.000-5.000
5. **Depois de 6 meses consistente:** Escala mais

**NUNCA:**
- Coloca mais do que pode perder
- Pega empréstimo pra tradear
- Usa dinheiro de conta/aluguel

---

## 🛡️ Segurança

### Minhas API keys são seguras?

**No sistema:** SIM
- Armazenadas localmente
- Nunca enviadas pra servidor externo
- Arquivo `.env` não é commitado no Git

**Na exchange:** Depende de ti
- ✅ Usa IP Whitelist
- ✅ NÃO ativa Withdraw
- ✅ Usa 2FA na conta
- ✅ Muda a senha regularmente

### O que acontece se alguém pegar minha API key?

**Sem permissão de Withdraw:**
- Pode fazer trades
- Pode ver teu saldo
- **NÃO pode sacar** (tua grana tá segura!)

**Com permissão de Withdraw:**
- Pode sacar tudo 💀
- **NUNCA ativa Withdraw nas API keys de trading!**

**Se suspeitar de vazamento:**
1. Deleta a API key na exchange (AGORA!)
2. Cria nova
3. Atualiza o `.env`
4. Muda senha da conta
5. Habilita 2FA se não tiver

### Posso rodar em VPS/servidor?

**SIM!** Mas algumas considerações:

**Vantagens:**
- ✅ Roda 24/7
- ✅ Internet estável
- ✅ Não depende do teu PC

**Cuidados:**
- 🔒 VPS tem IP fixo → adiciona na whitelist
- 🔒 Usa VPS confiável (AWS, Digital Ocean, Linode)
- 🔒 Firewall configurado
- 🔒 Acesso SSH com chave (não senha)
- 🔒 Mantém sistema atualizado

**Setup:**
```bash
# SSH no VPS
ssh user@seu-vps

# Instala dependências
# (vê docs/development/getting-started.md)

# Clona repo
git clone https://github.com/USER/RoboTrade.git
cd RoboTrade

# Configura .env
nano .env

# Build
cargo build --release

# Roda em background
nohup cargo run --release &

# Ou usa systemd pra rodar como serviço
```

---

## 🎓 Aprendendo Trading

### Sou iniciante, por onde começo?

**Passo a passo:**

1. **Aprende o básico de trading**
   - [Investopedia](https://www.investopedia.com/)
   - O que é long/short
   - O que é alavancagem
   - O que são futuros

2. **Pratica no Paper Trading**
   - Pelo menos 1 mês
   - Testa diferentes estratégias
   - Aprende SEM perder dinheiro

3. **Estuda análise técnica**
   - Candlestick patterns
   - Suporte e resistência
   - Indicadores (RSI, MACD)

4. **Entende gestão de risco**
   - Risk/Reward
   - Position sizing
   - Stop Loss placement

5. **Testa no Testnet**
   - Usa API real mas dinheiro fake
   - Sente como é de verdade

6. **Começa PEQUENO no Live**
   - $100-200
   - Alavancagem baixa (2-3x)
   - Aprende com dinheiro real mas pouco

### Livros recomendados

- **"Trading for a Living"** - Alexander Elder
- **"Market Wizards"** - Jack Schwager
- **"The Disciplined Trader"** - Mark Douglas
- **"Technical Analysis of the Financial Markets"** - John Murphy

### Canais/Recursos

- **Investopedia** - Conceitos e definições
- **TradingView** - Análise técnica e ideias
- **Babypips** - Forex, mas conceitos aplicam
- **YouTube** - Procura "crypto futures trading"

**⚠️ CUIDADO:**
- 90% dos "gurus" são scam
- Não compra curso de "ficar rico rápido"
- Não entra em grupos de pump/sinalização
- DYOR (Do Your Own Research)

---

## 🚀 Avançado

### Posso rodar múltiplas estratégias?

**Atualmente:** Uma estratégia por vez no worker.

**Futuro:** Sistema vai suportar múltiplas estratégias em paralelo.

**Workaround:** Roda múltiplas instâncias com configs diferentes (não recomendado).

### Como faço walk-forward analysis?

**Conceito:** Otimiza em período passado, valida em período futuro, repete.

**Implementação:** Ainda não tem CLI pronto, mas o backtest engine suporta.

**Manual:**
```rust
// 1. Treino (Jan-Jun 2023)
let train_candles = get_candles("2023-01-01", "2023-06-30");
let optimized_params = otimizar(train_candles);

// 2. Teste (Jul-Sep 2023)
let test_candles = get_candles("2023-07-01", "2023-09-30");
let test_result = backtest(test_candles, optimized_params);

// 3. Repete
// Treino: Jul-Dez 2023 → Teste: Jan-Mar 2024
// E assim vai...
```

### Como adiciono notificações (Telegram, Discord)?

**Planejado mas não implementado.**

**Como implementar:**

1. Cria trait `NotificationService` (já existe no core!)
2. Implementa `TelegramNotificationService`
3. Usa no worker quando sinal/ordem/risco

**Exemplo (futuro):**
```rust
impl NotificationService for TelegramNotificationService {
    async fn send(&self, notification: &Notification) -> Result<()> {
        let message = format!(
            "{} {}: {} at ${}",
            notification.severity,
            notification.title,
            notification.message,
            notification.price
        );
        
        self.bot.send_message(self.chat_id, &message).await?;
        Ok(())
    }
}
```

---

## 💡 Dicas Pro

### Use logs pra tudo

```rust
use tracing::{info, warn, error};

info!("Ordem criada: {} {} @ {}", order.side, order.symbol, order.price);
warn!("Saldo baixo: {} USDT", balance);
error!("Falha ao conectar Binance: {}", e);
```

Depois tu consegue:
```bash
# Ver todos os logs
tail -f ~/.local/share/robotrade/logs/robotrade.log

# Filtrar por erro
grep ERROR ~/.local/share/robotrade/logs/robotrade.log

# Últimas 100 linhas
tail -100 ~/.local/share/robotrade/logs/robotrade.log
```

### Backtesta em MÚLTIPLOS períodos

Não testa só em 2023!

Testa em:
- Bull market (2021)
- Bear market (2022)
- Lateral (2023)

Se funciona em TODOS, é uma estratégia robusta!

### Monitora correlação entre moedas

BTC sobe → altcoins sobem (geralmente).

Se tu tem posições em BTC + ETH + BNB **long**, tá **3x exposto** ao mesmo risco!

**Melhor:** Diversifica estratégias e direções.

### Usa Decimal, não float!

```rust
// ❌ NUNCA faça isso pra dinheiro
let price: f64 = 50123.456789;
let quantity: f64 = 0.1;
let total = price * quantity;  // Imprecisão!

// ✅ SEMPRE use Decimal
use rust_decimal::Decimal;
let price = Decimal::from_str("50123.456789")?;
let quantity = Decimal::from_str("0.1")?;
let total = price * quantity;  // Precisão perfeita!
```

**Por quê?**
- Floats tem imprecisão
- Em trading, $0.01 importa
- Decimal garante precisão até 28 casas decimais

### Faz backup do banco

```bash
# Backup manual
cp ~/.config/robotrade/robotrade.db ~/Backups/robotrade_$(date +%Y%m%d).db

# Ou automatiza com cron
0 0 * * * cp ~/.config/robotrade/robotrade.db ~/Backups/robotrade_$(date +\%Y\%m\%d).db
```

Teu histórico completo tá no banco!

---

## 🤔 Dúvidas Filosóficas

### Vale a pena fazer trading algorítmico?

**Honestamente:** É DIFÍCIL ganhar consistente.

**Vantagens:**
- Sem emoção
- Roda 24/7
- Disciplina perfeita
- Backtestável

**Desvantagens:**
- Mercado muda (estratégias param de funcionar)
- Competição com instituições e bots profissionais
- Taxas comem lucro
- Difícil criar estratégia vencedora

**Realidade:**
- ~90% dos traders perdem dinheiro
- Dos 10% que ganham, maioria ganha pouco
- Robô não muda essas odds

**Mas:**
- É divertido aprender!
- Ensina disciplina
- Pode ser side income se fizer direito
- Aprende programação + finanças

### Devo largar meu emprego pra tradear?

**NÃO!** Pelo menos não ainda.

**Só considere se:**
- ✅ Lucrando CONSISTENTE por 1-2 anos
- ✅ Lucro > teu salário atual
- ✅ Tem reserva de emergência (6-12 meses)
- ✅ Sabe que pode parar de funcionar a qualquer momento

**Maioria das pessoas:** Trading é side hustle, não carreira.

### Qual o melhor horário pra tradear?

**Maior volume:** 13h-21h UTC (overlap Europa + EUA)

**Menor spread** = mais volume = melhor.

**Evita:**
- Final de semana (volume baixo)
- Feriados (mercado parado)
- 2h-6h UTC (madrugada, baixo volume)

**Mas:** Depende da tua estratégia! Volatilidade baixa pode ser boa pra range trading.

---

## 📞 Suporte

### Onde consigo ajuda?

1. **Documentação:** Lê `/docs`
2. **GitHub Issues:** Reporta bugs
3. **Discord/Telegram:** (se tiver comunidade)
4. **Email:** (se tiver contato)

### Como contribuo financeiramente?

**Projeto open source!** Se quiser apoiar:
- ⭐ Star no GitHub
- 🐛 Reporta bugs
- 💻 Contribui código
- 📖 Melhora docs
- ☕ Buy me a coffee (se tiver link de doação)

---

**Boas trades e bons estudos! 📚🚀**
