import { useMutation, useQueryClient } from '@tanstack/react-query';
import { attemptsApi } from '@/lib/api';

export function useMerge(
  attemptId?: string,
  onSuccess?: () => void,
  onError?: (err: unknown) => void
) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: () => {
      if (!attemptId) return Promise.resolve();
      return attemptsApi.merge(attemptId);
    },
    onSuccess: async () => {
      // Refresh attempt-specific branch information
      await queryClient.invalidateQueries({ queryKey: ['branchStatus', attemptId] });

      // If a merge can change the list of branches shown elsewhere
      await queryClient.invalidateQueries({ queryKey: ['projectBranches'] });

      // Refetch task attempts to get updated merge status
      await queryClient.refetchQueries({ queryKey: ['taskAttempts'] });

      onSuccess?.();
    },
    onError: (err) => {
      console.error('Failed to merge:', err);
      onError?.(err);
    },
  });
}
