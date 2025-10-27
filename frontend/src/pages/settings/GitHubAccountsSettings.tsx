import { useCallback, useEffect, useState } from 'react';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Alert, AlertDescription } from '@/components/ui/alert';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Check, Loader2, MoreVertical, Plus, X, Key, AlertCircle } from 'lucide-react';
import { githubAccountsApi } from '@/lib/api';
import type { GitHubAccountSafe, CreateGitHubAccount, UpdateGitHubAccount } from 'shared/types';
import NiceModal from '@ebay/nice-modal-react';

export function GitHubAccountsSettings() {
  const [accounts, setAccounts] = useState<GitHubAccountSafe[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [editingAccount, setEditingAccount] = useState<GitHubAccountSafe | null>(null);
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [validatingId, setValidatingId] = useState<string | null>(null);
  const [validationResults, setValidationResults] = useState<Record<string, { valid: boolean; error?: string }>>({});

  // Form state
  const [formData, setFormData] = useState<CreateGitHubAccount | UpdateGitHubAccount>({
    username: '',
    oauth_token: null,
    pat: null,
    primary_email: null,
  });
  const [formError, setFormError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  const loadAccounts = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await githubAccountsApi.getAll();
      setAccounts(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load GitHub accounts');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadAccounts();
  }, [loadAccounts]);

  const handleOpenDialog = (account?: GitHubAccountSafe) => {
    if (account) {
      setEditingAccount(account);
      setFormData({
        username: account.username,
        oauth_token: null, // Don't pre-fill tokens for security
        pat: null,
        primary_email: account.primary_email,
      });
    } else {
      setEditingAccount(null);
      setFormData({
        username: '',
        oauth_token: null,
        pat: null,
        primary_email: null,
      });
    }
    setFormError(null);
    setIsDialogOpen(true);
  };

  const handleCloseDialog = () => {
    setIsDialogOpen(false);
    setEditingAccount(null);
    setFormData({
      username: '',
      oauth_token: null,
      pat: null,
      primary_email: null,
    });
    setFormError(null);
  };

  const handleSave = async () => {
    setFormError(null);
    setSaving(true);

    try {
      if (!formData.username?.trim()) {
        setFormError('Username is required');
        return;
      }

      if (editingAccount) {
        // Update existing account
        await githubAccountsApi.update(editingAccount.id, formData as UpdateGitHubAccount);
      } else {
        // Create new account
        if (!formData.oauth_token && !formData.pat) {
          setFormError('Either OAuth token or Personal Access Token is required');
          return;
        }
        await githubAccountsApi.create(formData as CreateGitHubAccount);
      }

      await loadAccounts();
      handleCloseDialog();
    } catch (err) {
      setFormError(err instanceof Error ? err.message : 'Failed to save account');
    } finally {
      setSaving(false);
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Are you sure you want to delete this GitHub account? Projects using this account will fall back to the global token.')) {
      return;
    }

    try {
      await githubAccountsApi.delete(id);
      await loadAccounts();
      // Clear validation result for deleted account
      setValidationResults((prev) => {
        const newResults = { ...prev };
        delete newResults[id];
        return newResults;
      });
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to delete account');
    }
  };

  const handleValidate = async (id: string) => {
    setValidatingId(id);
    try {
      const result = await githubAccountsApi.validate(id);
      setValidationResults((prev) => ({ ...prev, [id]: result }));
    } catch (err) {
      setValidationResults((prev) => ({
        ...prev,
        [id]: { valid: false, error: 'Failed to validate token' },
      }));
    } finally {
      setValidatingId(null);
    }
  };

  const handleConnectGitHub = () => {
    NiceModal.show('github-login').finally(() => {
      NiceModal.hide('github-login');
      // Optionally reload accounts after OAuth
      // loadAccounts();
    });
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center py-8">
        <Loader2 className="h-8 w-8 animate-spin" />
        <span className="ml-2">Loading GitHub accounts...</span>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {error && (
        <Alert variant="destructive">
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      )}

      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="flex items-center gap-2">
                <Key className="h-5 w-5" />
                GitHub Accounts
              </CardTitle>
              <CardDescription>
                Manage multiple GitHub accounts for different projects. Each project can use a
                specific account or fall back to the global token.
              </CardDescription>
            </div>
            <div className="flex gap-2">
              <Button variant="outline" onClick={handleConnectGitHub} size="sm">
                Connect via OAuth
              </Button>
              <Button onClick={() => handleOpenDialog()} size="sm">
                <Plus className="h-4 w-4 mr-2" />
                Add Account
              </Button>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          {accounts.length === 0 ? (
            <div className="text-center py-8 text-muted-foreground">
              <Key className="h-12 w-12 mx-auto mb-4 opacity-20" />
              <p>No GitHub accounts configured yet.</p>
              <p className="text-sm mt-2">
                Add an account to use project-specific authentication.
              </p>
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Username</TableHead>
                  <TableHead>Email</TableHead>
                  <TableHead>Auth Type</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead className="text-right">Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {accounts.map((account) => {
                  const authType = account.has_token ? 'Token' : 'None';
                  const validationResult = validationResults[account.id];
                  const isValidating = validatingId === account.id;

                  return (
                    <TableRow key={account.id}>
                      <TableCell className="font-medium">{account.username}</TableCell>
                      <TableCell className="text-muted-foreground">
                        {account.primary_email || '—'}
                      </TableCell>
                      <TableCell>
                        <span className="text-sm px-2 py-1 bg-secondary rounded">
                          {authType}
                        </span>
                      </TableCell>
                      <TableCell>
                        {isValidating ? (
                          <Loader2 className="h-4 w-4 animate-spin inline" />
                        ) : validationResult ? (
                          <div className="flex items-center gap-1">
                            {validationResult.valid ? (
                              <>
                                <Check className="h-4 w-4 text-green-600" />
                                <span className="text-sm text-green-600">Valid</span>
                              </>
                            ) : (
                              <>
                                <X className="h-4 w-4 text-red-600" />
                                <span className="text-sm text-red-600">
                                  {validationResult.error || 'Invalid'}
                                </span>
                              </>
                            )}
                          </div>
                        ) : (
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => handleValidate(account.id)}
                          >
                            Validate
                          </Button>
                        )}
                      </TableCell>
                      <TableCell className="text-right">
                        <DropdownMenu>
                          <DropdownMenuTrigger asChild>
                            <Button variant="ghost" size="sm">
                              <MoreVertical className="h-4 w-4" />
                            </Button>
                          </DropdownMenuTrigger>
                          <DropdownMenuContent align="end">
                            <DropdownMenuItem onClick={() => handleOpenDialog(account)}>
                              Edit
                            </DropdownMenuItem>
                            <DropdownMenuItem onClick={() => handleValidate(account.id)}>
                              Validate Token
                            </DropdownMenuItem>
                            <DropdownMenuItem
                              onClick={() => handleDelete(account.id)}
                              className="text-red-600"
                            >
                              Delete
                            </DropdownMenuItem>
                          </DropdownMenuContent>
                        </DropdownMenu>
                      </TableCell>
                    </TableRow>
                  );
                })}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>

      <Alert>
        <AlertCircle className="h-4 w-4" />
        <AlertDescription>
          <strong>Note:</strong> Projects without an assigned GitHub account will use the global token
          configured in General Settings. When you assign an account to a project, all git operations
          (push, pull, PR creation) will use that account's credentials.
        </AlertDescription>
      </Alert>

      {/* Add/Edit Account Dialog */}
      <Dialog open={isDialogOpen} onOpenChange={handleCloseDialog}>
        <DialogContent className="sm:max-w-[500px]">
          <DialogHeader>
            <DialogTitle>
              {editingAccount ? 'Edit GitHub Account' : 'Add GitHub Account'}
            </DialogTitle>
            <DialogDescription>
              {editingAccount
                ? 'Update the account details. Leave token fields empty to keep existing tokens.'
                : 'Add a new GitHub account for project-specific authentication.'}
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-4 py-4">
            {formError && (
              <Alert variant="destructive">
                <AlertDescription>{formError}</AlertDescription>
              </Alert>
            )}

            <div className="space-y-2">
              <Label htmlFor="username">Username *</Label>
              <Input
                id="username"
                value={formData.username || ''}
                onChange={(e) =>
                  setFormData({ ...formData, username: e.target.value })
                }
                placeholder="octocat"
                disabled={saving}
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="email">Primary Email</Label>
              <Input
                id="email"
                type="email"
                value={formData.primary_email || ''}
                onChange={(e) =>
                  setFormData({
                    ...formData,
                    primary_email: e.target.value || null,
                  })
                }
                placeholder="octocat@github.com"
                disabled={saving}
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="oauth-token">OAuth Token</Label>
              <Input
                id="oauth-token"
                type="password"
                value={formData.oauth_token || ''}
                onChange={(e) =>
                  setFormData({
                    ...formData,
                    oauth_token: e.target.value || null,
                  })
                }
                placeholder="gho_xxxxxxxxxxxxxxxxxxxx"
                disabled={saving}
              />
              <p className="text-sm text-muted-foreground">
                {editingAccount
                  ? 'Leave empty to keep the existing OAuth token'
                  : 'Obtained via GitHub OAuth flow'}
              </p>
            </div>

            <div className="space-y-2">
              <Label htmlFor="pat">Personal Access Token</Label>
              <Input
                id="pat"
                type="password"
                value={formData.pat || ''}
                onChange={(e) =>
                  setFormData({ ...formData, pat: e.target.value || null })
                }
                placeholder="ghp_xxxxxxxxxxxxxxxxxxxx"
                disabled={saving}
              />
              <p className="text-sm text-muted-foreground">
                {editingAccount
                  ? 'Leave empty to keep the existing PAT'
                  : 'Alternative to OAuth. Create at '}
                {!editingAccount && (
                  <a
                    href="https://github.com/settings/tokens"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-blue-600 hover:underline"
                  >
                    github.com/settings/tokens
                  </a>
                )}
              </p>
            </div>
          </div>

          <DialogFooter>
            <Button variant="outline" onClick={handleCloseDialog} disabled={saving}>
              Cancel
            </Button>
            <Button onClick={handleSave} disabled={saving}>
              {saving && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              {editingAccount ? 'Update' : 'Create'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
