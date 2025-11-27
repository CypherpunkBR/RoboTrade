import { useState, useRef, useEffect, type ReactNode } from 'react';

interface PopoverProps {
  trigger: ReactNode;
  children: ReactNode;
  align?: 'start' | 'center' | 'end';
  side?: 'top' | 'bottom' | 'left' | 'right';
  className?: string;
}

export function Popover({
  trigger,
  children,
  align = 'center',
  side = 'bottom',
  className = '',
}: PopoverProps) {
  const [isOpen, setIsOpen] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const handleMouseEnter = () => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
    }
    setIsOpen(true);
  };

  const handleMouseLeave = () => {
    timeoutRef.current = setTimeout(() => {
      setIsOpen(false);
    }, 150);
  };

  useEffect(() => {
    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
    };
  }, []);

  const alignmentClasses = {
    start: 'left-0',
    center: 'left-1/2 -translate-x-1/2',
    end: 'right-0',
  };

  const sideClasses = {
    top: 'bottom-full mb-2',
    bottom: 'top-full mt-2',
    left: 'right-full mr-2',
    right: 'left-full ml-2',
  };

  return (
    <div
      ref={containerRef}
      className="relative inline-block"
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
    >
      {trigger}
      {isOpen && (
        <div
          className={`absolute z-50 min-w-[200px] rounded-md border border-border bg-card p-3 shadow-lg animate-in fade-in-0 zoom-in-95 ${sideClasses[side]} ${alignmentClasses[align]} ${className}`}
        >
          {children}
        </div>
      )}
    </div>
  );
}

interface HoverCardProps {
  trigger: ReactNode;
  children: ReactNode;
  className?: string;
}

export function HoverCard({ trigger, children, className = '' }: HoverCardProps) {
  return (
    <Popover trigger={trigger} side="bottom" align="start" className={className}>
      {children}
    </Popover>
  );
}
