import {
  Check,
  FolderOpen,
  KeyRound,
  Save,
  ServerCog,
  ShieldCheck,
  Trash2
} from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import {
  tauriDesktopConfigClient,
  type AiProviderConfig,
  type CaptureConfig,
  type DesktopConfig,
  type DesktopConfigClient,
  type DesktopConfigView,
  type PrivacyConfig,
  type ProviderPreset,
  type ProviderTestResult,
  type SecretStatus,
  type StorageConfig,
  type SystemConfig
} from "./lib/desktopConfig";

type SettingsViewProps = {
  client?: DesktopConfigClient;
};

export function SettingsView({
  client = tauriDesktopConfigClient
}: SettingsViewProps) {
  const [view, setView] = useState<DesktopConfigView | null>(null);
  const [draft, setDraft] = useState<DesktopConfig | null>(null);
  const [secretStatus, setSecretStatus] = useState<SecretStatus | null>(null);
  const [apiKey, setApiKey] = useState("");
  const [providerTest, setProviderTest] = useState<ProviderTestResult | null>(null);
  const [status, setStatus] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    let active = true;
    setLoading(true);
    client
      .getConfig()
      .then((loaded) => {
        if (!active) return;
        setView(loaded);
        setDraft(loaded.config);
        setSecretStatus(loaded.aiSecretStatus);
        setError(null);
      })
      .catch((loadError) => {
        if (!active) return;
        setError(errorMessage(loadError));
      })
      .finally(() => {
        if (active) setLoading(false);
      });
    return () => {
      active = false;
    };
  }, [client]);

  const externalAiEnabled = draft?.ai.enabled ?? false;
  const configPath = useMemo(() => view?.configPath ?? "Unavailable", [view]);
  const isFirstRun = view?.firstRun ?? false;

  if (loading) {
    return (
      <section className="settingsView workspace" aria-label="Settings">
        <div className="panel settingsPanel">
          <div className="panelHeader">
            <ServerCog aria-hidden="true" size={20} />
            <h2>Settings</h2>
          </div>
          <p className="monitorConnecting">Loading settings...</p>
        </div>
      </section>
    );
  }

  if (!draft) {
    return (
      <section className="settingsView workspace" aria-label="Settings">
        <div className="panel settingsPanel">
          <div className="panelHeader">
            <ServerCog aria-hidden="true" size={20} />
            <h2>Settings</h2>
          </div>
          <p className="errors" role="status">
            {error ?? "Desktop settings are unavailable in this preview."}
          </p>
        </div>
      </section>
    );
  }

  async function saveSettings() {
    if (!draft) return;
    setSaving(true);
    setStatus(null);
    setError(null);
    try {
      const saved = await client.saveConfig(draft);
      setView(saved);
      setDraft(saved.config);
      setSecretStatus(saved.aiSecretStatus);
      setStatus("Settings saved.");
    } catch (saveError) {
      setError(errorMessage(saveError));
    } finally {
      setSaving(false);
    }
  }

  async function saveApiKey() {
    const nextKey = apiKey.trim();
    if (!nextKey) {
      setError("API key cannot be empty.");
      return;
    }
    setSaving(true);
    setStatus(null);
    setError(null);
    try {
      const nextStatus = await client.setApiKey(nextKey);
      setSecretStatus(nextStatus);
      setApiKey("");
      setStatus("API key saved.");
    } catch (secretError) {
      setError(errorMessage(secretError));
    } finally {
      setSaving(false);
    }
  }

  async function clearApiKey() {
    setSaving(true);
    setStatus(null);
    setError(null);
    try {
      const nextStatus = await client.clearApiKey();
      setSecretStatus(nextStatus);
      setStatus("API key removed.");
    } catch (secretError) {
      setError(errorMessage(secretError));
    } finally {
      setSaving(false);
    }
  }

  async function testProvider() {
    setSaving(true);
    setStatus(null);
    setError(null);
    setProviderTest(null);
    try {
      const result = await client.testProvider();
      setProviderTest(result);
      setStatus("Provider test ready.");
    } catch (testError) {
      setError(errorMessage(testError));
    } finally {
      setSaving(false);
    }
  }

  async function chooseDataDirectory() {
    setSaving(true);
    setStatus(null);
    setError(null);
    try {
      const selected = await client.chooseDataDirectory();
      if (selected) {
        applyDataDirectory(selected);
      }
    } catch (directoryError) {
      setError(errorMessage(directoryError));
    } finally {
      setSaving(false);
    }
  }

  return (
    <section className="settingsView workspace" aria-label="Settings">
      <div className="settingsHeader">
        <div>
          <p className="eyebrow">Desktop configuration</p>
          <h2>{isFirstRun ? "First-run setup" : "Settings"}</h2>
          <p className="settingsPath">{configPath}</p>
        </div>
        <button
          type="button"
          className="iconButton primaryAction"
          disabled={saving}
          onClick={() => void saveSettings()}
        >
          <Save aria-hidden="true" size={18} />
          <span>{isFirstRun ? "Complete Setup" : "Save Settings"}</span>
        </button>
      </div>

      {status && (
        <p className="settingsStatus" role="status">
          <Check aria-hidden="true" size={16} />
          {status}
        </p>
      )}
      {error && (
        <p className="errors" role="status">
          {error}
        </p>
      )}

      <div className="settingsGrid">
        <section className="panel settingsPanel" aria-label="Storage">
          <div className="panelHeader">
            <ServerCog aria-hidden="true" size={20} />
            <h3>Storage</h3>
          </div>
          <Field
            label="Data Directory"
            value={draft.storage.dataDir}
            onChange={(value) => updateStorage({ dataDir: value })}
          />
          <button
            type="button"
            className="iconButton secondaryAction"
            disabled={saving}
            onClick={() => void chooseDataDirectory()}
          >
            <FolderOpen aria-hidden="true" size={16} />
            <span>Choose Data Directory</span>
          </button>
          <Field
            label="Database Path"
            value={draft.storage.databasePath}
            onChange={(value) => updateStorage({ databasePath: value })}
          />
          <Field
            label="Screenshot Directory"
            value={draft.storage.screenshotDir}
            onChange={(value) => updateStorage({ screenshotDir: value })}
          />
          <NumberField
            label="Retention Days"
            value={draft.storage.retentionDays}
            min={1}
            onChange={(value) => updateStorage({ retentionDays: value })}
          />
        </section>

        <section className="panel settingsPanel" aria-label="Capture">
          <div className="panelHeader">
            <ShieldCheck aria-hidden="true" size={20} />
            <h3>Capture</h3>
          </div>
          <NumberField
            label="Poll ms"
            value={draft.capture.pollMs}
            min={100}
            onChange={(value) => updateCapture({ pollMs: value })}
          />
          <NumberField
            label="Screenshot Interval secs"
            value={draft.capture.screenshotIntervalSecs}
            min={10}
            onChange={(value) =>
              updateCapture({ screenshotIntervalSecs: value })
            }
          />
          <NumberField
            label="API Port"
            value={draft.system.apiPort}
            min={1}
            onChange={(value) => updateSystem({ apiPort: value })}
          />
          <CheckboxField
            label="Input capture"
            checked={draft.capture.inputCaptureEnabled}
            onChange={(checked) =>
              updateCapture({ inputCaptureEnabled: checked })
            }
          />
          <CheckboxField
            label="High-res capture"
            checked={draft.capture.highResCaptureEnabled}
            onChange={(checked) =>
              updateCapture({ highResCaptureEnabled: checked })
            }
          />
        </section>

        <section className="panel settingsPanel aiSettingsPanel" aria-label="AI Provider">
          <div className="panelHeader">
            <KeyRound aria-hidden="true" size={20} />
            <h3>AI Provider</h3>
            <span className={`healthPill ${externalAiEnabled ? "running" : "not_started"}`}>
              {externalAiEnabled ? "External AI" : "Local Only"}
            </span>
          </div>
          <CheckboxField
            label="Enable external AI"
            checked={draft.ai.enabled}
            onChange={(checked) => updateAi({ enabled: checked })}
          />
          {(externalAiEnabled || isFirstRun) && (
            <div className="retentionNotice externalAiNotice">
              <ShieldCheck aria-hidden="true" size={16} />
              <p>External AI can send text summaries to the configured provider.</p>
            </div>
          )}
          <CheckboxField
            label="External AI warning accepted"
            checked={draft.privacy.externalAiWarningAccepted}
            onChange={(checked) =>
              updatePrivacy({ externalAiWarningAccepted: checked })
            }
          />
          <label className="settingsField">
            <span>Provider Preset</span>
            <select
              value={draft.ai.providerPreset}
              onChange={(event) =>
                updateAi({ providerPreset: event.currentTarget.value as ProviderPreset })
              }
            >
              <option value="customOpenAiCompatible">Custom</option>
              <option value="miniMax">MiniMax</option>
              <option value="openAi">OpenAI</option>
            </select>
          </label>
          <Field
            label="Base URL"
            value={draft.ai.baseUrl}
            onChange={(value) => updateAi({ baseUrl: value })}
          />
          <Field
            label="Model"
            value={draft.ai.model}
            onChange={(value) => updateAi({ model: value })}
          />
          <NumberField
            label="Max Completion Tokens"
            value={draft.ai.maxCompletionTokens}
            min={1}
            onChange={(value) => updateAi({ maxCompletionTokens: value })}
          />
          <CheckboxField
            label="Visual analysis"
            checked={draft.ai.pipelines.visualAnalysis}
            onChange={(checked) =>
              updateAi({
                pipelines: { ...draft.ai.pipelines, visualAnalysis: checked }
              })
            }
          />
          <div className="secretRow">
            <span>API Key</span>
            <strong>{secretStatus?.present ? secretStatus.masked : "Not set"}</strong>
          </div>
          <label className="settingsField">
            <span>API key</span>
            <input
              type="password"
              value={apiKey}
              autoComplete="new-password"
              onChange={(event) => setApiKey(event.currentTarget.value)}
            />
          </label>
          <div className="inlineActions">
            <button type="button" disabled={saving} onClick={() => void saveApiKey()}>
              <KeyRound aria-hidden="true" size={16} />
              <span>Save API Key</span>
            </button>
            <button type="button" disabled={saving} onClick={() => void clearApiKey()}>
              <Trash2 aria-hidden="true" size={16} />
              <span>Remove API Key</span>
            </button>
            <button type="button" disabled={saving} onClick={() => void testProvider()}>
              <ServerCog aria-hidden="true" size={16} />
              <span>Test Provider</span>
            </button>
          </div>
          {providerTest && (
            <dl className="providerTestResult">
              <div>
                <dt>Status</dt>
                <dd>{providerTest.status}</dd>
              </div>
              <div>
                <dt>Request</dt>
                <dd>{providerTest.requestKind}</dd>
              </div>
              <div>
                <dt>Screenshots</dt>
                <dd>Screenshots: {providerTest.usesScreenshots ? "yes" : "no"}</dd>
              </div>
              {providerTest.endpoint && (
                <div>
                  <dt>Endpoint</dt>
                  <dd>{providerTest.endpoint}</dd>
                </div>
              )}
            </dl>
          )}
        </section>
      </div>
    </section>
  );

  function updateStorage(update: Partial<StorageConfig>) {
    setDraft((current) =>
      current ? { ...current, storage: { ...current.storage, ...update } } : current
    );
  }

  function updateCapture(update: Partial<CaptureConfig>) {
    setDraft((current) =>
      current ? { ...current, capture: { ...current.capture, ...update } } : current
    );
  }

  function updatePrivacy(update: Partial<PrivacyConfig>) {
    setDraft((current) =>
      current ? { ...current, privacy: { ...current.privacy, ...update } } : current
    );
  }

  function updateAi(update: Partial<AiProviderConfig>) {
    setDraft((current) =>
      current ? { ...current, ai: { ...current.ai, ...update } } : current
    );
  }

  function updateSystem(update: Partial<SystemConfig>) {
    setDraft((current) =>
      current ? { ...current, system: { ...current.system, ...update } } : current
    );
  }

  function applyDataDirectory(dataDir: string) {
    const root = trimTrailingSeparators(dataDir);
    setDraft((current) =>
      current
        ? {
            ...current,
            storage: {
              ...current.storage,
              dataDir: root,
              databasePath: joinPath(root, "local.sqlite3"),
              screenshotDir: joinPath(root, "screenshots"),
              highResScreenshotDir: joinPath(root, "high-res-screenshots")
            },
            privacy: {
              ...current.privacy,
              blockerConfigPath: joinPath(root, "blocker_config.json")
            }
          }
        : current
    );
  }
}

