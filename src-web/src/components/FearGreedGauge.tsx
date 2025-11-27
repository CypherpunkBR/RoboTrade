import { getFearGreedColor } from '@/lib/tauri';
import type { FearGreedClassification } from '@/types';

interface FearGreedGaugeProps {
  value: number | null;
  classification: FearGreedClassification | null;
  size?: 'sm' | 'md' | 'lg';
}

const classificationText: Record<FearGreedClassification, string> = {
  extreme_fear: 'Medo Extremo',
  fear: 'Medo',
  neutral: 'Neutro',
  greed: 'Ganância',
  extreme_greed: 'Ganância Extrema',
};

export function FearGreedGauge({ value, classification, size = 'md' }: FearGreedGaugeProps) {
  const sizeConfig = {
    sm: { width: 120, height: 80, strokeWidth: 8, fontSize: 20 },
    md: { width: 180, height: 110, strokeWidth: 12, fontSize: 28 },
    lg: { width: 240, height: 140, strokeWidth: 16, fontSize: 36 },
  }[size];

  if (value === null) {
    return (
      <div
        className="flex items-center justify-center text-muted-foreground"
        style={{ width: sizeConfig.width, height: sizeConfig.height }}
      >
        <span className="text-sm">Carregando...</span>
      </div>
    );
  }

  const color = getFearGreedColor(value);
  const radius = (sizeConfig.width - sizeConfig.strokeWidth) / 2;
  const circumference = Math.PI * radius;
  const progress = (value / 100) * circumference;

  return (
    <div className="flex flex-col items-center">
      <svg
        width={sizeConfig.width}
        height={sizeConfig.height}
        viewBox={`0 0 ${sizeConfig.width} ${sizeConfig.height}`}
      >
        {/* Background arc */}
        <path
          d={`M ${sizeConfig.strokeWidth / 2} ${sizeConfig.height - 10}
              A ${radius} ${radius} 0 0 1 ${sizeConfig.width - sizeConfig.strokeWidth / 2} ${sizeConfig.height - 10}`}
          fill="none"
          stroke="var(--color-secondary)"
          strokeWidth={sizeConfig.strokeWidth}
          strokeLinecap="round"
        />

        {/* Progress arc */}
        <path
          d={`M ${sizeConfig.strokeWidth / 2} ${sizeConfig.height - 10}
              A ${radius} ${radius} 0 0 1 ${sizeConfig.width - sizeConfig.strokeWidth / 2} ${sizeConfig.height - 10}`}
          fill="none"
          stroke={color}
          strokeWidth={sizeConfig.strokeWidth}
          strokeLinecap="round"
          strokeDasharray={circumference}
          strokeDashoffset={circumference - progress}
          style={{ transition: 'stroke-dashoffset 0.5s ease-in-out' }}
        />

        {/* Value text */}
        <text
          x={sizeConfig.width / 2}
          y={sizeConfig.height - 25}
          textAnchor="middle"
          fill="currentColor"
          fontSize={sizeConfig.fontSize}
          fontWeight="bold"
        >
          {value}
        </text>
      </svg>

      {classification && (
        <span
          className="text-sm font-medium mt-1"
          style={{ color }}
        >
          {classificationText[classification]}
        </span>
      )}
    </div>
  );
}
