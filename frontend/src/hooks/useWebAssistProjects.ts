import { useQuery } from '@tanstack/react-query';
import { webAssistApi } from '@/lib/webassist-api';

export const useWebAssistProjects = () => {
  return useQuery({
    queryKey: ['webassist-projects'],
    queryFn: () => webAssistApi.getAll(),
    refetchInterval: 10000, // Refetch every 10 seconds
  });
};
