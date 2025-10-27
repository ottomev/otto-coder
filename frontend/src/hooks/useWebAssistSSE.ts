import { useEffect, useState, useCallback } from 'react';
import { useQueryClient } from '@tanstack/react-query';

interface WebAssistEvent {
  type:
    | 'stage_changed'
    | 'approval_requested'
    | 'approval_responded'
    | 'task_started'
    | 'task_completed'
    | 'sync_status_changed'
    | 'error';
  project_id?: string;
  old_stage?: string;
  new_stage?: string;
  approval_id?: string;
  stage?: string;
  status?: string;
  task_id?: string;
  old_status?: string;
  new_status?: string;
  message?: string;
}

export const useWebAssistSSE = (projectId: string | undefined) => {
  const [isConnected, setIsConnected] = useState(false);
  const [lastEvent, setLastEvent] = useState<WebAssistEvent | null>(null);
  const queryClient = useQueryClient();

  const handleEvent = useCallback(
    (event: WebAssistEvent) => {
      setLastEvent(event);

      // Invalidate relevant queries based on event type
      switch (event.type) {
        case 'stage_changed':
          queryClient.invalidateQueries({ queryKey: ['webassist-project', projectId] });
          queryClient.invalidateQueries({ queryKey: ['webassist-projects'] });
          break;
        case 'approval_requested':
        case 'approval_responded':
          queryClient.invalidateQueries({ queryKey: ['webassist-approvals', projectId] });
          queryClient.invalidateQueries({ queryKey: ['webassist-project', projectId] });
          break;
        case 'task_started':
        case 'task_completed':
          queryClient.invalidateQueries({ queryKey: ['webassist-project', projectId] });
          break;
        case 'sync_status_changed':
          queryClient.invalidateQueries({ queryKey: ['webassist-project', projectId] });
          queryClient.invalidateQueries({ queryKey: ['webassist-projects'] });
          break;
      }
    },
    [projectId, queryClient]
  );

  useEffect(() => {
    if (!projectId) return;

    const eventSource = new EventSource(
      `/api/web-assist/projects/${projectId}/events`
    );

    eventSource.onopen = () => {
      setIsConnected(true);
    };

    eventSource.onmessage = (e) => {
      try {
        const event: WebAssistEvent = JSON.parse(e.data);
        handleEvent(event);
      } catch (error) {
        console.error('Failed to parse SSE event:', error);
      }
    };

    eventSource.onerror = () => {
      setIsConnected(false);
    };

    return () => {
      eventSource.close();
      setIsConnected(false);
    };
  }, [projectId, handleEvent]);

  return {
    isConnected,
    lastEvent,
  };
};
