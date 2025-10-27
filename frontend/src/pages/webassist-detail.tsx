import { useParams } from 'react-router-dom';
import { useWebAssistProject } from '@/hooks/useWebAssistProject';
import { useWebAssistApprovals } from '@/hooks/useWebAssistApprovals';
import { useWebAssistSSE } from '@/hooks/useWebAssistSSE';
import { StageProgressBar } from '@/components/webassist/StageProgressBar';
import { Loader } from '@/components/ui/loader';
import { Button } from '@/components/ui/button';
import { AlertCircle, CheckCircle, Clock } from 'lucide-react';
import { Link } from 'react-router-dom';

export function WebAssistDetail() {
  const { projectId } = useParams<{ projectId: string }>();
  const { data: project, isLoading, error } = useWebAssistProject(projectId);
  const { approvals, submitApproval, isSubmitting } = useWebAssistApprovals(projectId);
  const { isConnected } = useWebAssistSSE(projectId);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <Loader message="Loading project..." size={32} />
      </div>
    );
  }

  if (error || !project) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="text-center">
          <AlertCircle className="h-12 w-12 text-red-500 mx-auto mb-4" />
          <h2 className="text-xl font-semibold mb-2">Project Not Found</h2>
          <Link to="/webassist" className="text-blue-500 hover:underline">
            Back to Projects
          </Link>
        </div>
      </div>
    );
  }

  const pendingApprovals = approvals?.filter((a) => a.status.status === 'pending') || [];

  return (
    <div className="container mx-auto p-6">
      <div className="mb-6">
        <Link to="/webassist" className="text-blue-500 hover:underline mb-2 inline-block">
          ← Back to Projects
        </Link>
        <div className="flex items-center justify-between">
          <h1 className="text-3xl font-bold">WebAssist Project Details</h1>
          {isConnected && (
            <span className="flex items-center gap-2 text-sm text-green-600">
              <span className="h-2 w-2 bg-green-500 rounded-full animate-pulse" />
              Live
            </span>
          )}
        </div>
      </div>

      <div className="grid gap-6">
        {/* Progress Section */}
        <div className="border rounded-lg p-6 bg-card">
          <h2 className="text-xl font-semibold mb-4">Progress</h2>
          <div className="mb-2">
            <span className="text-sm text-muted-foreground">Current Stage:</span>
            <p className="text-lg font-medium capitalize">
              {project.current_stage.replace('_', ' ')}
            </p>
          </div>
          <StageProgressBar currentStage={project.current_stage} className="mt-4" />
          <div className="mt-2 text-sm text-muted-foreground">
            Sync Status: {project.sync_status}
          </div>
        </div>

        {/* Pending Approvals */}
        {pendingApprovals.length > 0 && (
          <div className="border rounded-lg p-6 bg-yellow-50 dark:bg-yellow-950 border-yellow-200 dark:border-yellow-800">
            <h2 className="text-xl font-semibold mb-4 flex items-center gap-2">
              <Clock className="h-5 w-5" />
              Pending Approvals ({pendingApprovals.length})
            </h2>
            {pendingApprovals.map((approval) => (
              <div key={approval.id} className="border rounded p-4 mb-4 bg-white dark:bg-gray-900">
                <div className="mb-3">
                  <h3 className="font-semibold capitalize">
                    {approval.stage_name.replace('_', ' ')}
                  </h3>
                  <p className="text-sm text-muted-foreground">
                    Requested: {new Date(approval.requested_at).toLocaleString()}
                  </p>
                  {approval.preview_url && (
                    <a
                      href={approval.preview_url}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-blue-500 hover:underline text-sm"
                    >
                      View Preview →
                    </a>
                  )}
                </div>
                <div className="flex gap-2">
                  <Button
                    size="sm"
                    onClick={() =>
                      submitApproval({
                        approvalId: approval.id,
                        decision: { status: { status: 'approved' }, feedback: null },
                      })
                    }
                    disabled={isSubmitting}
                  >
                    <CheckCircle className="h-4 w-4 mr-1" />
                    Approve
                  </Button>
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={() =>
                      submitApproval({
                        approvalId: approval.id,
                        decision: { status: { status: 'denied' }, feedback: 'Needs revision' },
                      })
                    }
                    disabled={isSubmitting}
                  >
                    <AlertCircle className="h-4 w-4 mr-1" />
                    Reject
                  </Button>
                </div>
              </div>
            ))}
          </div>
        )}

        {/* Tasks Section */}
        <div className="border rounded-lg p-6 bg-card">
          <h2 className="text-xl font-semibold mb-4">Stage Tasks</h2>
          <div className="space-y-2">
            {project.tasks.map((task: import('shared/types').WebAssistTaskStatus) => (
              <div key={task.task_id} className="flex items-center justify-between p-3 border rounded">
                <div>
                  <p className="font-medium capitalize">{task.stage.replace('_', ' ')}</p>
                  <p className="text-sm text-muted-foreground">Status: {task.status}</p>
                </div>
                {task.completed_at && (
                  <CheckCircle className="h-5 w-5 text-green-500" />
                )}
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
