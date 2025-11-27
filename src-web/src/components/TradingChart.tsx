/**
 * TradingChart - Componente principal de gráfico de trading
 *
 * Este componente renderiza um gráfico de candlestick interativo usando
 * lightweight-charts com suporte para:
 *
 * - Candlesticks com dados OHLCV
 * - Médias móveis (SMA e EMA) configuráveis
 * - Linhas de preço para ordens pendentes
 * - Linhas de preço para posições (entrada, SL, TP, liquidação)
 * - Seletor de timeframe
 * - Click para criar alertas de preço
 *
 * ## Uso Básico
 *
 * ```tsx
 * <TradingChart
 *   symbol="BTCUSDT"
 *   interval="1h"
 *   height={500}
 * />
 * ```
 *
 * ## Com Ordens e Posições
 *
 * ```tsx
 * <TradingChart
 *   symbol="BTCUSDT"
 *   orders={activeOrders}
 *   positions={openPositions}
 *   onPriceClick={(price) => createAlert(price)}
 * />
 * ```
 *
 * ## Linhas Exibidas Automaticamente
 *
 * | Tipo | Cor | Estilo | Descrição |
 * |------|-----|--------|-----------|
 * | Ordem Buy | Verde (#22c55e) | Tracejado | Ordem de compra pendente |
 * | Ordem Sell | Vermelho (#ef4444) | Tracejado | Ordem de venda pendente |
 * | Entry Long | Azul (#3b82f6) | Sólido | Preço de entrada posição long |
 * | Entry Short | Laranja (#f97316) | Sólido | Preço de entrada posição short |
 * | Stop Loss | Vermelho (#ef4444) | Tracejado | Preço de stop loss |
 * | Take Profit | Verde (#22c55e) | Tracejado | Preço de take profit |
 * | Liquidation | Vermelho escuro (#dc2626) | Pontilhado | Preço de liquidação |
 *
 * @see {@link ../lib/chart-helpers.ts} para utilitários adicionais
 *
 * @module TradingChart
 */

import { useEffect, useRef, useState, useCallback, useMemo } from 'react';
import {
  createChart,
  CrosshairMode,
  ColorType,
  CandlestickSeries,
  LineSeries,
} from 'lightweight-charts';
import type {
  IChartApi,
  ISeriesApi,
  CandlestickData,
  LineData,
  Time,
} from 'lightweight-charts';
import type { CandleDto, TimeFrame, OrderDto, PositionDto } from '@/types';
import { calculateIndicators } from '@/lib/tauri';
import { CHART_COLORS, LINE_STYLES } from '@/lib/chart-helpers';

// Debounce helper
function debounce<T extends (...args: Parameters<T>) => void>(
  fn: T,
  delay: number
): (...args: Parameters<T>) => void {
  let timeoutId: ReturnType<typeof setTimeout>;
  return (...args: Parameters<T>) => {
    clearTimeout(timeoutId);
    timeoutId = setTimeout(() => fn(...args), delay);
  };
}

interface MovingAverageConfig {
  enabled: boolean;
  period: number;
  color: string;
  type: 'sma' | 'ema';
}

interface ChartProps {
  symbol?: string;
  interval?: TimeFrame;
  height?: number;
  orders?: OrderDto[];
  positions?: PositionDto[];
  onPriceClick?: (price: number) => void;
}

const DEFAULT_MA_CONFIGS: MovingAverageConfig[] = [
  { enabled: true, period: 7, color: '#2196F3', type: 'sma' },
  { enabled: true, period: 25, color: '#FF9800', type: 'sma' },
  { enabled: false, period: 99, color: '#9C27B0', type: 'sma' },
  { enabled: true, period: 9, color: '#4CAF50', type: 'ema' },
  { enabled: false, period: 21, color: '#E91E63', type: 'ema' },
];

const TIMEFRAMES: { value: TimeFrame; label: string }[] = [
  { value: '1m', label: '1m' },
  { value: '5m', label: '5m' },
  { value: '15m', label: '15m' },
  { value: '30m', label: '30m' },
  { value: '1h', label: '1H' },
  { value: '4h', label: '4H' },
  { value: '1d', label: '1D' },
  { value: '1w', label: '1W' },
];

