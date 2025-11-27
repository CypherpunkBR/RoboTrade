import { forwardRef, type HTMLAttributes } from 'react';

export interface BadgeProps extends HTMLAttributes<HTMLDivElement> {
  variant?: 'default' | 'secondary' | 'success' | 'warning' | 'destructive' | 'outline';
}

const variantClasses: Record<NonNullable<BadgeProps['variant']>, string> = {
  default: 'bg-primary text-primary-foreground hover:bg-primary/80',
  secondary: 'bg-secondary text-secondary-foreground hover:bg-secondary/80',
  success: 'bg-success text-white hover:bg-success/80',
  warning: 'bg-warning text-black hover:bg-warning/80',
  destructive: 'bg-destructive text-destructive-foreground hover:bg-destructive/80',
  outline: 'border border-input bg-background hover:bg-accent hover:text-accent-foreground',
};

export const Badge = forwardRef<HTMLDivElement, BadgeProps>(
  ({ className = '', variant = 'default', ...props }, ref) => (
    <div
      ref={ref}
      className={`inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 ${variantClasses[variant]} ${className}`}
      {...props}
    />
  )
);
Badge.displayName = 'Badge';
