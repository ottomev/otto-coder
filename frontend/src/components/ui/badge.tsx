import * as React from 'react';
import { cva, type VariantProps } from 'class-variance-authority';

import { cn } from '@/lib/utils';

const badgeVariants = cva(
  'inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium transition-all focus:outline-none focus:ring-0',
  {
    variants: {
      variant: {
        default:
          'glass-surface text-primary border-primary/30',
        secondary:
          'glass-surface text-secondary-foreground border-white/20',
        destructive:
          'glass-surface text-destructive border-destructive/30',
        success:
          'glass-surface text-success border-success/30',
        outline: 'glass-input text-foreground',
        accent:
          'glass-surface text-accent border-accent/30',
      },
    },
    defaultVariants: {
      variant: 'default',
    },
  }
);

export interface BadgeProps
  extends React.HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof badgeVariants> {}

function Badge({ className, variant, ...props }: BadgeProps) {
  return (
    <div className={cn(badgeVariants({ variant }), className)} {...props} />
  );
}

export { Badge, badgeVariants };