// Helper to safely convert value to number
const toNumber = (val: string | number | undefined | null): number => {
  if (val === undefined || val === null) return 0;
  if (typeof val === 'number') return isNaN(val) ? 0 : val;
  const num = parseFloat(String(val));
  return isNaN(num) ? 0 : num;
};

export function TradingChart({
  symbol = 'BTCUSDT',
  interval: initialInterval = '1h',
  height = 500,
  orders = [],
  positions = [],
  onPriceClick,
}: ChartProps) {
  const chartContainerRef = useRef<HTMLDivElement>(null);
  const chartRef = useRef<IChartApi | null>(null);
  const candleSeriesRef = useRef<ISeriesApi<'Candlestick'> | null>(null);
  const maSeriesRefs = useRef<Map<string, ISeriesApi<'Line'>>>(new Map());
  const priceLinesRef = useRef<Map<string, ReturnType<ISeriesApi<'Candlestick'>['createPriceLine']>>>(new Map());
  const isDisposedRef = useRef(false);
  const abortControllerRef = useRef<AbortController | null>(null);
  const onPriceClickRef = useRef(onPriceClick);

  const [selectedInterval, setSelectedInterval] = useState<TimeFrame>(initialInterval);
  const [maConfigs, setMaConfigs] = useState<MovingAverageConfig[]>(DEFAULT_MA_CONFIGS);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [currentPrice, setCurrentPrice] = useState<number | null>(null);
  const [showMAPanel, setShowMAPanel] = useState(false);

  // Keep onPriceClick ref updated without causing re-renders
  useEffect(() => {
    onPriceClickRef.current = onPriceClick;
  }, [onPriceClick]);

  // Memoize MA periods to prevent unnecessary fetches
  const enabledMaPeriods = useMemo(() => ({
    sma: maConfigs.filter(m => m.enabled && m.type === 'sma').map(m => m.period),
    ema: maConfigs.filter(m => m.enabled && m.type === 'ema').map(m => m.period),
  }), [maConfigs]);

  // Convert CandleDto to lightweight-charts format
  const convertCandles = useCallback((candles: CandleDto[]): CandlestickData<Time>[] => {
    if (!candles || !Array.isArray(candles)) {
      console.warn('convertCandles: Invalid candles data', candles);
      return [];
    }

    return candles
      .filter((c) => c && c.time !== undefined && c.time !== null)
      .map((c) => ({
        time: c.time as Time,
        open: toNumber(c.open),
        high: toNumber(c.high),
        low: toNumber(c.low),
        close: toNumber(c.close),
      }))
      .filter((c) => c.open > 0 && c.high > 0 && c.low > 0 && c.close > 0);
  }, []);

  // Initialize chart
  useEffect(() => {
    if (!chartContainerRef.current) return;

    // Reset disposed flag when creating new chart
    isDisposedRef.current = false;

    const chart = createChart(chartContainerRef.current, {
      layout: {
        background: { type: ColorType.Solid, color: 'transparent' },
        textColor: '#d1d5db',
      },
      grid: {
        vertLines: { color: 'rgba(42, 46, 57, 0.5)' },
        horzLines: { color: 'rgba(42, 46, 57, 0.5)' },
      },
      crosshair: {
        mode: CrosshairMode.Normal,
      },
      rightPriceScale: {
        borderColor: 'rgba(42, 46, 57, 0.8)',
      },
      timeScale: {
        borderColor: 'rgba(42, 46, 57, 0.8)',
        timeVisible: true,
        secondsVisible: false,
      },
      handleScroll: {
        vertTouchDrag: true,
      },
      handleScale: {
        axisPressedMouseMove: true,
        mouseWheel: true,
        pinch: true,
      },
    });

    // Add candlestick series using new v5 API
    const candleSeries = chart.addSeries(CandlestickSeries, {
      upColor: '#22c55e',
      downColor: '#ef4444',
      borderDownColor: '#ef4444',
      borderUpColor: '#22c55e',
      wickDownColor: '#ef4444',
      wickUpColor: '#22c55e',
    });

    chartRef.current = chart;
    candleSeriesRef.current = candleSeries;

    // Handle resize
    const handleResize = () => {
      if (!isDisposedRef.current && chartContainerRef.current && chartRef.current) {
        chartRef.current.applyOptions({
          width: chartContainerRef.current.clientWidth,
        });
      }
    };

    window.addEventListener('resize', handleResize);

    // Handle click on chart (for price alerts) - use ref to avoid re-creating chart
    chart.subscribeClick((param) => {
      if (param.point && onPriceClickRef.current && candleSeries) {
        const price = candleSeries.coordinateToPrice(param.point.y);
        if (price !== null) {
          onPriceClickRef.current(price);
        }
      }
    });

    // Track crosshair for current price display
    chart.subscribeCrosshairMove((param) => {
      if (param.seriesData && candleSeries) {
        const data = param.seriesData.get(candleSeries) as CandlestickData<Time> | undefined;
        if (data) {
          setCurrentPrice(data.close);
        }
      }
    });

    return () => {
      isDisposedRef.current = true;
      abortControllerRef.current?.abort();
      window.removeEventListener('resize', handleResize);
      priceLinesRef.current.clear();
      maSeriesRefs.current.clear();
      chart.remove();
      chartRef.current = null;
      candleSeriesRef.current = null;
    };
  }, []); // No dependencies - chart created once

  // Fetch and update data
  const fetchData = useCallback(async () => {
    // Skip if chart is disposed or not initialized
    if (isDisposedRef.current || !chartRef.current || !candleSeriesRef.current) return;

    // Cancel previous request
    abortControllerRef.current?.abort();
    abortControllerRef.current = new AbortController();
    const signal = abortControllerRef.current.signal;

    setLoading(true);
    setError(null);

    try {
      const data = await calculateIndicators(
        symbol,
        selectedInterval,
        500,
        enabledMaPeriods.sma.length > 0 ? enabledMaPeriods.sma : undefined,
        enabledMaPeriods.ema.length > 0 ? enabledMaPeriods.ema : undefined
      );

      // Check if request was aborted
      if (signal.aborted) return;

      // Check if chart was disposed during async operation
      if (isDisposedRef.current || !chartRef.current || !candleSeriesRef.current) {
        return;
      }

      // Update candles
      const candles = convertCandles(data.candles);
      if (candles.length === 0) {
        console.warn('No valid candle data received');
        setError('Sem dados de candles válidos');
        return;
      }
      candleSeriesRef.current.setData(candles);

      // Clear old MA series
      maSeriesRefs.current.forEach((series) => {
        chartRef.current?.removeSeries(series);
      });
      maSeriesRefs.current.clear();

      // Add SMA lines
      for (const config of maConfigs.filter((m) => m.enabled && m.type === 'sma')) {
        const smaData = data.sma?.[config.period.toString()];
        if (smaData && Array.isArray(smaData)) {
          const lineData: LineData<Time>[] = [];
          smaData.forEach((value, index) => {
            if (value !== null && value !== undefined && data.candles[index]) {
              const numValue = toNumber(value);
              if (numValue > 0) {
                lineData.push({
                  time: data.candles[index].time as Time,
                  value: numValue,
                });
              }
            }
          });

          if (lineData.length > 0) {
            const lineSeries = chartRef.current!.addSeries(LineSeries, {
              color: config.color,
              lineWidth: 1,
              priceLineVisible: false,
              lastValueVisible: false,
            });
            lineSeries.setData(lineData);
            maSeriesRefs.current.set(`sma_${config.period}`, lineSeries);
          }
        }
      }

      // Add EMA lines
      for (const config of maConfigs.filter((m) => m.enabled && m.type === 'ema')) {
        const emaData = data.ema?.[config.period.toString()];
        if (emaData && Array.isArray(emaData)) {
          const lineData: LineData<Time>[] = [];
          emaData.forEach((value, index) => {
            if (value !== null && value !== undefined && data.candles[index]) {
              const numValue = toNumber(value);
              if (numValue > 0) {
                lineData.push({
                  time: data.candles[index].time as Time,
                  value: numValue,
                });
              }
            }
          });

          if (lineData.length > 0) {
            const lineSeries = chartRef.current!.addSeries(LineSeries, {
              color: config.color,
              lineWidth: 1,
              priceLineVisible: false,
              lastValueVisible: false,
            });
            lineSeries.setData(lineData);
            maSeriesRefs.current.set(`ema_${config.period}`, lineSeries);
          }
        }
      }

      // Set current price from last candle
      if (candles.length > 0) {
        setCurrentPrice(candles[candles.length - 1].close);
      }

      // Fit content
      chartRef.current.timeScale().fitContent();
    } catch (err) {
      // Don't show error if request was aborted
      if (err instanceof Error && err.name === 'AbortError') return;
      console.error('Error fetching chart data:', err);
      setError(err instanceof Error ? err.message : 'Erro ao carregar dados');
    } finally {
      setLoading(false);
    }
  }, [symbol, selectedInterval, enabledMaPeriods, convertCandles]);

  // Debounced fetch for MA changes
  const debouncedFetch = useMemo(
    () => debounce(() => fetchData(), 300),
    [fetchData]
  );

  // Fetch data on mount and when symbol/interval change (immediate)
  useEffect(() => {
    fetchData();
  }, [symbol, selectedInterval]); // eslint-disable-line react-hooks/exhaustive-deps

  // Fetch data when MA periods change (debounced)
  useEffect(() => {
    debouncedFetch();
  }, [enabledMaPeriods]); // eslint-disable-line react-hooks/exhaustive-deps

  // Auto-refresh data
  useEffect(() => {
    const refreshInterval = setInterval(fetchData, 60000); // Refresh every minute
    return () => clearInterval(refreshInterval);
  }, [fetchData]);

  // Draw orders and positions as price lines
  useEffect(() => {
    if (isDisposedRef.current || !candleSeriesRef.current) return;

    // Clear existing price lines
    priceLinesRef.current.forEach((line) => {
      try {
        candleSeriesRef.current?.removePriceLine(line);
      } catch {
        // Ignore errors if line was already removed
      }
    });
    priceLinesRef.current.clear();

    const series = candleSeriesRef.current;

    // Filter orders for current symbol
    const symbolOrders = orders.filter((o) => o.symbol === symbol);
    const symbolPositions = positions.filter((p) => p.symbol === symbol);

    // Draw order price lines
    symbolOrders.forEach((order) => {
      const price = toNumber(order.price);
      if (price > 0) {
        const isBuy = order.side === 'buy';
        const line = series.createPriceLine({
          price,
          color: isBuy ? CHART_COLORS.ORDER_BUY : CHART_COLORS.ORDER_SELL,
          lineWidth: 1,
          lineStyle: LINE_STYLES.DASHED,
          axisLabelVisible: true,
          title: `${isBuy ? 'BUY' : 'SELL'} ${toNumber(order.quantity).toFixed(4)}`,
        });
        priceLinesRef.current.set(`order_${order.id}`, line);
      }
    });

    // Draw position entry prices
    symbolPositions.forEach((position) => {
      const entryPrice = toNumber(position.entry_price);
      if (entryPrice > 0) {
        const isLong = position.side === 'long';
        const line = series.createPriceLine({
          price: entryPrice,
          color: isLong ? CHART_COLORS.ENTRY_LONG : CHART_COLORS.ENTRY_SHORT,
          lineWidth: 2,
          lineStyle: LINE_STYLES.SOLID,
          axisLabelVisible: true,
          title: `${isLong ? 'LONG' : 'SHORT'} Entry`,
        });
        priceLinesRef.current.set(`pos_entry_${position.id}`, line);
      }

      // Draw stop loss if exists
      const stopLoss = toNumber(position.stop_loss_price);
      if (stopLoss > 0) {
        const line = series.createPriceLine({
          price: stopLoss,
          color: CHART_COLORS.STOP_LOSS,
          lineWidth: 1,
          lineStyle: LINE_STYLES.DASHED,
          axisLabelVisible: true,
          title: 'SL',
        });
        priceLinesRef.current.set(`pos_sl_${position.id}`, line);
      }

      // Draw take profit if exists
      const takeProfit = toNumber(position.take_profit_price);
      if (takeProfit > 0) {
        const line = series.createPriceLine({
          price: takeProfit,
          color: CHART_COLORS.TAKE_PROFIT,
          lineWidth: 1,
          lineStyle: LINE_STYLES.DASHED,
          axisLabelVisible: true,
          title: 'TP',
        });
        priceLinesRef.current.set(`pos_tp_${position.id}`, line);
      }

      // Draw liquidation price if exists
      const liquidationPrice = toNumber(position.liquidation_price);
      if (liquidationPrice > 0) {
        const line = series.createPriceLine({
          price: liquidationPrice,
          color: CHART_COLORS.LIQUIDATION,
          lineWidth: 2,
          lineStyle: LINE_STYLES.DOTTED,
          axisLabelVisible: true,
          title: 'LIQ',
        });
        priceLinesRef.current.set(`pos_liq_${position.id}`, line);
      }
    });
  }, [orders, positions, symbol]);

  // Update chart height
  useEffect(() => {
    if (!isDisposedRef.current && chartRef.current && chartContainerRef.current) {
      chartRef.current.applyOptions({
        height: height,
        width: chartContainerRef.current.clientWidth,
      });
    }
  }, [height]);

  const toggleMA = (index: number) => {
    setMaConfigs((prev) => {
      const newConfigs = [...prev];
      newConfigs[index] = { ...newConfigs[index], enabled: !newConfigs[index].enabled };
      return newConfigs;
    });
  };

  const updateMAPeriod = (index: number, period: number) => {
    setMaConfigs((prev) => {
      const newConfigs = [...prev];
      newConfigs[index] = { ...newConfigs[index], period };
      return newConfigs;
    });
  };

  return (
    <div className="relative">
      {/* Toolbar */}
      <div className="flex items-center justify-between mb-2 p-2 bg-card rounded-t-lg border border-border">
        <div className="flex items-center gap-2">
          <span className="font-semibold text-lg">{symbol}</span>
          {currentPrice !== null && (
            <span className="text-muted-foreground">
              ${currentPrice.toLocaleString('en-US', { minimumFractionDigits: 2 })}
            </span>
          )}
        </div>

        <div className="flex items-center gap-2">
          {/* Timeframe selector */}
          <div className="flex bg-muted rounded-md p-0.5">
            {TIMEFRAMES.map((tf) => (
              <button
                key={tf.value}
                onClick={() => setSelectedInterval(tf.value)}
                className={`px-2 py-1 text-xs font-medium rounded transition-colors ${
                  selectedInterval === tf.value
                    ? 'bg-primary text-white'
                    : 'text-muted-foreground hover:text-foreground'
                }`}
              >
                {tf.label}
              </button>
            ))}
          </div>

          {/* MA settings toggle */}
          <button
            onClick={() => setShowMAPanel(!showMAPanel)}
            className={`px-3 py-1 text-xs font-medium rounded transition-colors ${
              showMAPanel
                ? 'bg-primary text-white'
                : 'bg-muted text-muted-foreground hover:text-foreground'
            }`}
          >
            MA
          </button>

          {/* Refresh button */}
          <button
            onClick={fetchData}
            disabled={loading}
            className="px-3 py-1 text-xs font-medium bg-muted text-muted-foreground hover:text-foreground rounded transition-colors disabled:opacity-50"
          >
            {loading ? 'Carregando...' : 'Atualizar'}
          </button>
        </div>
      </div>

      {/* MA Configuration Panel */}
      {showMAPanel && (
        <div className="absolute top-14 right-2 z-10 p-3 bg-card border border-border rounded-lg shadow-lg min-w-[280px]">
          <h4 className="font-medium mb-2">Medias Moveis</h4>
          <div className="space-y-2">
            {maConfigs.map((config, index) => (
              <div key={index} className="flex items-center gap-2">
                <input
                  type="checkbox"
                  checked={config.enabled}
                  onChange={() => toggleMA(index)}
                  className="rounded"
                />
                <span
                  className="w-4 h-4 rounded"
                  style={{ backgroundColor: config.color }}
                />
                <span className="text-sm uppercase w-10">{config.type}</span>
                <input
                  type="number"
                  value={config.period}
                  onChange={(e) => updateMAPeriod(index, parseInt(e.target.value) || 1)}
                  className="w-16 px-2 py-1 text-sm bg-muted rounded border border-border"
                  min={1}
                  max={500}
                />
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Error message */}
      {error && (
        <div className="absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 z-10 p-4 bg-destructive/10 border border-destructive/20 rounded-md text-destructive">
          {error}
        </div>
      )}

      {/* Loading overlay */}
      {loading && (
        <div className="absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 z-10">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary" />
        </div>
      )}

      {/* Chart container */}
      <div
        ref={chartContainerRef}
        className="rounded-b-lg overflow-hidden border-x border-b border-border"
        style={{ height: `${height}px` }}
      />
    </div>
  );
}
