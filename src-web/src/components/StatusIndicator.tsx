import type { ServiceStatus } from '@/types';

interface StatusIndicatorProps {
  status: ServiceStatus;
  label?: string;
}

export function StatusIndicator({ status, label }: StatusIndicatorProps) {
  const statusClass = {
    connected: 'connected',
    disconnected: 'disconnected',
    connecting: 'connecting',
    error: 'disconnected',
  }[status];

  const statusText = {
    connected: 'Conectado',
    disconnected: 'Desconectado',
    connecting: 'Conectando...',
    error: 'Erro',
  }[status];

  return (
    <div className="flex items-center gap-2">
      <div className={`status-dot ${statusClass}`} />
      {label && <span className="text-sm text-muted-foreground">{label}:</span>}
      <span className="text-sm">{statusText}</span>
    </div>
  );
}