function joinPath(root: string, child: string): string {
  const separator = root.includes("\\") && !root.includes("/") ? "\\" : "/";
  return `${root}${separator}${child}`;
}

function trimTrailingSeparators(path: string): string {
  return path.replace(/[\\/]+$/, "");
}

function Field({
  label,
  value,
  onChange
}: {
  label: string;
  value: string;
  onChange: (value: string) => void;
}) {
  return (
    <label className="settingsField">
      <span>{label}</span>
      <input
        type="text"
        value={value}
        onChange={(event) => onChange(event.currentTarget.value)}
      />
    </label>
  );
}

function NumberField({
  label,
  value,
  min,
  onChange
}: {
  label: string;
  value: number;
  min: number;
  onChange: (value: number) => void;
}) {
  return (
    <label className="settingsField">
      <span>{label}</span>
      <input
        type="number"
        min={min}
        value={value}
        onChange={(event) => onChange(Number(event.currentTarget.value))}
      />
    </label>
  );
}

function CheckboxField({
  label,
  checked,
  onChange
}: {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}) {
  return (
    <label className="settingsCheckbox">
      <input
        type="checkbox"
        checked={checked}
        onChange={(event) => onChange(event.currentTarget.checked)}
      />
      <span>{label}</span>
    </label>
  );
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
