import { useQuery } from '@tanstack/react-query';
import { webAssistApi } from '@/lib/webassist-api';

export const useWebAssistProject = (projectId: string | undefined) => {
  return useQuery({
    queryKey: ['webassist-project', projectId],
    queryFn: () => {
      if (!projectId) throw new Error('Project ID is required');
      return webAssistApi.getById(projectId);
    },
    enabled: !!projectId,
    refetchInterval: 5000, // Refetch every 5 seconds for real-time updates
  });
};
