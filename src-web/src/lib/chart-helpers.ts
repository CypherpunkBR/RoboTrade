/**
 * Chart Helpers - Utilitários para desenhar elementos no gráfico de trading
 *
 * Este módulo contém funções e configurações para desenhar linhas de preço,
 * marcadores e outros elementos visuais no gráfico usando lightweight-charts.
 *
 * ## Uso Futuro
 *
 * Estas funções estão preparadas para serem integradas no TradingChart.tsx
 * quando houver necessidade de funcionalidades mais avançadas.
 *
 * @module chart-helpers
 */

import type { CreatePriceLineOptions, SeriesMarker, Time } from 'lightweight-charts';

// ============================================================================
// TIPOS E INTERFACES
// ============================================================================

/**
 * Tipos de linha de preço que podem ser desenhadas no gráfico
 */
export type PriceLineType =
  | 'order_buy'      // Ordem de compra pendente
  | 'order_sell'     // Ordem de venda pendente
  | 'entry_long'     // Preço de entrada (posição long)
  | 'entry_short'    // Preço de entrada (posição short)
  | 'stop_loss'      // Stop loss
  | 'take_profit'    // Take profit
  | 'liquidation'    // Preço de liquidação
  | 'alert'          // Alerta de preço
  | 'custom';        // Linha customizada

/**
 * Configuração para criar uma linha de preço
 */
export interface PriceLineConfig {
  type: PriceLineType;
  price: number;
  label?: string;
  quantity?: number;
}

/**
 * Configuração para marcador no gráfico
 */
export interface ChartMarkerConfig {
  time: Time;
  position: 'aboveBar' | 'belowBar' | 'inBar';
  color: string;
  shape: 'circle' | 'square' | 'arrowUp' | 'arrowDown';
  text?: string;
  size?: number;
}

// ============================================================================
// CONFIGURAÇÕES DE ESTILO
// ============================================================================

/**
 * Estilos de linha do lightweight-charts
 * 0 = Solid, 1 = Dotted, 2 = Dashed, 3 = Large Dashed, 4 = Sparse Dotted
 */
export const LINE_STYLES = {
  SOLID: 0,
  DOTTED: 1,
  DASHED: 2,
  LARGE_DASHED: 3,
  SPARSE_DOTTED: 4,
} as const;

/**
 * Paleta de cores para elementos do gráfico
 */
export const CHART_COLORS = {
  // Ordens
  ORDER_BUY: '#22c55e',      // Verde - compra
  ORDER_SELL: '#ef4444',     // Vermelho - venda

  // Posições
  ENTRY_LONG: '#3b82f6',     // Azul - entrada long
  ENTRY_SHORT: '#f97316',    // Laranja - entrada short

  // Risk Management
  STOP_LOSS: '#ef4444',      // Vermelho
  TAKE_PROFIT: '#22c55e',    // Verde
  LIQUIDATION: '#dc2626',    // Vermelho escuro (perigo)

  // Alertas
  ALERT_ACTIVE: '#eab308',   // Amarelo
  ALERT_TRIGGERED: '#8b5cf6', // Roxo

  // Indicadores
  SMA_DEFAULT: '#2196F3',
  EMA_DEFAULT: '#FF9800',
  BOLLINGER_UPPER: '#9C27B0',
  BOLLINGER_LOWER: '#9C27B0',
  BOLLINGER_MIDDLE: '#E91E63',
} as const;

/**
 * Configurações padrão para cada tipo de linha de preço
 */
export const PRICE_LINE_DEFAULTS: Record<PriceLineType, Partial<CreatePriceLineOptions>> = {
  order_buy: {
    color: CHART_COLORS.ORDER_BUY,
    lineWidth: 1,
    lineStyle: LINE_STYLES.DASHED,
    axisLabelVisible: true,
  },
  order_sell: {
    color: CHART_COLORS.ORDER_SELL,
    lineWidth: 1,
    lineStyle: LINE_STYLES.DASHED,
    axisLabelVisible: true,
  },
  entry_long: {
    color: CHART_COLORS.ENTRY_LONG,
    lineWidth: 2,
    lineStyle: LINE_STYLES.SOLID,
    axisLabelVisible: true,
  },
  entry_short: {
    color: CHART_COLORS.ENTRY_SHORT,
    lineWidth: 2,
    lineStyle: LINE_STYLES.SOLID,
    axisLabelVisible: true,
  },
  stop_loss: {
    color: CHART_COLORS.STOP_LOSS,
    lineWidth: 1,
    lineStyle: LINE_STYLES.DASHED,
    axisLabelVisible: true,
  },
  take_profit: {
    color: CHART_COLORS.TAKE_PROFIT,
    lineWidth: 1,
    lineStyle: LINE_STYLES.DASHED,
    axisLabelVisible: true,
  },
  liquidation: {
    color: CHART_COLORS.LIQUIDATION,
    lineWidth: 2,
    lineStyle: LINE_STYLES.DOTTED,
    axisLabelVisible: true,
  },
  alert: {
    color: CHART_COLORS.ALERT_ACTIVE,
    lineWidth: 1,
    lineStyle: LINE_STYLES.SPARSE_DOTTED,
    axisLabelVisible: true,
  },
  custom: {
    color: '#888888',
    lineWidth: 1,
    lineStyle: LINE_STYLES.SOLID,
    axisLabelVisible: true,
  },
};

