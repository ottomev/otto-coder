import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Shield, CheckCircle, XCircle, Settings } from 'lucide-react';
import { useUserSystem } from '@/components/config-provider';
import NiceModal, { useModal } from '@ebay/nice-modal-react';

const PrivacyOptInDialog = NiceModal.create(() => {
  const modal = useModal();
  const { config } = useUserSystem();

  // Check if user is authenticated with GitHub
  const isGitHubAuthenticated =
    config?.github?.username && config?.github?.oauth_token;

  const handleOptIn = () => {
    modal.resolve(true);
  };

  const handleOptOut = () => {
    modal.resolve(false);
  };

  return (
    <Dialog open={modal.visible} uncloseable={true}>
      <DialogContent className="sm:max-w-[700px]">
        <DialogHeader>
          <div className="flex items-center gap-3">
            <Shield className="h-6 w-6 text-primary text-primary-foreground" />
            <DialogTitle>Feedback</DialogTitle>
          </div>
          <DialogDescription className="text-left pt-1">
            Help us improve Otto Coder by sharing usage data and allowing us to
            contact you if needed.
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-3">
          <h2>What data do we collect?</h2>
          <div>
            {isGitHubAuthenticated && (
              <div className="flex items-start gap-2">
                <CheckCircle className="h-4 w-4 text-green-500 mt-0.5 flex-shrink-0" />
                <div className="min-w-0">
                  <p className="text-sm font-medium">
                    GitHub profile information
                  </p>
                  <p className="text-xs text-muted-foreground">
                    Username and email address to send you only very important
                    updates about the project. We promise not to abuse this
                  </p>
                </div>
              </div>
            )}
            <div className="flex items-start gap-2">
              <CheckCircle className="h-4 w-4 text-green-500 mt-0.5 flex-shrink-0" />
              <div className="min-w-0">
                <p className="text-sm font-medium">High-level usage metrics</p>
                <p className="text-xs text-muted-foreground">
                  Number of tasks created, projects managed, feature usage
                </p>
              </div>
            </div>
            <div className="flex items-start gap-2">
              <CheckCircle className="h-4 w-4 text-green-500 mt-0.5 flex-shrink-0" />
              <div className="min-w-0">
                <p className="text-sm font-medium">
                  Performance and error data
                </p>
                <p className="text-xs text-muted-foreground">
                  Application crashes, response times, technical issues
                </p>
              </div>
            </div>
            <div className="flex items-start gap-2">
              <XCircle className="h-4 w-4 text-destructive mt-0.5 flex-shrink-0" />
              <div className="min-w-0">
                <p className="text-sm font-medium">We do NOT collect</p>
                <p className="text-xs text-muted-foreground">
                  Task contents, code snippets, project names, or other personal
                  data
                </p>
              </div>
            </div>
          </div>

          <div className="flex items-center gap-2 text-xs text-muted-foreground bg-muted/50 p-2 rounded-lg">
            <Settings className="h-3 w-3 flex-shrink-0" />
            <span>
              This helps us prioritize improvements. You can change this
              preference anytime in Settings.
            </span>
          </div>
        </div>

        <DialogFooter className="gap-3 flex-col sm:flex-row pt-2">
          <Button variant="outline" onClick={handleOptOut} className="flex-1">
            <XCircle className="h-4 w-4 mr-2" />
            No thanks
          </Button>
          <Button onClick={handleOptIn} className="flex-1">
            <CheckCircle className="h-4 w-4 mr-2" />
            Yes, help improve Otto Coder
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
});

export { PrivacyOptInDialog };
