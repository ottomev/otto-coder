import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { webAssistApi } from '@/lib/webassist-api';
import type { ApprovalDecision } from 'shared/types';

export const useWebAssistApprovals = (projectId: string | undefined) => {
  const queryClient = useQueryClient();

  const approvalsQuery = useQuery({
    queryKey: ['webassist-approvals', projectId],
    queryFn: () => {
      if (!projectId) throw new Error('Project ID is required');
      return webAssistApi.getApprovals(projectId);
    },
    enabled: !!projectId,
    refetchInterval: 5000,
  });

  const submitApprovalMutation = useMutation({
    mutationFn: ({
      approvalId,
      decision,
    }: {
      approvalId: string;
      decision: ApprovalDecision;
    }) => webAssistApi.submitApproval(approvalId, decision),
    onSuccess: () => {
      // Invalidate and refetch
      queryClient.invalidateQueries({ queryKey: ['webassist-approvals', projectId] });
      queryClient.invalidateQueries({ queryKey: ['webassist-project', projectId] });
      queryClient.invalidateQueries({ queryKey: ['webassist-projects'] });
    },
  });

  return {
    approvals: approvalsQuery.data,
    isLoading: approvalsQuery.isLoading,
    error: approvalsQuery.error,
    submitApproval: submitApprovalMutation.mutate,
    isSubmitting: submitApprovalMutation.isPending,
  };
};
