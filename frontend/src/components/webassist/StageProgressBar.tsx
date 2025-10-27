import { cn } from '@/lib/utils';

const STAGES = [
  { key: 'initial_review', label: 'Initial Review' },
  { key: 'ai_research', label: 'AI Research' },
  { key: 'design_mockup', label: 'Design' },
  { key: 'content_collection', label: 'Content' },
  { key: 'development', label: 'Development' },
  { key: 'quality_assurance', label: 'QA' },
  { key: 'client_preview', label: 'Preview' },
  { key: 'deployment', label: 'Deploy' },
  { key: 'delivered', label: 'Delivered' },
];

interface StageProgressBarProps {
  currentStage: string;
  className?: string;
}

export function StageProgressBar({ currentStage, className }: StageProgressBarProps) {
  const currentIndex = STAGES.findIndex((s) => s.key === currentStage);

  return (
    <div className={cn('flex items-center gap-1', className)}>
      {STAGES.map((stage, index) => {
        const isCompleted = index < currentIndex;
        const isCurrent = index === currentIndex;

        return (
          <div
            key={stage.key}
            className={cn(
              'flex-1 h-2 rounded-full transition-all',
              isCompleted && 'bg-green-500',
              isCurrent && 'bg-blue-500',
              !isCompleted && !isCurrent && 'bg-gray-200 dark:bg-gray-700'
            )}
            title={stage.label}
          />
        );
      })}
    </div>
  );
}
