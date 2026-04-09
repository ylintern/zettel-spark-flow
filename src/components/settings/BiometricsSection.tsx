import { useEffect, useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Switch } from '@/components/ui/switch';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Label } from '@/components/ui/label';
import { Fingerprint, AlertCircle, CheckCircle } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';

interface BiometricStatus {
  available: boolean;
  enabled: boolean;
  loading: boolean;
  error?: string;
  success?: string;
}

/**
 * P0-06: Mobile Biometrics Component
 * 
 * Allows users to enable/disable FaceID/Fingerprint unlock on iOS/Android
 * Falls back to passphrase unlock if biometric is unavailable or fails
 */
export function BiometricsSection() {
  const [status, setStatus] = useState<BiometricStatus>({
    available: false,
    enabled: false,
    loading: true,
  });
  
  const [passphrase, setPassphrase] = useState('');
  const [showPassphraseInput, setShowPassphraseInput] = useState(false);

  // Check if device supports biometric on component mount
  useEffect(() => {
    const checkBiometricSupport = async () => {
      try {
        const [available, enabled] = await Promise.all([
          invoke<boolean>('is_biometric_available'),
          invoke<boolean>('is_biometric_enabled'),
        ]);

        setStatus({
          available,
          enabled,
          loading: false,
        });
      } catch (error) {
        setStatus({
          available: false,
          enabled: false,
          loading: false,
          error: 'Failed to check biometric support',
        });
      }
    };

    checkBiometricSupport();
  }, []);

  const handleToggleBiometric = async (enabled: boolean) => {
    setStatus({ ...status, loading: true, error: undefined, success: undefined });

    try {
      if (enabled) {
        // User wants to ENABLE biometric
        // Step 1: Get passphrase from user
        if (!passphrase) {
          setShowPassphraseInput(true);
          setStatus({ ...status, loading: false });
          return;
        }

        await invoke('enable_biometric_unlock', {
          encryptedPassphrase: passphrase,
        });

        setStatus({
          available: true,
          enabled: true,
          loading: false,
          success: 'Biometric unlock enabled. You can now unlock with FaceID/Fingerprint.',
        });
        
        setPassphrase('');
        setShowPassphraseInput(false);
      } else {
        // User wants to DISABLE biometric
        await invoke('disable_biometric_unlock');

        setStatus({
          available: true,
          enabled: false,
          loading: false,
          success: 'Biometric unlock disabled.',
        });
      }
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : 'Failed to update biometric settings';
      setStatus({
        available: status.available,
        enabled: status.enabled,
        loading: false,
        error: errorMsg,
      });
    }
  };

  const handleConfirmPassphrase = () => {
    if (passphrase.trim()) {
      handleToggleBiometric(true);
    }
  };

  // Only show on mobile
  if (!status.available) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Fingerprint className="w-5 h-5" />
            Biometric Unlock
          </CardTitle>
          <CardDescription>
            Not available on your device
          </CardDescription>
        </CardHeader>
        <CardContent>
          <p className="text-sm text-gray-500">
            Biometric authentication (FaceID, Fingerprint) is only available on iOS and Android devices.
          </p>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Fingerprint className="w-5 h-5" />
          Biometric Unlock
        </CardTitle>
        <CardDescription>
          Unlock your vault with FaceID or Fingerprint instead of your passphrase (mobile only)
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        {/* Status Alert */}
        {status.error && (
          <Alert variant="destructive">
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>{status.error}</AlertDescription>
          </Alert>
        )}

        {status.success && (
          <Alert className="border-green-200 bg-green-50">
            <CheckCircle className="h-4 w-4 text-green-600" />
            <AlertDescription className="text-green-800">{status.success}</AlertDescription>
          </Alert>
        )}

        {/* Toggle */}
        <div className="flex items-center justify-between">
          <Label htmlFor="biometric-toggle">Enable Biometric Unlock</Label>
          <Switch
            id="biometric-toggle"
            checked={status.enabled}
            onCheckedChange={handleToggleBiometric}
            disabled={status.loading}
          />
        </div>

        {/* Passphrase Input (shown when enabling) */}
        {showPassphraseInput && (
          <div className="space-y-3 border-t pt-4">
            <Label htmlFor="passphrase-input">
              Enter your vault passphrase to enable biometric unlock
            </Label>
            <input
              id="passphrase-input"
              type="password"
              placeholder="Your vault passphrase"
              value={passphrase}
              onChange={(e) => setPassphrase(e.target.value)}
              className="w-full px-3 py-2 border rounded-md border-gray-300 focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
            <div className="flex gap-2">
              <Button
                onClick={handleConfirmPassphrase}
                disabled={!passphrase || status.loading}
                className="flex-1"
              >
                Enable
              </Button>
              <Button
                onClick={() => {
                  setShowPassphraseInput(false);
                  setPassphrase('');
                  setStatus({ ...status, error: undefined });
                }}
                variant="outline"
                className="flex-1"
              >
                Cancel
              </Button>
            </div>
          </div>
        )}

        {/* Info */}
        <div className="bg-blue-50 border border-blue-200 rounded-md p-3">
          <p className="text-sm text-blue-900">
            <strong>Hardening note:</strong> biometric unlock will only be enabled when the native layer can prove a hardware-backed key release path. Manual passphrase unlock remains the trusted fallback.
          </p>
        </div>

        {/* Status indicator */}
        {status.enabled && (
          <div className="flex items-center gap-2 text-green-700">
            <CheckCircle className="w-4 h-4" />
            <span className="text-sm font-medium">Biometric unlock is active</span>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