// ============================================================================
// FUNÇÕES DE CRIAÇÃO DE LINHAS
// ============================================================================

/**
 * Cria as opções para uma linha de preço baseado no tipo
 *
 * @param config - Configuração da linha de preço
 * @returns Opções formatadas para createPriceLine()
 *
 * @example
 * ```typescript
 * const options = createPriceLineOptions({
 *   type: 'stop_loss',
 *   price: 42000,
 *   label: 'SL -2%',
 * });
 * candleSeries.createPriceLine(options);
 * ```
 */
export function createPriceLineOptions(config: PriceLineConfig): CreatePriceLineOptions {
  const defaults = PRICE_LINE_DEFAULTS[config.type];
  const title = config.label ?? getDefaultLabel(config);

  return {
    ...defaults,
    price: config.price,
    title,
  };
}

/**
 * Gera um label padrão para a linha de preço
 */
function getDefaultLabel(config: PriceLineConfig): string {
  const qty = config.quantity ? ` ${config.quantity.toFixed(4)}` : '';

  switch (config.type) {
    case 'order_buy':
      return `BUY${qty}`;
    case 'order_sell':
      return `SELL${qty}`;
    case 'entry_long':
      return `LONG Entry`;
    case 'entry_short':
      return `SHORT Entry`;
    case 'stop_loss':
      return 'SL';
    case 'take_profit':
      return 'TP';
    case 'liquidation':
      return 'LIQ';
    case 'alert':
      return 'Alert';
    default:
      return '';
  }
}

// ============================================================================
// FUNÇÕES DE MARCADORES
// ============================================================================

/**
 * Cria um marcador para indicar execução de trade
 *
 * @param time - Timestamp do trade
 * @param side - 'buy' ou 'sell'
 * @param price - Preço de execução (para exibir no texto)
 * @returns Configuração do marcador
 *
 * @example
 * ```typescript
 * const markers = [
 *   createTradeMarker(1699999999, 'buy', 42000),
 *   createTradeMarker(1700000999, 'sell', 43500),
 * ];
 * candleSeries.setMarkers(markers);
 * ```
 */
export function createTradeMarker(
  time: Time,
  side: 'buy' | 'sell',
  price?: number
): SeriesMarker<Time> {
  const isBuy = side === 'buy';
  return {
    time,
    position: isBuy ? 'belowBar' : 'aboveBar',
    color: isBuy ? CHART_COLORS.ORDER_BUY : CHART_COLORS.ORDER_SELL,
    shape: isBuy ? 'arrowUp' : 'arrowDown',
    text: price ? `${isBuy ? 'B' : 'S'} $${price.toLocaleString()}` : (isBuy ? 'B' : 'S'),
    size: 2,
  };
}

/**
 * Cria um marcador para alerta disparado
 *
 * @param time - Timestamp do alerta
 * @param text - Texto do alerta
 * @returns Configuração do marcador
 */
export function createAlertMarker(time: Time, text?: string): SeriesMarker<Time> {
  return {
    time,
    position: 'aboveBar',
    color: CHART_COLORS.ALERT_TRIGGERED,
    shape: 'circle',
    text: text ?? 'Alert',
    size: 1,
  };
}

// ============================================================================
// FUNÇÕES UTILITÁRIAS
// ============================================================================

/**
 * Calcula o preço de liquidação estimado para uma posição
 *
 * NOTA: Esta é uma estimativa simplificada. O preço real de liquidação
 * depende de vários fatores da exchange (maintenance margin, funding, etc.)
 *
 * @param entryPrice - Preço de entrada
 * @param leverage - Alavancagem usada
 * @param isLong - Se é posição long
 * @param maintenanceMarginRate - Taxa de margem de manutenção (padrão 0.5%)
 * @returns Preço de liquidação estimado
 *
 * @example
 * ```typescript
 * // Long 10x entry at 42000
 * const liqPrice = estimateLiquidationPrice(42000, 10, true);
 * // ~= 38220 (aproximadamente -9% do entry)
 * ```
 */
