import { forwardRef, type SelectHTMLAttributes } from 'react';

export interface SelectProps extends SelectHTMLAttributes<HTMLSelectElement> {
  variant?: 'default' | 'outline';
}

const variantClasses: Record<NonNullable<SelectProps['variant']>, string> = {
  default: 'bg-background border-border',
  outline: 'bg-transparent border-input',
};

export const Select = forwardRef<HTMLSelectElement, SelectProps>(
  ({ className = '', variant = 'default', disabled, children, ...props }, ref) => (
    <select
      ref={ref}
      className={`flex h-10 w-full rounded-md border px-3 py-2 text-sm ring-offset-background focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 ${variantClasses[variant]} ${className}`}
      disabled={disabled}
      {...props}
    >
      {children}
    </select>
  )
);
Select.displayName = 'Select';

export interface SelectOptionProps {
  value: string;
  children: React.ReactNode;
  disabled?: boolean;
}

export function SelectOption({ value, children, disabled }: SelectOptionProps) {
  return (
    <option value={value} disabled={disabled}>
      {children}
    </option>
  );
}
