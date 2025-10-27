import { useWebAssistProjects } from '@/hooks/useWebAssistProjects';
import { Link } from 'react-router-dom';
import { StageProgressBar } from '@/components/webassist/StageProgressBar';
import { Loader } from '@/components/ui/loader';
import { Globe, AlertCircle } from 'lucide-react';

export function WebAssistDashboard() {
  const { data: projects, isLoading, error } = useWebAssistProjects();

  if (isLoading) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <Loader message="Loading WebAssist projects..." size={32} />
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="text-center">
          <AlertCircle className="h-12 w-12 text-red-500 mx-auto mb-4" />
          <h2 className="text-xl font-semibold mb-2">Error Loading Projects</h2>
          <p className="text-muted-foreground">{error.message}</p>
        </div>
      </div>
    );
  }

  return (
    <div className="container mx-auto p-6">
      <div className="flex items-center justify-between mb-8">
        <div className="flex items-center gap-3">
          <Globe className="h-8 w-8 text-blue-500" />
          <div>
            <h1 className="text-3xl font-bold">WebAssist Projects</h1>
            <p className="text-muted-foreground">
              AI-powered website development workflow
            </p>
          </div>
        </div>
      </div>

      {projects && projects.length === 0 ? (
        <div className="text-center py-16">
          <Globe className="h-16 w-16 text-gray-300 mx-auto mb-4" />
          <h3 className="text-lg font-semibold mb-2">No WebAssist Projects Yet</h3>
          <p className="text-muted-foreground">
            WebAssist projects will appear here when created from Supabase
          </p>
        </div>
      ) : (
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {projects?.map((project) => (
            <Link
              key={project.id}
              to={`/webassist/${project.webassist_project_id}`}
              className="block"
            >
              <div className="border rounded-lg p-6 hover:shadow-lg transition-shadow bg-card">
                <div className="flex items-start justify-between mb-4">
                  <div>
                    <h3 className="font-semibold text-lg mb-1">
                      {project.company_name}
                    </h3>
                    <p className="text-sm text-muted-foreground">
                      Stage: {project.current_stage.replace('_', ' ')}
                    </p>
                  </div>
                  {project.pending_approvals_count > 0 && (
                    <span className="bg-yellow-100 text-yellow-800 text-xs font-medium px-2.5 py-0.5 rounded dark:bg-yellow-900 dark:text-yellow-300">
                      {project.pending_approvals_count} pending
                    </span>
                  )}
                </div>

                <StageProgressBar currentStage={project.current_stage} className="mb-4" />

                <div className="flex items-center justify-between text-sm">
                  <span className="text-muted-foreground">
                    {project.sync_status}
                  </span>
                  <span className="text-muted-foreground">
                    {new Date(project.updated_at).toLocaleDateString()}
                  </span>
                </div>
              </div>
            </Link>
          ))}
        </div>
      )}
    </div>
  );
}
