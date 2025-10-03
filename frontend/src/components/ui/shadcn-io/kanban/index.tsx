'use client';

import { Card } from '@/components/ui/card';
import { cn } from '@/lib/utils';
import type { DragEndEvent, Modifier } from '@dnd-kit/core';
import {
  DndContext,
  PointerSensor,
  rectIntersection,
  useDraggable,
  useDroppable,
  useSensor,
  useSensors,
} from '@dnd-kit/core';
import { type ReactNode, type Ref, type KeyboardEvent } from 'react';

import type { ClientRect } from '@dnd-kit/core';
import type { Transform } from '@dnd-kit/utilities';
export type { DragEndEvent } from '@dnd-kit/core';

export type Status = {
  id: string;
  name: string;
  color: string;
};

export type Feature = {
  id: string;
  name: string;
  startAt: Date;
  endAt: Date;
  status: Status;
};

export type KanbanBoardProps = {
  id: Status['id'];
  children: ReactNode;
  className?: string;
};

export const KanbanBoard = ({ id, children, className }: KanbanBoardProps) => {
  const { isOver, setNodeRef } = useDroppable({ id });

  return (
    <div
      className={cn(
        'flex h-full min-h-40 flex-col',
        isOver ? 'outline-primary' : 'outline-black',
        className
      )}
      ref={setNodeRef}
    >
      {children}
    </div>
  );
};

export type KanbanCardProps = Pick<Feature, 'id' | 'name'> & {
  index: number;
  parent: string;
  children?: ReactNode;
  className?: string;
  onClick?: () => void;
  tabIndex?: number;
  forwardedRef?: Ref<HTMLDivElement>;
  onKeyDown?: (e: KeyboardEvent) => void;
  isOpen?: boolean;
};

export const KanbanCard = ({
  id,
  name,
  index,
  parent,
  children,
  className,
  onClick,
  tabIndex,
  forwardedRef,
  onKeyDown,
  isOpen,
}: KanbanCardProps) => {
  const { attributes, listeners, setNodeRef, transform, isDragging } =
    useDraggable({
      id,
      data: { index, parent },
    });

  // Combine DnD ref and forwarded ref
  const combinedRef = (node: HTMLDivElement | null) => {
    setNodeRef(node);
    if (typeof forwardedRef === 'function') {
      forwardedRef(node);
    } else if (forwardedRef && typeof forwardedRef === 'object') {
      (forwardedRef as React.MutableRefObject<HTMLDivElement | null>).current =
        node;
    }
  };

  return (
    <Card
      className={cn(
        'p-3 outline-none border-b border-white/10 flex-col space-y-2',
        isDragging && 'cursor-grabbing',
        isOpen && 'ring-2 ring-primary ring-inset',
        className
      )}
      {...listeners}
      {...attributes}
      ref={combinedRef}
      tabIndex={tabIndex}
      onClick={onClick}
      onKeyDown={onKeyDown}
      style={{
        zIndex: isDragging ? 1000 : 1,
        transform: transform
          ? `translateX(${transform.x}px) translateY(${transform.y}px)`
          : 'none',
      }}
    >
      {children ?? <p className="m-0 font-medium text-sm">{name}</p>}
    </Card>
  );
};

export type KanbanCardsProps = {
  children: ReactNode;
  className?: string;
};

export const KanbanCards = ({ children, className }: KanbanCardsProps) => (
  <div className={cn('flex flex-1 flex-col pt-2', className)}>{children}</div>
);

export type KanbanHeaderProps =
  | {
      children: ReactNode;
    }
  | {
      name: Status['name'];
      color: Status['color'];
      className?: string;
    };

export const KanbanHeader = (props: KanbanHeaderProps) =>
  'children' in props ? (
    props.children
  ) : (
    <Card
      className={cn(
        'sticky top-0 z-20 flex shrink-0 items-center gap-2 p-3 border-b border-white/10',
        'glass-surface',
        props.className
      )}
      style={{
        backgroundImage: `linear-gradient(hsl(var(${props.color}) / 0.05), hsl(var(${props.color}) / 0.05))`,
      }}
    >
      <div
        className="h-2 w-2 rounded-full"
        style={{ backgroundColor: `hsl(var(${props.color}))` }}
      />
      <p className="m-0 text-sm font-medium">{props.name}</p>
    </Card>
  );

function restrictToBoundingRectWithRightPadding(
  transform: Transform,
  rect: ClientRect,
  boundingRect: ClientRect,
  rightPadding: number
): Transform {
  console.log(rect, boundingRect);
  const value = {
    ...transform,
  };

  if (rect.top + transform.y <= boundingRect.top) {
    value.y = boundingRect.top - rect.top;
  } else if (
    rect.bottom + transform.y >=
    boundingRect.top + boundingRect.height
  ) {
    value.y = boundingRect.top + boundingRect.height - rect.bottom;
  }

  if (rect.left + transform.x <= boundingRect.left) {
    value.x = boundingRect.left - rect.left;
  } else if (
    // branch that checks if the right edge of the dragged element is beyond
    // the right edge of the bounding rectangle
    rect.right + transform.x + rightPadding >=
    boundingRect.left + boundingRect.width
  ) {
    value.x =
      boundingRect.left + boundingRect.width - rect.right - rightPadding;
  }

  return {
    ...value,
    x: value.x,
  };
}

// An alternative to `restrictToFirstScrollableAncestor` from the dnd-kit library
const restrictToFirstScrollableAncestorCustom: Modifier = (args) => {
  const { draggingNodeRect, transform, scrollableAncestorRects } = args;
  const firstScrollableAncestorRect = scrollableAncestorRects[0];

  if (!draggingNodeRect || !firstScrollableAncestorRect) {
    return transform;
  }

  // Inset the right edge that the rect can be dragged to by this amount.
  // This is a workaround for the kanban board where dragging a card too far
  // to the right causes infinite horizontal scrolling if there are also
  // enough cards for vertical scrolling to be enabled.
  const rightPadding = 16;
  return restrictToBoundingRectWithRightPadding(
    transform,
    draggingNodeRect,
    firstScrollableAncestorRect,
    rightPadding
  );
};

export type KanbanProviderProps = {
  children: ReactNode;
  onDragEnd: (event: DragEndEvent) => void;
  className?: string;
};

export const KanbanProvider = ({
  children,
  onDragEnd,
  className,
}: KanbanProviderProps) => {
  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: { distance: 8 },
    })
  );

  return (
    <DndContext
      collisionDetection={rectIntersection}
      onDragEnd={onDragEnd}
      sensors={sensors}
      modifiers={[restrictToFirstScrollableAncestorCustom]}
    >
      <div
        className={cn(
          'inline-grid grid-flow-col auto-cols-[minmax(200px,400px)] divide-x divide-white/10 border-x border-white/10 h-full',
          className
        )}
      >
        {children}
      </div>
    </DndContext>
  );
};