export function estimateLiquidationPrice(
  entryPrice: number,
  leverage: number,
  isLong: boolean,
  maintenanceMarginRate: number = 0.005
): number {
  // Fórmula simplificada:
  // Long: liqPrice = entryPrice * (1 - 1/leverage + maintenanceMarginRate)
  // Short: liqPrice = entryPrice * (1 + 1/leverage - maintenanceMarginRate)

  if (isLong) {
    return entryPrice * (1 - 1 / leverage + maintenanceMarginRate);
  } else {
    return entryPrice * (1 + 1 / leverage - maintenanceMarginRate);
  }
}

/**
 * Calcula a distância percentual até o preço de liquidação
 *
 * @param currentPrice - Preço atual
 * @param liquidationPrice - Preço de liquidação
 * @returns Percentual de distância (negativo = próximo da liquidação)
 */
export function distanceToLiquidation(
  currentPrice: number,
  liquidationPrice: number
): number {
  return ((currentPrice - liquidationPrice) / currentPrice) * 100;
}

/**
 * Formata preço para exibição no gráfico
 *
 * @param price - Preço a formatar
 * @param decimals - Casas decimais (auto-detecta se não especificado)
 * @returns String formatada
 */
export function formatChartPrice(price: number, decimals?: number): string {
  if (decimals !== undefined) {
    return price.toFixed(decimals);
  }

  // Auto-detecta decimais baseado no valor
  if (price >= 1000) return price.toFixed(2);
  if (price >= 1) return price.toFixed(4);
  if (price >= 0.001) return price.toFixed(6);
  return price.toFixed(8);
}

// ============================================================================
// TIPOS EXPORTADOS PARA USO FUTURO
// ============================================================================

/**
 * Interface para gerenciamento de linhas no gráfico
 *
 * @future Implementar classe ChartLinesManager que gerencia
 * todas as linhas de preço de forma centralizada
 */
export interface ChartLinesManager {
  addOrderLine(orderId: string, price: number, side: 'buy' | 'sell', quantity: number): void;
  addPositionLines(positionId: string, entry: number, sl?: number, tp?: number, liq?: number): void;
  addAlertLine(alertId: string, price: number, condition: string): void;
  removeLine(lineId: string): void;
  removeAllLines(): void;
  updateLine(lineId: string, price: number): void;
}

/**
 * Interface para gerenciamento de marcadores no gráfico
 *
 * @future Implementar classe ChartMarkersManager que gerencia
 * todos os marcadores de trades e eventos
 */
export interface ChartMarkersManager {
  addTradeMarker(time: Time, side: 'buy' | 'sell', price: number): void;
  addAlertMarker(time: Time, text: string): void;
  addEventMarker(time: Time, event: string, color: string): void;
  clearMarkers(): void;
  setMarkers(markers: SeriesMarker<Time>[]): void;
}

/**
 * Interface para desenho de regiões/zonas no gráfico
 *
 * @future Implementar suporte para desenhar zonas de suporte/resistência,
 * zonas de entrada, e áreas de risco
 */
export interface ChartZone {
  id: string;
  priceTop: number;
  priceBottom: number;
  color: string;
  opacity: number;
  label?: string;
}

// ============================================================================
// DOCUMENTAÇÃO DE IMPLEMENTAÇÃO FUTURA
// ============================================================================

/**
 * ## Roadmap de Funcionalidades do Gráfico
 *
 * ### Fase 1 - Básico (Implementado)
 * - [x] Linhas de preço para ordens pendentes
 * - [x] Linhas de preço para posições (entry, SL, TP)
 * - [x] Linha de preço de liquidação
 * - [x] Cores diferenciadas por tipo
 *
 * ### Fase 2 - Interatividade
 * - [ ] Drag & drop para mover linhas de SL/TP
 * - [ ] Click para criar alertas de preço
 * - [ ] Hover tooltip com detalhes da ordem/posição
 * - [ ] Right-click menu contextual
 *
 * ### Fase 3 - Marcadores de Eventos
 * - [ ] Marcadores de execução de trades
 * - [ ] Marcadores de alertas disparados
 * - [ ] Marcadores de eventos de funding
 * - [ ] Histórico visual de trades no gráfico
 *
 * ### Fase 4 - Zonas e Desenho
 * - [ ] Zonas de suporte/resistência
 * - [ ] Ferramentas de desenho (linhas, fibonnaci)
 * - [ ] Salvar/carregar desenhos
 *
 * ### Fase 5 - Analytics
 * - [ ] Heatmap de volume por preço
 * - [ ] Indicador de liquidações em massa
 * - [ ] Open interest overlay
 */
