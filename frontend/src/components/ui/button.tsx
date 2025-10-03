import * as React from 'react';
import { Slot } from '@radix-ui/react-slot';
import { cva, type VariantProps } from 'class-variance-authority';

import { cn } from '@/lib/utils';

const buttonVariants = cva(
  'inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium transition-all focus-visible:outline-none focus-visible:ring-0 focus-visible:shadow-[0_0_20px_rgba(91,156,242,0.4)] disabled:pointer-events-none disabled:opacity-50',
  {
    variants: {
      variant: {
        default:
          'glass-button text-white hover:brightness-110 active:scale-95 shadow-lg',
        destructive:
          'glass-button text-destructive-foreground hover:brightness-110 active:scale-95 shadow-lg border-destructive/30',
        outline:
          'glass-input hover:brightness-110 active:scale-95',
        secondary: 'glass-surface text-secondary-foreground hover:brightness-110 active:scale-95 shadow-md',
        ghost: 'hover:glass-surface active:scale-95',
        link: 'text-primary underline-offset-4 hover:underline hover:brightness-125',
      },
      size: {
        default: 'h-9 px-4 py-2',
        xs: 'h-7 px-2 text-xs',
        sm: 'h-8 px-3 text-sm',
        lg: 'h-10 px-6',
        icon: 'h-9 w-9',
      },
    },
    defaultVariants: {
      variant: 'default',
      size: 'default',
    },
  }
);

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {
  asChild?: boolean;
}

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, size, asChild = false, ...props }, ref) => {
    const Comp = asChild ? Slot : 'button';
    return (
      <Comp
        className={cn(buttonVariants({ variant, size, className }))}
        ref={ref}
        {...props}
      />
    );
  }
);
Button.displayName = 'Button';

export { Button, buttonVariants };
