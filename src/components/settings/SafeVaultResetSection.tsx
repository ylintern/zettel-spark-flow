import { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { AlertTriangle, CheckCircle } from 'lucide-react';
import { factoryReset } from '@/lib/commands';

interface ResetState {
  loading: boolean;
  error?: string;
  success?: string;
  showConfirm: boolean;
  confirmCount: number;
}

/**
 * Safe Vault Reset Component
 * 
 * Allows users to completely reset their vault when they've forgotten their passphrase.
 * This is a destructive operation with multiple confirmation steps.
 */
export function SafeVaultResetSection() {
  const [state, setState] = useState<ResetState>({
    loading: false,
    showConfirm: false,
    confirmCount: 0,
  });

  const handleResetClick = () => {
    if (!state.showConfirm) {
      setState(prev => ({ ...prev, showConfirm: true, confirmCount: 0 }));
    }
  };

  const handleConfirmReset = async () => {
    // Require multiple confirmations
    if (state.confirmCount < 1) {
      setState(prev => ({ ...prev, confirmCount: prev.confirmCount + 1 }));
      return;
    }

    setState(prev => ({ ...prev, loading: true, error: undefined }));

    try {
      await factoryReset();

      // Clear all known localStorage keys to fully reset all storage layers
      const VIBO_LEGACY_KEYS = [
        // Legacy zettel-* namespace (older installs)
        "zettel-notes",
        "zettel-columns",
        "zettel-encrypted-notes",
        "zettel-agent-notes",
        // Current vibo-* namespace
        "vibo-notes",
        "vibo-columns",
        "vibo-folders",
        "vibo-pin",
        "vibo-salt",
        "vibo-agent-notes",
      ];
      VIBO_LEGACY_KEYS.forEach((key) => localStorage.removeItem(key));

      // Force full app reload to reinitialize all state
      window.location.reload();
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : 'Failed to reset vault';
      setState(prev => ({
        ...prev,
        loading: false,
        error: errorMsg,
      }));
    }
  };

  const handleCancel = () => {
    setState({
      loading: false,
      showConfirm: false,
      confirmCount: 0,
    });
  };

  return (
    <Card className="border-red-200">
      <CardHeader>
        <CardTitle className="flex items-center gap-2 text-red-700">
          <AlertTriangle className="w-5 h-5" />
          Vault Reset
        </CardTitle>
        <CardDescription>
          Completely remove your vault and all stored secrets
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        {/* Error Alert */}
        {state.error && (
          <Alert variant="destructive">
            <AlertTriangle className="h-4 w-4" />
            <AlertDescription>{state.error}</AlertDescription>
          </Alert>
        )}

        {/* Success Alert */}
        {state.success && (
          <Alert className="border-green-200 bg-green-50">
            <CheckCircle className="h-4 w-4 text-green-600" />
            <AlertDescription className="text-green-800">{state.success}</AlertDescription>
          </Alert>
        )}

        {/* Warning Info */}
        {!state.showConfirm && (
          <div className="bg-red-50 border border-red-200 rounded-md p-4">
            <h4 className="font-semibold text-red-900 mb-2">⚠️ Warning: This cannot be undone</h4>
            <ul className="text-sm text-red-800 space-y-1 ml-4 list-disc">
              <li>All secrets stored in the vault will be permanently deleted</li>
              <li>API keys, passwords, and encryption keys will be lost</li>
              <li>Private notes will become unreadable</li>
              <li>You will need to set up a new vault with a new passphrase</li>
            </ul>
          </div>
        )}

        {/* Confirmation Steps */}
        {state.showConfirm && (
          <div className="space-y-3 border-t pt-4">
            <p className="text-sm font-medium text-foreground">
              Are you absolutely sure? This action cannot be reversed.
            </p>
            
            {state.confirmCount === 0 && (
              <p className="text-xs text-muted-foreground">
                Click "Yes, reset my vault" to confirm.
              </p>
            )}
            
            {state.confirmCount === 1 && (
              <p className="text-xs text-red-600 font-medium">
                ⚠️ Click "Permanently Delete" one more time to confirm.
              </p>
            )}

            <div className="flex gap-2">
              <Button
                onClick={handleConfirmReset}
                disabled={state.loading}
                variant="destructive"
                className="flex-1"
              >
                {state.confirmCount === 0 ? 'Yes, reset my vault' : 'Permanently Delete'}
              </Button>
              <Button
                onClick={handleCancel}
                variant="outline"
                disabled={state.loading}
                className="flex-1"
              >
                Cancel
              </Button>
            </div>
          </div>
        )}

        {/* Reset Button (when not confirming) */}
        {!state.showConfirm && (
          <Button
            onClick={handleResetClick}
            variant="destructive"
            className="w-full"
          >
            Reset Vault
          </Button>
        )}

        {/* Info Box */}
        <div className="bg-blue-50 border border-blue-200 rounded-md p-3">
          <p className="text-sm text-blue-900">
            <strong>Why reset?</strong> If you've forgotten your passphrase and cannot unlock your vault, resetting will allow you to start fresh with a new passphrase. All existing data will be permanently deleted.
          </p>
        </div>
      </CardContent>
    </Card>
  );
}
