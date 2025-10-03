import { Virtuoso, VirtuosoHandle } from 'react-virtuoso';
import { useEffect, useRef, useState } from 'react';

import DisplayConversationEntry from '../NormalizedConversation/DisplayConversationEntry';
import { useEntries } from '@/contexts/EntriesContext';
import {
  AddEntryType,
  PatchTypeWithKey,
  useConversationHistory,
} from '@/hooks/useConversationHistory';
import { Loader2 } from 'lucide-react';
import { TaskAttempt } from 'shared/types';

interface VirtualizedListProps {
  attempt: TaskAttempt;
}

const VirtualizedList = ({ attempt }: VirtualizedListProps) => {
  const [entries, setEntriesState] = useState<PatchTypeWithKey[]>([]);
  const [loading, setLoading] = useState(true);
  const [atBottom, setAtBottom] = useState(true);
  const { setEntries, reset } = useEntries();

  const virtuosoRef = useRef<VirtuosoHandle | null>(null);
  const didInitialScroll = useRef(false);
  const prevLengthRef = useRef(0);

  useEffect(() => {
    setLoading(true);
    setEntriesState([]);
    reset();
    didInitialScroll.current = false;
    prevLengthRef.current = 0;
  }, [attempt.id, reset]);

  // Initial scroll to bottom once data loads
  useEffect(() => {
    if (!didInitialScroll.current && entries.length > 0) {
      didInitialScroll.current = true;
      requestAnimationFrame(() => {
        virtuosoRef.current?.scrollToIndex({
          index: entries.length - 1,
          align: 'end',
          behavior: 'auto',
        });
      });
    }
  }, [entries.length]);

  // Handle large bursts of new entries while at bottom
  useEffect(() => {
    const prev = prevLengthRef.current;
    const grewBy = entries.length - prev;
    prevLengthRef.current = entries.length;

    const LARGE_BURST = 5;
    if (grewBy >= LARGE_BURST && atBottom && entries.length > 0 && didInitialScroll.current) {
      requestAnimationFrame(() => {
        virtuosoRef.current?.scrollToIndex({
          index: entries.length - 1,
          align: 'end',
          behavior: 'smooth',
        });
      });
    }
  }, [entries.length, atBottom]);

  const onEntriesUpdated = (
    newEntries: PatchTypeWithKey[],
    _addType: AddEntryType,
    newLoading: boolean
  ) => {
    setEntriesState(newEntries);
    setEntries(newEntries);

    if (loading) {
      setLoading(newLoading);
    }
  };

  useConversationHistory({ attempt, onEntriesUpdated });

  const itemContent = (_index: number, data: PatchTypeWithKey) => {
    if (data.type === 'STDOUT') {
      return <p>{data.content}</p>;
    }
    if (data.type === 'STDERR') {
      return <p>{data.content}</p>;
    }
    if (data.type === 'NORMALIZED_ENTRY') {
      return (
        <DisplayConversationEntry
          expansionKey={data.patchKey}
          entry={data.content}
          executionProcessId={data.executionProcessId}
          taskAttempt={attempt}
        />
      );
    }
    return null;
  };

  const computeItemKey = (_index: number, data: PatchTypeWithKey) =>
    `l-${data.patchKey}`;

  return (
    <>
      <Virtuoso<PatchTypeWithKey>
        ref={virtuosoRef}
        className="flex-1"
        data={entries}
        itemContent={itemContent}
        computeItemKey={computeItemKey}
        atBottomStateChange={setAtBottom}
        followOutput={atBottom ? 'smooth' : false}
        increaseViewportBy={{ top: 0, bottom: 600 }}
        components={{
          Header: () => <div className="h-2"></div>,
          Footer: () => <div className="h-2"></div>,
        }}
      />
      {loading && (
        <div className="absolute top-0 left-0 w-full h-full bg-primary flex flex-col gap-2 justify-center items-center">
          <Loader2 className="h-8 w-8 animate-spin" />
          <p>Loading History</p>
        </div>
      )}
    </>
  );
};

export default VirtualizedList;
