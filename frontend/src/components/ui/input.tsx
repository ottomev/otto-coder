import * as React from 'react';
import { cn } from '@/lib/utils';

export interface InputProps
  extends React.InputHTMLAttributes<HTMLInputElement> {
  onCommandEnter?: (e: React.KeyboardEvent<HTMLInputElement>) => void;
  onCommandShiftEnter?: (e: React.KeyboardEvent<HTMLInputElement>) => void;
}

const Input = React.forwardRef<HTMLInputElement, InputProps>(
  (
    {
      className,
      type,
      onKeyDown,
      onCommandEnter,
      onCommandShiftEnter,
      ...props
    },
    ref
  ) => {
    const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
      if (e.key === 'Escape') {
        e.currentTarget.blur();
      }
      if (e.key === 'Enter') {
        if (e.metaKey && e.shiftKey) {
          onCommandShiftEnter?.(e);
        } else {
          onCommandEnter?.(e);
        }
      }
      onKeyDown?.(e);
    };

    return (
      <input
        ref={ref}
        type={type}
        onKeyDown={handleKeyDown}
        className={cn(
          'flex h-10 w-full rounded-md glass-input px-3 py-2 text-sm file:border-0 file:text-sm file:font-medium focus-visible:outline-none focus-visible:ring-0 disabled:cursor-not-allowed disabled:opacity-50',
          className
        )}
        {...props}
      />
    );
  }
);

Input.displayName = 'Input';
export { Input };
