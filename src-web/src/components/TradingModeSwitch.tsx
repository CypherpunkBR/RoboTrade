import type { TradingMode } from '@/types';

interface TradingModeSwitchProps {
  mode: TradingMode;
  onModeChange: (mode: TradingMode) => void;
  disabled?: boolean;
}

export function TradingModeSwitch({ mode, onModeChange, disabled }: TradingModeSwitchProps) {
  return (
    <div className="flex items-center gap-3">
      <span className="text-sm text-muted-foreground">Modo:</span>
      <div className="relative inline-flex items-center rounded-full bg-secondary p-1">
        <button
          onClick={() => onModeChange('paper')}
          disabled={disabled}
          className={`relative z-10 px-3 py-1 text-sm font-medium rounded-full transition-colors ${
            mode === 'paper'
              ? 'text-white'
              : 'text-muted-foreground hover:text-foreground'
          }`}
        >
          Paper
        </button>
        <button
          onClick={() => onModeChange('live')}
          disabled={disabled}
          className={`relative z-10 px-3 py-1 text-sm font-medium rounded-full transition-colors ${
            mode === 'live'
              ? 'text-white'
              : 'text-muted-foreground hover:text-foreground'
          }`}
        >
          Live
        </button>
        <div
          className={`absolute top-1 h-[calc(100%-8px)] rounded-full transition-all duration-200 ${
            mode === 'paper'
              ? 'left-1 w-[52px] bg-warning'
              : 'left-[60px] w-[42px] bg-destructive'
          }`}
        />
      </div>
      {mode === 'live' && (
        <span className="text-xs text-destructive font-medium animate-pulse">
          ⚠ LIVE
        </span>
      )}
    </div>
  );
}
