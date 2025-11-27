import { useEffect, useRef, useState, useCallback } from 'react';
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
import type { CandleDto, TimeFrame } from '@/types';
import { calculateIndicators } from '@/lib/tauri';

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
  onPriceClick,
}: ChartProps) {
  const chartContainerRef = useRef<HTMLDivElement>(null);
  const chartRef = useRef<IChartApi | null>(null);
  const candleSeriesRef = useRef<ISeriesApi<'Candlestick'> | null>(null);
  const maSeriesRefs = useRef<Map<string, ISeriesApi<'Line'>>>(new Map());

  const [selectedInterval, setSelectedInterval] = useState<TimeFrame>(initialInterval);
  const [maConfigs, setMaConfigs] = useState<MovingAverageConfig[]>(DEFAULT_MA_CONFIGS);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [currentPrice, setCurrentPrice] = useState<number | null>(null);
  const [showMAPanel, setShowMAPanel] = useState(false);

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
      if (chartContainerRef.current && chartRef.current) {
        chartRef.current.applyOptions({
          width: chartContainerRef.current.clientWidth,
        });
      }
    };

    window.addEventListener('resize', handleResize);

    // Handle click on chart (for price alerts)
    chart.subscribeClick((param) => {
      if (param.point && onPriceClick && candleSeries) {
        const price = candleSeries.coordinateToPrice(param.point.y);
        if (price !== null) {
          onPriceClick(price);
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
      window.removeEventListener('resize', handleResize);
      chart.remove();
    };
  }, [onPriceClick]);

  // Fetch and update data
  const fetchData = useCallback(async () => {
    if (!chartRef.current || !candleSeriesRef.current) return;

    setLoading(true);
    setError(null);

    try {
      const enabledSmas = maConfigs
        .filter((m) => m.enabled && m.type === 'sma')
        .map((m) => m.period);
      const enabledEmas = maConfigs
        .filter((m) => m.enabled && m.type === 'ema')
        .map((m) => m.period);

      const data = await calculateIndicators(
        symbol,
        selectedInterval,
        500,
        enabledSmas.length > 0 ? enabledSmas : undefined,
        enabledEmas.length > 0 ? enabledEmas : undefined
      );

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
      console.error('Error fetching chart data:', err);
      setError(err instanceof Error ? err.message : 'Erro ao carregar dados');
    } finally {
      setLoading(false);
    }
  }, [symbol, selectedInterval, maConfigs, convertCandles]);

  // Fetch data on mount and when dependencies change
  useEffect(() => {
    fetchData();
  }, [fetchData]);

  // Auto-refresh data
  useEffect(() => {
    const refreshInterval = setInterval(fetchData, 60000); // Refresh every minute
    return () => clearInterval(refreshInterval);
  }, [fetchData]);

  // Update chart height
  useEffect(() => {
    if (chartRef.current && chartContainerRef.current) {
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
