import { useState, useEffect } from 'react';
import { Label } from '@/components/ui/label';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { Alert, AlertDescription } from '@/components/ui/alert';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import {
  AlertCircle,
  Folder,
  Search,
  FolderGit,
  FolderPlus,
  ArrowLeft,
} from 'lucide-react';
import {
  createScriptPlaceholderStrategy,
  ScriptPlaceholderContext,
} from '@/utils/script-placeholders';
import { useUserSystem } from '@/components/config-provider';
import { CopyFilesField } from './copy-files-field';
// Removed collapsible sections for simplicity; show fields always in edit mode
import { fileSystemApi } from '@/lib/api';
import { showFolderPicker } from '@/lib/modals';
import { DirectoryEntry, GitHubAccountSafe } from 'shared/types';
import { generateProjectNameFromPath } from '@/utils/string';

interface ProjectFormFieldsProps {
  isEditing: boolean;
  repoMode: 'existing' | 'new';
  setRepoMode: (mode: 'existing' | 'new') => void;
  gitRepoPath: string;
  handleGitRepoPathChange: (path: string) => void;
  parentPath: string;
  setParentPath: (path: string) => void;
  setFolderName: (name: string) => void;
  setName: (name: string) => void;
  name: string;
  setupScript: string;
  setSetupScript: (script: string) => void;
  devScript: string;
  setDevScript: (script: string) => void;
  cleanupScript: string;
  setCleanupScript: (script: string) => void;
  copyFiles: string;
  setCopyFiles: (files: string) => void;
  error: string;
  setError: (error: string) => void;
  projectId?: string;
  onCreateProject?: (path: string, name: string) => void;
  githubAccountId: string | null;
  setGithubAccountId: (id: string | null) => void;
  availableAccounts: GitHubAccountSafe[];
}

