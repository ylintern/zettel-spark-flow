import { useEffect, useState } from "react";
import { Lock, ShieldCheck, Eye, EyeOff, Fingerprint } from "lucide-react";
import { isPinSetup, verifyPin, setupPin } from "@/lib/crypto";
import { getAiConfig } from "@/components/OnboardingWizard";
import {
  fallbackPassphraseUnlock,
  getDeviceCapabilities,
  isTauriRuntimeAvailable,
  verifyBiometricUnlock,
  type DeviceCapabilities,
} from "@/lib/commands";

interface LockScreenProps {
  onUnlock: (pin: string) => void;
}

const DEFAULT_DEVICE_CAPABILITIES: DeviceCapabilities = {
  platform: "web",
  isDesktop: true,
  isMobile: false,
  hasSecureEnclave: false,
  hasTouchId: false,
  hasFaceId: false,
  supportsBiometricPrompt: false,
  canOfferBiometrics: false,
  supportsPin: true,
  supportsPassphrase: true,
};

export function LockScreen({ onUnlock }: LockScreenProps) {
  const [authMethod, setAuthMethod] = useState<"biometrics" | "pin" | "passphrase">("pin");
  const [deviceCapabilities, setDeviceCapabilities] = useState<DeviceCapabilities>(DEFAULT_DEVICE_CAPABILITIES);
  const [isSetup, setIsSetup] = useState(false);
  const [secret, setSecret] = useState("");
  const [confirmSecret, setConfirmSecret] = useState("");
  const [step, setStep] = useState<"enter" | "confirm">(isSetup ? "enter" : "enter");
  const [error, setError] = useState("");
  const [showPin, setShowPin] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const usesPassphrase = authMethod === "passphrase" || authMethod === "biometrics";
  const supportsBiometricUnlock =
    authMethod === "biometrics" &&
    isSetup &&
    deviceCapabilities.canOfferBiometrics &&
    deviceCapabilities.isMobile;
  const credentialLabel = usesPassphrase ? "passphrase" : "PIN";
  const createCopy = usesPassphrase
    ? "Create a passphrase to encrypt your notes at rest"
    : "Create a PIN to encrypt your notes at rest";
  const confirmCopy = usesPassphrase ? "Confirm your passphrase" : "Confirm your PIN";
  const unlockCopy = usesPassphrase
    ? "Enter your passphrase to decrypt your notes"
    : "Enter your PIN to decrypt your notes";
  const minLengthError = usesPassphrase
    ? "Passphrase must be at least 8 characters"
    : "PIN must be at least 4 characters";
  const mismatchError = usesPassphrase
    ? "Passphrases don't match. Try again."
    : "PINs don't match. Try again.";
  const placeholder = step === "confirm"
    ? usesPassphrase
      ? "Confirm passphrase..."
      : "Confirm PIN..."
    : usesPassphrase
      ? "Enter passphrase..."
      : "Enter PIN...";

  useEffect(() => {
    try {
      setAuthMethod(getAiConfig().authMethod);
    } catch {
      setAuthMethod("pin");
    }

    void isPinSetup().then(setIsSetup).catch(() => setIsSetup(false));

    if (isTauriRuntimeAvailable()) {
      void getDeviceCapabilities()
        .then(setDeviceCapabilities)
        .catch(() => setDeviceCapabilities(DEFAULT_DEVICE_CAPABILITIES));
    }
  }, []);

  const handleSubmit = async () => {
    setError("");
    setIsSubmitting(true);

    try {
      if (isSetup) {
        const valid = usesPassphrase
          ? await fallbackPassphraseUnlock(secret).then(() => true).catch(() => false)
          : await verifyPin(secret);
        if (valid) {
          onUnlock(secret);
        } else {
          setError(`Incorrect ${credentialLabel}. Try again.`);
          setSecret("");
        }
      } else {
        if (step === "enter") {
          const minLength = usesPassphrase ? 8 : 4;
          if (secret.trim().length < minLength) {
            setError(minLengthError);
            return;
          }
          setStep("confirm");
          setConfirmSecret("");
        } else {
          if (confirmSecret !== secret) {
            setError(mismatchError);
            setConfirmSecret("");
            return;
          }
          await setupPin(secret);
          onUnlock(secret);
        }
      }
    } catch (submissionError) {
      const message = submissionError instanceof Error ? submissionError.message : `Failed to set up ${credentialLabel}`;
      setError(message);
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleBiometricUnlock = async () => {
    setError("");
    setIsSubmitting(true);

    try {
      const unlocked = await verifyBiometricUnlock();
      if (unlocked) {
        onUnlock("");
        return;
      }
      setError("Biometric unlock is not available yet on this build.");
    } catch (unlockError) {
      const message = unlockError instanceof Error ? unlockError.message : "Failed to unlock with biometrics";
      setError(message);
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className="min-h-[100dvh] flex flex-col items-center justify-center bg-background p-6 overflow-y-auto">
      <div className="w-full max-w-xs flex flex-col items-center gap-6">
        {/* Icon */}
        <div className="h-16 w-16 rounded-2xl bg-foreground/10 flex items-center justify-center">
          {isSetup ? (
            <Lock className="h-8 w-8 text-foreground" />
          ) : (
            <ShieldCheck className="h-8 w-8 text-foreground" />
          )}
        </div>

        {/* Title */}
        <div className="text-center">
          <h1 className="text-xl font-bold text-foreground">
            {isSetup ? "Vault Locked" : "Set Up Encryption"}
          </h1>
          <p className="text-sm text-muted-foreground mt-1">
            {isSetup
              ? unlockCopy
              : step === "enter"
              ? createCopy
              : confirmCopy}
          </p>
        </div>

        {/* Credential Input */}
        <div className="w-full space-y-3">
          <div className="relative">
            <input
              type={showPin ? "text" : "password"}
              value={step === "confirm" ? confirmSecret : secret}
              onChange={(e) =>
                step === "confirm"
                  ? setConfirmSecret(e.target.value)
                  : setSecret(e.target.value)
              }
              onKeyDown={(e) => {
                if (e.key === "Enter" && !isSubmitting) {
                  void handleSubmit();
                }
              }}
              placeholder={placeholder}
              className="w-full h-12 rounded-xl bg-muted border border-border px-4 pr-10 text-center text-lg tracking-[0.3em] font-mono text-foreground placeholder:text-muted-foreground placeholder:tracking-normal placeholder:text-sm focus:outline-none focus:ring-2 focus:ring-foreground/20"
              autoFocus
              disabled={isSubmitting}
            />
            <button
              type="button"
              onClick={() => setShowPin(!showPin)}
              className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
              disabled={isSubmitting}
            >
              {showPin ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
            </button>
          </div>

          {error && (
            <p className="text-xs text-destructive text-center">{error}</p>
          )}

          <button
            onClick={() => void handleSubmit()}
            disabled={isSubmitting}
            className="w-full h-11 rounded-xl bg-foreground text-background font-medium text-sm hover:opacity-90 transition-opacity"
          >
            {isSubmitting ? "Working..." : isSetup ? "Unlock" : step === "enter" ? "Next" : "Enable Encryption"}
          </button>

          {supportsBiometricUnlock && (
            <button
              onClick={() => void handleBiometricUnlock()}
              disabled={isSubmitting}
              className="w-full h-11 rounded-xl border border-border text-foreground font-medium text-sm hover:border-foreground/30 transition-colors flex items-center justify-center gap-2"
            >
              <Fingerprint className="h-4 w-4" />
              Unlock with Biometrics
            </button>
          )}
        </div>

        <p className="text-[10px] text-muted-foreground text-center max-w-[200px]">
          {isSetup
            ? "Your secure vault protects private notes and secrets"
            : `Your ${credentialLabel} unlocks the secure vault for private notes and secrets`}
        </p>
      </div>
    </div>
  );
}
