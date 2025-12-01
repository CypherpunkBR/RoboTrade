# 🚀 RoboTrade - Guia Completo (Português Informal)

Fala, trader! Esse é o guia completo do RoboTrade, seu robô de trading de futuros de cripto. Vou te explicar TUDO que tu pode fazer com esse sistema, de um jeito que qualquer um entende.

---

## 🎯 O que é o RoboTrade?

É uma **plataforma desktop completa** pra fazer trading de futuros de criptomoedas. Tipo, tu pode:

- 📊 **Ver gráficos** de preço com indicadores técnicos
- 💰 **Fazer trades manualmente** (comprar/vender)
- 🤖 **Deixar o robô tradear sozinho** (com tuas estratégias)
- 📈 **Analisar teus resultados** com relatórios fodas
- ⚠️ **Gerenciar risco** automaticamente (pra não perder tudo 😅)
- 🎮 **Testar no paper trading** antes de arriscar grana de verdade

Funciona com **Binance Futures** e **Kraken Futures**, e tem modo testnet pra tu brincar sem gastar nada.

---

## 📦 O que tu precisa pra começar

### 1. Credenciais da Exchange

Tu vai precisar criar API keys nas exchanges. Aqui vai o passo a passo:

#### Binance Futures
1. Entra no [Binance Futures](https://www.binance.com/en/futures)
2. Vai em **API Management**
3. Cria uma nova API key
4. **ATIVA** apenas as permissões de **Trading** (não precisa de Withdraw!)
5. Anota a **API Key** e o **Secret** (NÃO PERDE ISSO!)

**Pra testar sem risco:** Usa o [Binance Futures Testnet](https://testnet.binancefuture.com) antes. É de graça e dá pra errar à vontade!

#### Kraken Futures
1. Entra no [Kraken Futures](https://futures.kraken.com/)
2. Vai em **Settings → API**
3. Cria uma nova API key
4. Habilita **Read** e **Trade**
5. Anota a **API Key** e o **Secret**

**Pra testar:** Kraken tem modo demo também, ativa lá nas configs.

### 2. Configurar o .env

Cria um arquivo `.env` na raiz do projeto com isso:

```bash
# Binance
BINANCE_API_KEY=tua_api_key_aqui
BINANCE_API_SECRET=teu_secret_aqui
BINANCE_TESTNET=true  # Deixa true pra testar, false pra real

# Kraken
KRAKEN_FUTURES_API_KEY=tua_api_key_aqui
KRAKEN_FUTURES_API_SECRET=teu_secret_aqui
KRAKEN_FUTURES_DEMO=true  # Deixa true pra testar, false pra real

# Logging (opcional)
RUST_LOG=info
```

**⚠️ IMPORTANTE:** NÃO COMMITA O .ENV NO GIT! Ele já tá no `.gitignore`, mas fica esperto!

### 3. Instalar e rodar

```bash
# Instalar dependências
npm install

# Rodar em modo dev (com hot reload)
npm run tauri dev

# Build pra produção
npm run tauri build
```

Pronto! O app vai abrir e tu já pode começar a brincar.

---

## 🏠 Dashboard - Tua tela principal

Quando tu abre o app, a primeira coisa que tu vê é o **Dashboard**. Aqui tu tem uma visão geral de TUDO:

### 📊 Cards de Resumo (no topo)

1. **Saldo Total** 💰
   - Mostra quanto tu tem em USDT
   - Passa o mouse em cima → vê o breakdown por exchange e moeda
   - Exemplo: "R$ 10.000 USDT" (se tiver posições, mostra quanto tá bloqueado)

2. **PnL Diário** 📈
   - Quanto tu ganhou ou perdeu HOJE
   - Fica verde se positivo, vermelho se negativo
   - Exemplo: "+R$ 150 (+1.5%)" ou "-R$ 80 (-0.8%)"

3. **Posições Abertas** 📍
   - Quantas posições tu tem abertas agora
   - Click ali → vai direto pra ver elas

4. **Ordens Ativas** 📋
   - Quantas ordens pendentes tu tem (limit orders esperando executar)

### 😨 Fear & Greed Index

Logo abaixo tu vê um **medidor** tipo velocímetro mostrando o sentimento do mercado:

- 🟢 **0-25: Extreme Fear** (mercado tá cagado, pode ser hora de comprar)
- 🔵 **25-45: Fear** (um medinho)
- 🟡 **45-55: Neutral** (mercado de boa)
- 🟠 **55-75: Greed** (galera gananciosa, cuidado)
- 🔴 **75-100: Extreme Greed** (euforia, pode ser topo)

Esse índice vem do [Alternative.me](https://alternative.me/crypto/fear-and-greed-index/) e atualiza a cada 30 minutos.

### 📊 Tabela de Posições

Lista TODAS tuas posições abertas (Binance + Kraken + Paper):

| Símbolo | Lado | Quantidade | Preço Entrada | Preço Atual | PnL | Alavancagem | Duração |
|---------|------|------------|---------------|-------------|-----|-------------|---------|
| BTCUSDT | Long | 0.05 BTC | $50,000 | $51,000 | +$50 (+2%) 🟢 | 10x | 2h 15min |
| ETHUSDT | Short | 1.5 ETH | $3,000 | $2,950 | +$75 (+5%) 🟢 | 5x | 45min |

- **Verde** = tá no lucro
- **Vermelho** = tá no prejuízo
- **Click no símbolo** → vai pra página de Trading com aquele par selecionado

### 📋 Tabela de Ordens

Lista todas ordens que ainda tão esperando executar:

| Símbolo | Tipo | Lado | Preço | Quantidade | Status | Ações |
|---------|------|------|-------|------------|--------|-------|
| BTCUSDT | Limit | Buy | $49,500 | 0.1 BTC | Pending | ❌ Cancelar |
| ETHUSDT | Stop Market | Sell | $2,900 | 2 ETH | Pending | ❌ Cancelar |

- **Limit** = ordem que só executa no preço que tu definiu
- **Stop** = ordem que só executa quando o preço bate num certo nível
- Click em **Cancelar** → cancela a ordem na hora

**Atualização:** O dashboard atualiza sozinho a cada **5 segundos**!

---

## 💹 Trading - Onde tu faz os trades

Aqui é onde a magia acontece! É a tela mais completa do sistema.

### 📊 Gráfico de Preços

Um **gráfico de candlestick** gigante mostrando os preços em tempo real:

#### Controles do Gráfico

1. **Seletor de Símbolo** (topo esquerdo)
   - BTC, ETH, BNB, SOL
   - Click → muda o gráfico
   - Carrega automaticamente os últimos candles

2. **Timeframe** (botões no topo)
   - 1m, 5m, 15m, 30m, 1h, 4h, 1d, 1w
   - Muda o período de cada vela

3. **Médias Móveis** (toggle no canto)
   - SMA 20 (linha amarela)
   - SMA 50 (linha verde)
   - EMA 20 (linha roxa tracejada)
   - Liga/desliga como quiser

4. **Linhas de Ordem** (aparecem automaticamente)
   - 🟢 Verde = ordens de compra (limit buy)
   - 🔴 Vermelho = ordens de venda (limit sell)
   - Mostra o preço de cada ordem

5. **Linhas de Posição** (se tu tiver posição aberta)
   - 🔵 Azul = preço de entrada (se long)
   - 🟠 Laranja = preço de entrada (se short)
   - 🔴 Vermelho tracejado = Stop Loss
   - 🟢 Verde tracejado = Take Profit
   - 🔴 Pontilhado = Preço de Liquidação (CUIDADO!)

#### Interação com o Gráfico

- **Click no gráfico** → define o preço automaticamente no formulário de ordem
- **Scroll** → zoom in/out
- **Arrasta** → move o gráfico pra ver histórico

### 📝 Formulário de Ordem (lado direito)

Aqui tu cria tuas ordens:

#### 1. Escolhe o LADO
- **Buy (Long)** 🟢 → tu aposta que o preço vai SUBIR
- **Sell (Short)** 🔴 → tu aposta que o preço vai CAIR

#### 2. Escolhe o TIPO de Ordem
- **Market** 🚀
  - Executa NA HORA no preço atual de mercado
  - Usa quando tu quer entrar AGORA
  - Pode ter slippage (executar num preço ligeiramente diferente)

- **Limit** 🎯
  - Só executa no preço que TU definir (ou melhor)
  - Exemplo: "Só compro BTC se cair pra $49,500"
  - Fica pendente até executar ou tu cancelar
  - Sem slippage garantido!

- **Stop Market** 🛑
  - Vira uma ordem Market quando o preço bate num nível
  - Exemplo: "Se BTC cair pra $48,000, vende tudo no mercado"
  - Usado pra Stop Loss ou pra entrar em rompimentos

- **Stop Limit** 🎯🛑
  - Vira uma ordem Limit quando o preço bate num nível
  - Mais controle que Stop Market, mas pode não executar
  - Exemplo: "Se BTC cair pra $48,000, coloca ordem de venda a $47,900"

#### 3. Define a QUANTIDADE
- Quanto tu quer comprar/vender
- Exemplo: 0.1 BTC, 2 ETH
- O sistema calcula automaticamente o valor em USDT

#### 4. Define o PREÇO (se for Limit ou Stop Limit)
- Click no gráfico → preenche automaticamente
- Ou digita manual

#### 5. Escolhe a ALAVANCAGEM 🎰
- Slider de **1x até 125x**
- **⚠️ CUIDADO:** Alavancagem alta = lucro alto MAS também prejuízo alto
- Iniciante? Usa 2x-5x no máximo!
- Pro? Até 20x é razoável
- Maluco? 125x (vai perder tudo rápido 💀)

**Como funciona:**
- 10x = tu controla 10x mais que tu tem
- Exemplo: Com R$100 e 10x, tu controla R$1.000
- Se subir 10%, tu ganha R$100 (100% do teu capital!)
- Mas se cair 10%, tu PERDE TUDO (liquidado)

#### 6. Stop Loss e Take Profit (OPCIONAL mas RECOMENDADO!)

**Stop Loss (SL)** 🛡️
- Preço que tu SAIR automaticamente no PREJUÍZO
- Limita tuas perdas
- Exemplo: Comprou BTC a $50k com SL a $49k → se cair pra $49k, vende sozinho
- **SEMPRE USA SL!** Sério, tu vai agradecer depois.

**Take Profit (TP)** 🎯
- Preço que tu SAIR automaticamente no LUCRO
- Garante teu gain
- Exemplo: Comprou BTC a $50k com TP a $52k → se subir pra $52k, vende sozinho

**Dica:** Usa SL mais perto que TP (risk/reward). Exemplo:
- Entrada: $50k
- SL: $49k (risco de $1k)
- TP: $53k (lucro de $3k)
- Risk/Reward = 1:3 (ideal!)

#### 7. Botão de ENVIAR

- **Buy Long** 🟢 ou **Sell Short** 🔴
- Mostra resumo da ordem
- Confirma → ordem enviada pra exchange!

### 💰 Painel de Saldos

Mostra todos teus ativos:

| Ativo | Livre | Bloqueado | Total |
|-------|-------|-----------|-------|
| USDT | 9,500 | 500 | 10,000 |
| BTC | 0.05 | 0 | 0.05 |

- **Livre** = tu pode usar agora
- **Bloqueado** = tá em margin de posição ou ordem pendente

### 🎮 Painel de Gestão de Risco

Aqui tu controla o sistema automático:

#### Status
- **Trading Status** 🟢/🔴
  - Ativo = sistema pode operar
  - Desativado = sistema pausado

- **Circuit Breaker** ⚡
  - Ativo = se perder demais, para tudo
  - Desativado = sem proteção (perigoso!)

- **PnL Diário** 📊
  - Quanto tu ganhou/perdeu hoje

- **Exposição Total** 💸
  - Quanto tu tem exposto no mercado

- **Posições Abertas** 📍
  - Quantas posições tu tem

#### Controles
- **Start Worker** ▶️ → inicia o robô de trading automático
- **Stop Worker** ⏸️ → pausa o robô
- **Enable Trading** ✅ → permite trades
- **Disable Trading** ❌ → bloqueia novos trades
- **Reset Daily Losses** 🔄 → zera o contador de perda diária

**Atualização:** A página de Trading atualiza a cada **3 segundos**!

---

## 📊 Charts - Gráficos e Alertas de Preço

Essa tela é pra tu **ANALISAR** o mercado e **CRIAR ALERTAS**.

### 📈 Gráfico Gigante

Mesma coisa que na tela de Trading, mas com:
- **Altura maior** (550px) → melhor visualização
- **Mais símbolos** disponíveis: BTC, ETH, BNB, SOL, XRP, ADA, DOGE, AVAX

### 🔔 Sistema de Alertas de Preço

Aqui tu cria notificações quando o preço bate num nível que tu quer!

#### Como Criar um Alerta

1. **Click no gráfico** no preço que tu quer
   - O preço vai automaticamente pro formulário

2. **Escolhe o Símbolo** (BTC, ETH, etc)

3. **Escolhe a CONDIÇÃO:**
   - **Above (Acima)** 📈
     - Dispara quando preço fica ACIMA do valor
     - Exemplo: "Avisa quando BTC passar de $52k"
   
   - **Below (Abaixo)** 📉
     - Dispara quando preço fica ABAIXO do valor
     - Exemplo: "Avisa quando BTC cair pra $48k"
   
   - **Cross Above (Cruza Acima)** ⬆️
     - Dispara quando preço CRUZA o valor subindo
     - Só dispara UMA VEZ no momento do cruzamento
     - Exemplo: "Avisa quando BTC romper $50k subindo"
   
   - **Cross Below (Cruza Abaixo)** ⬇️
     - Dispara quando preço CRUZA o valor descendo
     - Exemplo: "Avisa quando BTC romper $50k caindo"
   
   - **Percent Up (% Subiu)** 📊
     - Dispara quando preço sobe X% do valor de referência
     - Exemplo: "Avisa quando BTC subir 5% de $50k" (dispara em $52.5k)
   
   - **Percent Down (% Caiu)** 📊
     - Dispara quando preço cai X% do valor de referência
     - Exemplo: "Avisa quando BTC cair 3% de $50k" (dispara em $48.5k)

4. **Define o PREÇO** (ou %)

5. **Mensagem Customizada** (opcional)
   - O que tu quer que apareça quando disparar
   - Exemplo: "BTC rompeu resistência! 🚀"

6. **Recorrente?** 🔄
   - ON = dispara toda vez que a condição for verdadeira
   - OFF = dispara só uma vez e desativa

7. **Criar Alerta** → pronto!

#### Gerenciar Alertas

A lista mostra todos teus alertas:

| Status | Símbolo | Condição | Preço | Mensagem | Disparos | Ações |
|--------|---------|----------|-------|----------|----------|-------|
| 🟢 Ativo | BTCUSDT | Above | $52,000 | Topo! | 3x | ⏸️ ❌ |
| 🔴 Inativo | ETHUSDT | Below | $2,900 | Fundo! | 0x | ▶️ ❌ |
| 🟡 Disparado | BNBUSDT | Cross Above | $300 | Rompeu! | 1x | 🔄 ❌ |

**Badges de Status:**
- 🟢 **Ativo** = monitorando
- 🔴 **Inativo** = pausado (tu desativou)
- 🟡 **Disparado** = acabou de disparar (se não for recorrente, desativa)

**Ações:**
- ▶️ **Ativar** → volta a monitorar
- ⏸️ **Desativar** → pausa (não deleta)
- ❌ **Excluir** → remove permanentemente

---

## 📜 History - Histórico de Trades

Aqui tu vê TUDO que já rolou.

### Duas Abas:

#### 1. Fills (Execuções) 📋

Lista TODAS as execuções de ordem que aconteceram:

| Exchange | Símbolo | Lado | Preço | Quantidade | Taxa | PnL | Timestamp |
|----------|---------|------|-------|------------|------|-----|-----------|
| Binance | BTCUSDT | Buy | $50,123.45 | 0.1 BTC | $2.50 | - | 27/11 14:32 |
| Binance | BTCUSDT | Sell | $51,234.56 | 0.1 BTC | $2.56 | +$106.44 | 27/11 16:15 |
| Kraken | ETHUSDT | Buy | $3,001.23 | 2 ETH | $3.00 | - | 27/11 15:00 |

**Suporta:** Binance + Kraken (busca direto das exchanges!)

**Dados importantes:**
- **PnL** = só aparece quando tu fecha a posição
- **Taxa** = quanto tu pagou de comissão pra exchange
- **Timestamp** = quando executou

#### 2. Trades Completos 💰

Lista os trades do início ao fim (entrada + saída):

| Símbolo | Entrada | Saída | PnL | ROI | Duração | Motivo |
|---------|---------|-------|-----|-----|---------|--------|
| BTCUSDT | $50,000 | $51,500 | +$150 🟢 | +3% | 2h 15m | Take Profit |
| ETHUSDT | $3,000 | $2,950 | -$75 🔴 | -2.5% | 45m | Stop Loss |
| BNBUSDT | $310 | $320 | +$100 🟢 | +3.2% | 1h 30m | Manual |

**Cores:**
- 🟢 Verde = lucro
- 🔴 Vermelho = prejuízo

**Motivo de Fechamento:**
- **Take Profit** = atingiu o TP
- **Stop Loss** = atingiu o SL
- **Manual** = tu fechou na mão
- **Signal** = estratégia automática decidiu fechar
- **Liquidation** = foi liquidado (perda total 💀)

### 📊 Estatísticas de Trading

Um card no topo mostra teu desempenho geral:

```
📊 Estatísticas Gerais
━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Total de Trades:     45
Trades Vencedoras:   28 (62.22%)
Trades Perdedoras:   17 (37.78%)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Win Rate:            62.22% 🟢
Lucro Total:         +$5,234.56
Profit Factor:       2.15 (excelente!)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Média de Ganho:      $258.38
Média de Perda:      -$198.01
Maior Ganho:         $1,234.56 🤑
Maior Perda:         -$567.89 😢
━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Total de Taxas:      $635.79
```

**Interpretação:**
- **Win Rate > 50%** = tu acerta mais que erra (bom!)
- **Profit Factor > 1.5** = lucro > prejuízo (ótimo!)
- **Profit Factor > 2.0** = tá mandando bem! 🚀

### Filtros
- **Por Símbolo:** Mostra só BTC, ou só ETH, etc
- **Limite:** Mostra últimos 50, 100, 200 trades

---

## 📈 Reports - Relatórios Visuais

A tela mais LINDA do sistema! Gráficos que mostram teu desempenho.

### Filtro de Período

No topo tu escolhe o período:
- **Hoje** → só trades de hoje
- **Esta Semana** → últimos 7 dias
- **Este Mês** → últimos 30 dias
- **Todo Período** → desde sempre

### 📊 Cards de Resumo

Quatro cards grandes mostrando:
1. **Total de Trades** → ex: "45 trades"
2. **PnL Total** → ex: "+$5,234.56" (verde se positivo)
3. **Win Rate** → ex: "62.22%"
4. **Profit Factor** → ex: "2.15"

### 📈 Gráficos Interativos

#### 1. PnL ao Longo do Tempo (Linha)
- Eixo X = Tempo
- Eixo Y = PnL Cumulativo
- Mostra como teu lucro evoluiu
- Passa o mouse → vê valor exato em cada ponto

**Interpretação:**
- Linha subindo = lucrando 🟢
- Linha descendo = perdendo 🔴
- Linha estável = nem perde nem ganha

#### 2. Distribuição Win/Loss (Pizza)
- Verde = % de trades vencedoras
- Vermelho = % de trades perdedoras
- Clique nas fatias → destaca

**Ideal:** Mais de 50% verde

#### 3. Curva de Capital (Equity Curve)
- Mostra evolução do teu capital total
- Eixo X = Tempo
- Eixo Y = Capital em USDT
- **A linha MAIS importante!**

**Como analisar:**
- Subindo consistente = estratégia funcionando
- Descendo = tá perdendo, revê a estratégia
- Flat (reto) = breakeven, não tá ganhando nem perdendo
- Muita volatilidade = risco alto

#### 4. PnL por Símbolo (Barras)
- Barra verde = lucro naquele símbolo
- Barra vermelha = prejuízo naquele símbolo
- Mostra qual moeda tá te dando mais retorno

**Exemplo:**
- BTC: +$2,500 (maior lucro)
- ETH: +$1,800
- BNB: -$500 (prejuízo)
- SOL: +$934

**Interpretação:** Talvez vale a pena focar mais em BTC e evitar BNB 🤔

### 📊 Tabela de Estatísticas Detalhadas

Uma tabela expandida com TUDO:

| Métrica | Valor |
|---------|-------|
| Total de Trades | 45 |
| Trades Vencedoras | 28 |
| Trades Perdedoras | 17 |
| Win Rate | 62.22% |
| Gross Profit | $7,234.56 |
| Gross Loss | $-3,365.21 |
| Net Profit | $5,234.56 |
| Profit Factor | 2.15 |
| Expectancy | $116.33 por trade |
| Avg Win | $258.38 |
| Avg Loss | $-198.01 |
| Largest Win | $1,234.56 |
| Largest Loss | $-567.89 |
| Total Fees | $635.79 |
| Avg Trade Duration | 1h 45min |

---

## ⚙️ Settings - Configurações

Aqui tu personaliza TUDO no sistema.

### 🎮 Modo de Trading

**Toggle GIGANTE no topo:**
- 🟢 **Live Trading** = DINHEIRO REAL (cuidado!)
- 🟡 **Paper Trading** = Simulação (sem risco)

**⚠️ ATENÇÃO ao mudar pra Live:**
- Aparece um alerta vermelho
- "Tem certeza? Isso usa DINHEIRO REAL!"
- Confirma duas vezes

**Dica:** Sempre testa no Paper primeiro!

### 🏦 Configuração de Exchanges

#### Binance
- **Testnet** ON/OFF
  - ON = usa testnet.binancefuture.com (grana fake)
  - OFF = usa fapi.binance.com (grana real!)

#### Kraken
- **Demo** ON/OFF
  - ON = usa demo.futures.kraken.com (grana fake)
  - OFF = usa futures.kraken.com (grana real!)

### 💹 Configurações de Trading

1. **Alavancagem Padrão**
   - Slider 1-125x
   - Valor que vem selecionado quando tu cria uma ordem
   - Recomendado: 5-10x

2. **Alavancagem Máxima**
   - Máximo que o sistema permite
   - Previne tu de fazer cagada 😅
   - Recomendado: 20x (no máximo)

3. **Tamanho Padrão da Ordem**
   - % do saldo que tu quer usar por trade
   - Exemplo: 10% = se tu tem $1000, cada trade usa $100
   - Recomendado: 5-10%

4. **Trailing Stop**
   - ON/OFF
   - Move o Stop Loss automaticamente quando tá no lucro
   - Protege lucro e deixa correr 🏃

**Como funciona o Trailing Stop:**
- Comprou BTC a $50k com SL a $49k
- Subiu pra $52k → SL move pra $51k (2% abaixo)
- Subiu pra $54k → SL move pra $53k
- Caiu pra $53k → fecha no lucro de $3k!
- Sem trailing, tu fecharia manualmente ou esperaria o TP

### ⚠️ Gestão de Risco

**MUITO IMPORTANTE!** Essas configs protegem teu capital.

1. **Perda Diária Máxima**
   - % do capital que tu aceita perder NUM DIA
   - Exemplo: 5% = se perder $500 num dia (de $10k), para tudo
   - Recomendado: 3-5%
   - **Circuit Breaker dispara se atingir!**

2. **Tamanho Máximo por Posição**
   - % máximo do capital em UMA posição
   - Exemplo: 20% = no máximo $2k numa posição (de $10k total)
   - Previne concentração de risco
   - Recomendado: 10-20%

3. **Máximo de Posições Simultâneas**
   - Quantas posições abertas ao mesmo tempo
   - Exemplo: 5 = no máximo 5 pares abertos junto
   - Previne overtrading
   - Recomendado: 3-5

4. **Circuit Breaker**
   - ON/OFF
   - Liga/desliga a proteção automática
   - **Deixa SEMPRE LIGADO!**

**Como funciona o Circuit Breaker:**
1. Tu perde 5% num dia (atingiu a perda máxima diária)
2. Circuit Breaker **DISPARA**
3. Sistema **fecha todas posições** na hora
4. Sistema **bloqueia novos trades**
5. Tu não perde mais que os 5% definidos
6. No dia seguinte, reseta e volta ao normal

É tipo um fusível que queima pra não pegar fogo! 🔥

### 💾 Salvar Configurações

Tudo que tu muda aqui salva automaticamente no arquivo `config.toml`.

---

## 🤖 Trading Automático - Como funciona o Worker

O **Trading Worker** é o robô que opera sozinho seguindo tuas estratégias.

### Como ativar

1. Vai em **Trading** → Painel de Gestão de Risco
2. Clica em **Start Worker** ▶️
3. Pronto! O robô tá rodando

### O que ele faz

1. **Coleta dados** de mercado a cada minuto
2. **Avalia estratégias** (ex: Fear & Greed)
3. **Gera sinais** de compra/venda
4. **Valida com Risk Manager:**
   - Tá dentro dos limites?
   - Não vai estourar o risco?
   - Circuit Breaker tá ok?
5. **Se OK:** Envia ordem pra exchange
6. **Se NÃO:** Ignora o sinal

### Controles do Worker

- **Start Worker** ▶️ → inicia o robô
- **Stop Worker** ⏸️ → pausa (posições ficam abertas)
- **Enable Trading** ✅ → permite que ele faça trades
- **Disable Trading** ❌ → bloqueia novos trades (útil pra pausar sem parar)

### Estratégias Disponíveis

#### Fear & Greed Strategy

A única implementada por padrão:

**Lógica:**
- Compra quando **Fear ≤ 25** (extreme fear)
- Vende quando **Greed ≥ 75** (extreme greed)
- Ideia: "Buy the fear, sell the greed"

**Configuração:**
- Buy Threshold: 25 (ajustável)
- Sell Threshold: 75 (ajustável)
- Position Size: % do capital por trade

**Performance:** Funciona melhor em mercados laterais/rangebound.

### Como adicionar mais estratégias

Tu pode criar tuas próprias! O sistema já tem:
- RSI
- MACD
- Bollinger Bands
- SMA/EMA
- ATR

Só precisa implementar o trait `Strategy` em `crates/analytics/src/strategies/`.

Exemplo rápido:
```rust
pub struct MyStrategy {
    // tuas configs
}

impl Strategy for MyStrategy {
    async fn evaluate(&self, candles: &[Candle]) -> Result<Option<Signal>> {
        // tua lógica aqui
        // retorna Buy, Sell ou None
    }
}
```

---

## 📚 Dicas e Boas Práticas

### Pra Iniciantes 🌱

1. **Começa no PAPER TRADING!**
   - Sério, testa por pelo menos 1 semana
   - Se tu não lucra no paper, não vai lucrar no real

2. **Usa alavancagem BAIXA**
   - Começa com 2x-5x
   - Alavancagem alta parece legal mas tu vai se fuder

3. **SEMPRE usa Stop Loss**
   - Toda posição precisa de SL
   - Define antes de entrar no trade
   - Não move o SL pra baixo depois (aceita a perda!)

4. **Risk Management é FUNDAMENTAL**
   - Não arrisca mais que 1-2% do capital por trade
   - Define perda diária máxima (3-5%)
   - Deixa o Circuit Breaker LIGADO

5. **Anota teus trades**
   - Usa a aba Reports pra revisar
   - Vê o que funciona e o que não funciona
   - Aprende com os erros

### Pra Intermediários 📈

1. **Otimiza teu Risk/Reward**
   - Mínimo 1:2 (arrisca $1 pra ganhar $2)
   - Ideal: 1:3
   - Usa o ATR pra definir SL/TP

2. **Diversifica**
   - Não fica só no BTC
   - Testa ETH, BNB, SOL
   - Vê qual moeda funciona melhor pra ti no Reports

3. **Backtesta antes**
   - Testa a estratégia em dados históricos
   - Se não funcionou no passado, não vai funcionar agora

4. **Monitora a Performance**
   - Equity Curve tem que subir consistente
   - Se tiver drawdown >20%, para e revê
   - Profit Factor tem que ser >1.5

### Pra Avançados 🚀

1. **Cria tuas próprias estratégias**
   - Combina indicadores (RSI + MACD, etc)
   - Testa diferentes timeframes
   - Otimiza parâmetros com walk-forward analysis

2. **Automação Completa**
   - Deixa o Worker rodar 24/7
   - Define alertas de risco
   - Monitora via logs

3. **Multi-Exchange**
   - Opera Binance + Kraken
   - Aproveita diferenças de spread
   - Diversifica risco de exchange

4. **Análise Profunda**
   - Usa os dados do banco SQLite
   - Exporta pra análise externa
   - Correlaciona com métricas on-chain

---

## ⚠️ Avisos Importantes

### Segurança 🔐

1. **NUNCA commita o .env no Git!**
   - Tuas API keys são tipo senha
   - Se vazar, podem roubar tua grana

2. **Usa IP Whitelist nas exchanges**
   - Binance e Kraken permitem restringir por IP
   - Adiciona só o IP da tua casa/VPS

3. **NÃO ativa permissão de Withdraw nas API keys**
   - Só precisa de Read + Trade
   - Sem Withdraw, se alguém pegar tua key, não pode sacar

4. **Backup do banco de dados**
   - `~/.config/robotrade/robotrade.db`
   - Faz backup de vez em quando
   - Tem todo teu histórico lá

### Riscos de Trading ⚠️

1. **Tu PODE perder dinheiro**
   - Trading é arriscado
   - Nunca investe o que não pode perder
   - Mercado cripto é volátil pra caralho

2. **Alavancagem é perigosa**
   - Pode liquidar em minutos
   - 100x = aposta de cassino
   - Usa com responsabilidade

3. **Robô não é mágica**
   - Não garante lucro
   - Estratégia pode parar de funcionar
   - Mercado muda, adapta tua estratégia

4. **Taxas somam**
   - Cada trade paga comissão (0.04% Binance)
   - Overtrading = muito custo
   - Monitora as taxas totais no History

### Legal 📜

**Este software é fornecido "como está":**
- Sem garantias de lucro
- Use por sua conta e risco
- Desenvolvedor não se responsabiliza por perdas
- Não é conselho financeiro
- Faça sua própria pesquisa (DYOR!)

---

## 🆘 Problemas Comuns

### "Connection Error" no Dashboard

**Causa:** Credenciais erradas ou API fora do ar

**Solução:**
1. Confere o `.env`
2. Testa as API keys no site da exchange
3. Vê se não bloqueou por IP
4. Testa com Testnet primeiro

### "Insufficient Balance" ao criar ordem

**Causa:** Saldo insuficiente (óbvio 😅)

**Solução:**
1. Vê o saldo no painel
2. Diminui a quantidade
3. Diminui a alavancagem
4. Fecha posições abertas pra liberar margin

### Ordem não executa (fica Pending)

**Causa:** Preço da Limit Order não bateu

**Explicação:**
- Limit Order só executa no preço que TU definiu
- Se o mercado não chegar lá, fica pendente pra sempre

**Solução:**
- Ajusta o preço pra mais perto do mercado
- Ou cancela e usa Market Order

### Liquidado! 💀

**Causa:** Preço foi contra ti e atingiu o preço de liquidação

**O que rolou:**
- Alavancagem muito alta
- Sem Stop Loss
- Mercado moveu rápido demais

**Como evitar:**
- USA STOP LOSS!
- Alavancagem menor
- Monitora o preço de liquidação (linha vermelha pontilhada no gráfico)

### Worker não tá fazendo trades

**Checklist:**
1. Worker tá rodando? (botão deve mostrar "Running")
2. Trading tá habilitado? (Enable Trading = ON)
3. Tem saldo disponível?
4. Não atingiu limites de risco?
5. Estratégia tá gerando sinais? (vê os logs)

---

## 🎓 Glossário

**Termos que tu precisa saber:**

- **Long** 🟢 = Aposta que vai subir (compra)
- **Short** 🔴 = Aposta que vai cair (vende sem ter)
- **Alavancagem** = Multiplicador (10x = controla 10x mais)
- **Margin** = Garantia bloqueada pra manter posição
- **Liquidação** = Perda total quando preço vai muito contra ti
- **Stop Loss (SL)** = Sai no prejuízo automaticamente
- **Take Profit (TP)** = Sai no lucro automaticamente
- **PnL** = Profit and Loss (lucro ou prejuízo)
- **ROI** = Return on Investment (% de retorno)
- **Slippage** = Diferença entre preço esperado e executado
- **Fill** = Execução de ordem
- **Market Order** = Ordem que executa na hora
- **Limit Order** = Ordem que só executa num preço específico
- **Candle** = Vela do gráfico (OHLC = Open, High, Low, Close)
- **Timeframe** = Período de cada vela (1m, 1h, 1d, etc)
- **Indicator** = Indicador técnico (RSI, MACD, etc)
- **Signal** = Sinal de compra/venda
- **Strategy** = Estratégia de trading
- **Backtest** = Testar estratégia em dados históricos
- **Equity Curve** = Curva de evolução do capital
- **Drawdown** = Queda do pico até o fundo
- **Win Rate** = % de trades vencedoras
- **Profit Factor** = Lucro bruto / Prejuízo bruto
- **Paper Trading** = Simulação (sem dinheiro real)
- **Circuit Breaker** = Proteção que para tudo se perder demais

---

## 🚀 Vamos lá!

Agora tu já sabe TUDO sobre o RoboTrade!

**Próximos passos:**
1. Configura o `.env` com tuas API keys
2. Roda o app: `npm run tauri dev`
3. Começa no **Paper Trading**
4. Testa por 1-2 semanas
5. Se tiver lucro consistente, move pra Live (com pouco capital)
6. Escala conforme ganha confiança

**Lembre-se:**
- 🛡️ Risk management em PRIMEIRO lugar
- 📊 Sempre analisa os Reports
- 🎯 Define metas realistas (5-10% ao mês é MUITO bom)
- 🧠 Aprende com cada trade
- 😌 Não fica emocional (robô existe pra isso)

Boa sorte e bons trades! 🚀💰

---

**Dúvidas?** Abre uma issue no GitHub ou consulta a documentação técnica em `/docs`.