export function ProjectFormFields({
  isEditing,
  repoMode,
  setRepoMode,
  gitRepoPath,
  handleGitRepoPathChange,
  parentPath,
  setParentPath,
  setFolderName,
  setName,
  name,
  setupScript,
  setSetupScript,
  devScript,
  setDevScript,
  cleanupScript,
  setCleanupScript,
  copyFiles,
  setCopyFiles,
  error,
  setError,
  projectId,
  onCreateProject,
  githubAccountId,
  setGithubAccountId,
  availableAccounts,
}: ProjectFormFieldsProps) {
  const { system } = useUserSystem();

  // Create strategy-based placeholders
  const placeholders = system.environment
    ? new ScriptPlaceholderContext(
        createScriptPlaceholderStrategy(system.environment.os_type)
      ).getPlaceholders()
    : {
        setup: '#!/bin/bash\nnpm install\n# Add any setup commands here...',
        dev: '#!/bin/bash\nnpm run dev\n# Add dev server start command here...',
        cleanup:
          '#!/bin/bash\n# Add cleanup commands here...\n# This runs after coding agent execution',
      };

  // Repository loading state
  const [allRepos, setAllRepos] = useState<DirectoryEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const [reposError, setReposError] = useState('');
  const [showMoreOptions, setShowMoreOptions] = useState(false);
  const [showRecentRepos, setShowRecentRepos] = useState(false);

  // Lazy-load repositories when the user navigates to the repo list
  useEffect(() => {
    if (!isEditing && showRecentRepos && !loading && allRepos.length === 0) {
      loadRecentRepos();
    }
  }, [isEditing, showRecentRepos]);

  const loadRecentRepos = async () => {
    setLoading(true);
    setReposError('');

    try {
      const discoveredRepos = await fileSystemApi.listGitRepos();
      setAllRepos(discoveredRepos);
    } catch (err) {
      setReposError('Failed to load repositories');
      console.error('Failed to load repos:', err);
    } finally {
      setLoading(false);
    }
  };

  return (
    <>
      {!isEditing && repoMode === 'existing' && (
        <div className="space-y-4">
          {/* Show selection interface only when no repo is selected */}
          <>
            {/* Initial choice cards - Stage 1 */}
            {!showRecentRepos && (
              <>
                {/* From Git Repository card */}
                <div
                  className="p-4 glass-card cursor-pointer hover:shadow-xl hover:brightness-110 transition-all rounded-lg"
                  style={{ background: 'rgba(255, 255, 255, 0.05)' }}
                  onClick={() => setShowRecentRepos(true)}
                >
                  <div className="flex items-start gap-3">
                    <FolderGit className="h-5 w-5 mt-0.5 flex-shrink-0 text-muted-foreground" />
                    <div className="min-w-0 flex-1">
                      <div className="font-medium text-foreground">
                        From Git Repository
                      </div>
                      <div className="text-xs text-muted-foreground mt-1">
                        Use an existing repository as your project base
                      </div>
                    </div>
                  </div>
                </div>

                {/* Create Blank Project card */}
                <div
                  className="p-4 glass-card cursor-pointer hover:shadow-xl hover:brightness-110 transition-all rounded-lg"
                  style={{ background: 'rgba(255, 255, 255, 0.05)' }}
                  onClick={() => {
                    setRepoMode('new');
                    setError('');
                  }}
                >
                  <div className="flex items-start gap-3">
                    <FolderPlus className="h-5 w-5 mt-0.5 flex-shrink-0 text-muted-foreground" />
                    <div className="min-w-0 flex-1">
                      <div className="font-medium text-foreground">
                        Create Blank Project
                      </div>
                      <div className="text-xs text-muted-foreground mt-1">
                        Start a new project from scratch
                      </div>
                    </div>
                  </div>
                </div>
              </>
            )}

            {/* Repository selection - Stage 2A */}
            {showRecentRepos && (
              <>
                {/* Back button */}
                <button
                  className="text-sm text-muted-foreground hover:text-foreground flex items-center gap-1 mb-4"
                  onClick={() => {
                    setShowRecentRepos(false);
                    setError('');
                  }}
                >
                  <ArrowLeft className="h-3 w-3" />
                  Back to options
                </button>

                {/* Repository cards */}
                {!loading && allRepos.length > 0 && (
                  <div className="space-y-2">
                    {allRepos
                      .slice(0, showMoreOptions ? allRepos.length : 3)
                      .map((repo) => (
                        <div
                          key={repo.path}
                          className="p-4 glass-card cursor-pointer hover:shadow-xl transition-all rounded-lg"
                          onClick={() => {
                            setError('');
                            const cleanName = generateProjectNameFromPath(
                              repo.path
                            );
                            onCreateProject?.(repo.path, cleanName);
                          }}
                        >
                          <div className="flex items-start gap-3">
                            <FolderGit className="h-5 w-5 mt-0.5 flex-shrink-0 text-muted-foreground" />
                            <div className="min-w-0 flex-1">
                              <div className="font-medium text-foreground">
                                {repo.name}
                              </div>
                              <div className="text-xs text-muted-foreground truncate mt-1">
                                {repo.path}
                              </div>
                            </div>
                          </div>
                        </div>
                      ))}

                    {/* Show more/less for repositories */}
                    {!showMoreOptions && allRepos.length > 3 && (
                      <button
                        className="text-sm text-muted-foreground hover:text-foreground transition-colors text-left"
                        onClick={() => setShowMoreOptions(true)}
                      >
                        Show {allRepos.length - 3} more repositories
                      </button>
                    )}
                    {showMoreOptions && allRepos.length > 3 && (
                      <button
                        className="text-sm text-muted-foreground hover:text-foreground transition-colors text-left"
                        onClick={() => setShowMoreOptions(false)}
                      >
                        Show less
                      </button>
                    )}
                  </div>
                )}

                {/* Loading state */}
                {loading && (
                  <div className="p-4 glass-card rounded-lg">
                    <div className="flex items-center gap-3">
                      <div className="animate-spin h-5 w-5 border-2 border-muted-foreground border-t-transparent rounded-full"></div>
                      <div className="text-sm text-muted-foreground">
                        Loading repositories...
                      </div>
                    </div>
                  </div>
                )}

                {/* Error state */}
                {!loading && reposError && (
                  <div className="p-4 border border-destructive rounded-lg bg-destructive/5">
                    <div className="flex items-center gap-3">
                      <AlertCircle className="h-5 w-5 text-destructive flex-shrink-0" />
                      <div className="text-sm text-destructive">
                        {reposError}
                      </div>
                    </div>
                  </div>
                )}

                {/* Browse for repository card */}
                <div
                  className="p-4 border border-dashed cursor-pointer hover:shadow-xl transition-all rounded-lg glass-surface"
                  onClick={async () => {
                    setError('');
                    const selectedPath = await showFolderPicker({
                      title: 'Select Git Repository',
                      description: 'Choose an existing git repository',
                    });
                    if (selectedPath) {
                      const projectName =
                        generateProjectNameFromPath(selectedPath);
                      if (onCreateProject) {
                        onCreateProject(selectedPath, projectName);
                      }
                    }
                  }}
                >
                  <div className="flex items-start gap-3">
                    <Search className="h-5 w-5 mt-0.5 flex-shrink-0 text-muted-foreground" />
                    <div className="min-w-0 flex-1">
                      <div className="font-medium text-foreground">
                        Search all repos
                      </div>
                      <div className="text-xs text-muted-foreground mt-1">
                        Browse and select any repository on your system
                      </div>
                    </div>
                  </div>
                </div>
              </>
            )}
          </>
        </div>
      )}

      {/* Blank Project Form */}
      {!isEditing && repoMode === 'new' && (
        <div className="space-y-4">
          {/* Back button */}
          <Button
            type="button"
            variant="ghost"
            size="sm"
            onClick={() => {
              setRepoMode('existing');
              setError('');
              setName('');
              setParentPath('');
              setFolderName('');
            }}
            className="flex items-center gap-2"
          >
            <ArrowLeft className="h-4 w-4" />
            Back to options
          </Button>

          <div className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="new-project-name">
                Project Name <span className="text-red-500">*</span>
              </Label>
              <Input
                id="new-project-name"
                type="text"
                value={name}
                onChange={(e) => {
                  setName(e.target.value);
                  if (e.target.value) {
                    setFolderName(
                      e.target.value
                        .toLowerCase()
                        .replace(/\s+/g, '-')
                        .replace(/[^a-z0-9-]/g, '')
                    );
                  }
                }}
                placeholder="My Awesome Project"
                className="placeholder:text-secondary-foreground placeholder:opacity-100"
                required
              />
              <p className="text-xs text-muted-foreground">
                The folder name will be auto-generated from the project name
              </p>
            </div>

            <div className="space-y-2">
              <Label htmlFor="parent-path">Parent Directory</Label>
              <div className="flex space-x-2">
                <Input
                  id="parent-path"
                  type="text"
                  value={parentPath}
                  onChange={(e) => setParentPath(e.target.value)}
                  placeholder="Current Directory"
                  className="flex-1 placeholder:text-secondary-foreground placeholder:opacity-100"
                />
                <Button
                  type="button"
                  variant="ghost"
                  size="icon"
                  onClick={async () => {
                    const selectedPath = await showFolderPicker({
                      title: 'Select Parent Directory',
                      description: 'Choose where to create the new repository',
                      value: parentPath,
                    });
                    if (selectedPath) {
                      setParentPath(selectedPath);
                    }
                  }}
                >
                  <Folder className="h-4 w-4" />
                </Button>
              </div>
              <p className="text-xs text-muted-foreground">
                Leave empty to use your current working directory, or specify a
                custom path.
              </p>
            </div>
          </div>
        </div>
      )}

      {isEditing && (
        <>
          <div className="space-y-2">
            <Label htmlFor="git-repo-path">Git Repository Path</Label>
            <div className="flex space-x-2">
              <Input
                id="git-repo-path"
                type="text"
                value={gitRepoPath}
                onChange={(e) => handleGitRepoPathChange(e.target.value)}
                placeholder="/path/to/your/existing/repo"
                required
                className="flex-1"
              />
              <Button
                type="button"
                variant="outline"
                onClick={async () => {
                  const selectedPath = await showFolderPicker({
                    title: 'Select Git Repository',
                    description: 'Choose an existing git repository',
                    value: gitRepoPath,
                  });
                  if (selectedPath) {
                    handleGitRepoPathChange(selectedPath);
                  }
                }}
              >
                <Folder className="h-4 w-4" />
              </Button>
            </div>
          </div>

          <div className="space-y-2">
            <Label htmlFor="name">Project Name</Label>
            <Input
              id="name"
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="Enter project name"
              required
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="github-account">GitHub Account</Label>
            <Select
              value={githubAccountId || 'none'}
              onValueChange={(value) =>
                setGithubAccountId(value === 'none' ? null : value)
              }
            >
              <SelectTrigger id="github-account">
                <SelectValue placeholder="Use global token" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="none">Use global token</SelectItem>
                {availableAccounts.map((account) => (
                  <SelectItem key={account.id} value={account.id}>
                    {account.username}
                    {account.primary_email && ` (${account.primary_email})`}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <p className="text-sm text-muted-foreground">
              Select a GitHub account for this project, or use the global token
              from General Settings
            </p>
          </div>
        </>
      )}

      {isEditing && (
        <div className="space-y-4 pt-4 border-t border-border">
          <div className="space-y-2">
            <Label htmlFor="setup-script">Setup Script</Label>
            <textarea
              id="setup-script"
              value={setupScript}
              onChange={(e) => setSetupScript(e.target.value)}
              placeholder={placeholders.setup}
              rows={4}
              className="w-full px-3 py-2 text-sm glass-input text-foreground rounded-md resize-vertical focus:outline-none focus:ring-0"
            />
            <p className="text-sm text-muted-foreground">
              This script will run after creating the worktree and before the
              coding agent starts. Use it for setup tasks like installing
              dependencies or preparing the environment.
            </p>
          </div>

          <div className="space-y-2">
            <Label htmlFor="dev-script">Dev Server Script</Label>
            <textarea
              id="dev-script"
              value={devScript}
              onChange={(e) => setDevScript(e.target.value)}
              placeholder={placeholders.dev}
              rows={4}
              className="w-full px-3 py-2 text-sm glass-input text-foreground rounded-md resize-vertical focus:outline-none focus:ring-0"
            />
            <p className="text-sm text-muted-foreground">
              This script can be run from task attempts to start a development
              server. Use it to quickly start your project's dev server for
              testing changes.
            </p>
          </div>

          <div className="space-y-2">
            <Label htmlFor="cleanup-script">Cleanup Script</Label>
            <textarea
              id="cleanup-script"
              value={cleanupScript}
              onChange={(e) => setCleanupScript(e.target.value)}
              placeholder={placeholders.cleanup}
              rows={4}
              className="w-full px-3 py-2 text-sm glass-input text-foreground rounded-md resize-vertical focus:outline-none focus:ring-0"
            />
            <p className="text-sm text-muted-foreground">
              This script runs after coding agent execution{' '}
              <strong>only if changes were made</strong>. Use it for quality
              assurance tasks like running linters, formatters, tests, or other
              validation steps. If no changes are made, this script is skipped.
            </p>
          </div>

          <div className="space-y-2">
            <Label>Copy Files</Label>
            <CopyFilesField
              value={copyFiles}
              onChange={setCopyFiles}
              projectId={projectId}
            />
            <p className="text-sm text-muted-foreground">
              Comma-separated list of files to copy from the original project
              directory to the worktree. These files will be copied after the
              worktree is created but before the setup script runs. Useful for
              environment-specific files like .env, configuration files, and
              local settings. Make sure these are gitignored or they could get
              committed!
            </p>
          </div>
        </div>
      )}

      {error && (
        <Alert variant="destructive">
          <AlertCircle className="h-4 w-4" />
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      )}
    </>
  );
}
