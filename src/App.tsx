import {
  Activity,
  AlertTriangle,
  CheckCircle2,
  ChevronRight,
  Monitor,
  RefreshCw,
  RotateCcw,
  Save,
  Settings2,
  SquareStack,
  TriangleAlert,
} from 'lucide-react';
import {
  type ReactNode,
  useCallback,
  useEffect,
  useMemo,
  useState,
} from 'react';
import {
  hideMainWindow,
  loadAppState,
  resetSettings,
  saveSettings,
  showMainWindow,
  toggleMainWindow,
} from './lib/api';
import {
  type AppSettings,
  type AppState,
  DEFAULT_APP_STATE,
  DEFAULT_SETTINGS,
  LANGUAGE_OPTIONS,
  OUTPUT_SPEED_OPTIONS,
} from './lib/types';

type BannerTone = 'neutral' | 'warning' | 'error';

const panelClassName =
  'rounded-lg border border-slate-200 bg-white/95 p-5 shadow-sm shadow-slate-200/60';

// Owns the stage-1 UI state bridge between React and the Tauri backend snapshot.
function App() {
  const [state, setState] = useState<AppState>(DEFAULT_APP_STATE);
  const [draft, setDraft] = useState<AppSettings>(DEFAULT_SETTINGS);
  const [isBusy, setIsBusy] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [notice, setNotice] = useState<{
    tone: BannerTone;
    text: string;
  } | null>(null);

  // Reloads backend state and is the first place to inspect if startup hydration fails.
  const hydrate = useCallback(async () => {
    setIsBusy(true);
    setNotice(null);

    try {
      const snapshot = await loadAppState();
      setState(snapshot);
      setDraft(snapshot.settings);
      if (snapshot.startupWarning) {
        setNotice({ tone: 'warning', text: snapshot.startupWarning });
      }
    } catch (error) {
      const fallbackMessage =
        error instanceof Error
          ? error.message
          : 'The desktop backend did not respond.';
      setNotice({
        tone: 'error',
        text: fallbackMessage,
      });
      setState({
        ...DEFAULT_APP_STATE,
        status: 'error',
        startupWarning: fallbackMessage,
      });
      setDraft(DEFAULT_SETTINGS);
    } finally {
      setIsBusy(false);
    }
  }, []);

  useEffect(() => {
    void hydrate();
  }, [hydrate]);

  const isDirty = useMemo(
    () => JSON.stringify(draft) !== JSON.stringify(state.settings),
    [draft, state.settings],
  );

  // Keeps settings form changes typed to the AppSettings contract.
  const updateField = useCallback(
    <K extends keyof AppSettings>(key: K, value: AppSettings[K]) => {
      setDraft((current) => ({
        ...current,
        [key]: value,
      }));
    },
    [],
  );

  // Persists the draft settings and re-syncs the UI from the backend snapshot.
  const persist = useCallback(async () => {
    setIsSaving(true);
    setNotice(null);

    try {
      const snapshot = await saveSettings(draft);
      setState(snapshot);
      setDraft(snapshot.settings);
      setNotice({
        tone: 'neutral',
        text: 'Settings saved to the local config store.',
      });
    } catch (error) {
      const message =
        error instanceof Error ? error.message : 'Failed to save settings.';
      setNotice({ tone: 'error', text: message });
    } finally {
      setIsSaving(false);
    }
  }, [draft]);

  // Resets both backend settings and the local draft to the default profile.
  const handleReset = useCallback(async () => {
    setIsSaving(true);
    setNotice(null);

    try {
      const snapshot = await resetSettings();
      setState(snapshot);
      setDraft(snapshot.settings);
      setNotice({
        tone: 'neutral',
        text: 'Settings were reset to the default local profile.',
      });
    } catch (error) {
      const message =
        error instanceof Error ? error.message : 'Failed to reset settings.';
      setNotice({ tone: 'error', text: message });
    } finally {
      setIsSaving(false);
    }
  }, []);

  // Centralizes window visibility commands so tray and header behavior stay comparable.
  const handleWindowAction = useCallback(
    async (action: 'show' | 'hide' | 'toggle') => {
      setNotice(null);

      try {
        const snapshot =
          action === 'show'
            ? await showMainWindow()
            : action === 'hide'
              ? await hideMainWindow()
              : await toggleMainWindow();
        setState(snapshot);
        setDraft(snapshot.settings);
      } catch (error) {
        const message =
          error instanceof Error ? error.message : 'Window action failed.';
        setNotice({ tone: 'error', text: message });
      }
    },
    [],
  );

  const currentSpeedLabel =
    OUTPUT_SPEED_OPTIONS.find((option) => option.value === draft.outputSpeed)
      ?.label ?? 'Normal';

  return (
    <div className="min-h-screen bg-slate-50 text-slate-900">
      <main className="mx-auto flex min-h-screen w-full max-w-7xl flex-col gap-5 px-4 py-4 sm:px-6 lg:px-8">
        <header className={panelClassName}>
          <div className="flex flex-col gap-4 xl:flex-row xl:items-start xl:justify-between">
            <div className="space-y-2">
              <div className="flex items-center gap-2 text-xs font-medium uppercase tracking-[0.28em] text-slate-500">
                <SquareStack className="h-4 w-4" />
                Question Scan
              </div>
              <div className="space-y-1">
                <h1 className="text-2xl font-semibold text-slate-950">
                  Desktop shell
                </h1>
                <p className="max-w-3xl text-sm leading-6 text-slate-600">
                  Tray-backed workspace, local settings storage, and an answer
                  panel scaffold for the rest of the MVP.
                </p>
              </div>
            </div>

            <div className="flex flex-wrap items-center gap-2">
              <StatusPill tone={state.status}>
                {state.status === 'loading'
                  ? 'Loading'
                  : state.status === 'warning'
                    ? 'Warning'
                    : state.status === 'error'
                      ? 'Error'
                      : 'Ready'}
              </StatusPill>
              <ActionButton
                icon={<Monitor className="h-4 w-4" />}
                label={state.windowVisible ? 'Hide window' : 'Show window'}
                onClick={() =>
                  void handleWindowAction(state.windowVisible ? 'hide' : 'show')
                }
              />
              <ActionButton
                icon={<RefreshCw className="h-4 w-4" />}
                label="Reload"
                onClick={() => void hydrate()}
                disabled={isBusy}
              />
              <ActionButton
                icon={<RotateCcw className="h-4 w-4" />}
                label="Reset"
                onClick={() => void handleReset()}
                disabled={isSaving}
              />
              <ActionButton
                icon={<Save className="h-4 w-4" />}
                label={isSaving ? 'Saving' : 'Save'}
                onClick={persist}
                disabled={!isDirty || isSaving}
                emphasis="primary"
              />
            </div>
          </div>

          <div className="mt-4 grid gap-3 md:grid-cols-3">
            <InfoTile
              title="Window"
              value={state.windowVisible ? 'Visible' : 'Hidden'}
            />
            <InfoTile title="Tray" value={state.trayStatus} />
            <InfoTile title="Version" value={state.version} />
          </div>
        </header>

        {notice ? (
          <NoticeBanner tone={notice.tone}>
            {notice.tone === 'warning' ? (
              <TriangleAlert className="mt-0.5 h-4 w-4 shrink-0" />
            ) : notice.tone === 'error' ? (
              <AlertTriangle className="mt-0.5 h-4 w-4 shrink-0" />
            ) : (
              <CheckCircle2 className="mt-0.5 h-4 w-4 shrink-0" />
            )}
            <span>{notice.text}</span>
          </NoticeBanner>
        ) : null}

        <div className="grid gap-5 xl:grid-cols-[minmax(0,1.08fr)_minmax(360px,0.92fr)]">
          <section className={panelClassName}>
            <SectionTitle
              icon={<Settings2 className="h-4 w-4" />}
              title="Settings"
              subtitle="Local defaults, provider placeholders, and the shell-level options that later stages will expand."
            />

            <form
              className="mt-5 grid gap-4"
              onSubmit={(event) => {
                event.preventDefault();
                void persist();
              }}
            >
              <div className="grid gap-4 lg:grid-cols-2">
                <Field label="Provider base URL" labelFor="provider-base-url">
                  <input
                    id="provider-base-url"
                    className="field-input"
                    type="url"
                    value={draft.providerBaseUrl}
                    onChange={(event) =>
                      updateField('providerBaseUrl', event.target.value)
                    }
                    placeholder="https://api.openai.com/v1"
                  />
                </Field>

                <Field label="Model" labelFor="provider-model">
                  <input
                    id="provider-model"
                    className="field-input"
                    value={draft.providerModel}
                    onChange={(event) =>
                      updateField('providerModel', event.target.value)
                    }
                    placeholder="gpt-4o-mini"
                  />
                </Field>

                <Field label="Default language" labelFor="default-language">
                  <select
                    id="default-language"
                    className="field-input"
                    value={draft.defaultLanguage}
                    onChange={(event) =>
                      updateField(
                        'defaultLanguage',
                        event.target.value as AppSettings['defaultLanguage'],
                      )
                    }
                  >
                    {LANGUAGE_OPTIONS.map((option) => (
                      <option key={option.value} value={option.value}>
                        {option.label}
                      </option>
                    ))}
                  </select>
                </Field>

                <Field label="Global shortcut" labelFor="global-shortcut">
                  <input
                    id="global-shortcut"
                    className="field-input"
                    value={draft.globalShortcut}
                    onChange={(event) =>
                      updateField('globalShortcut', event.target.value)
                    }
                    placeholder="Ctrl+Shift+Q"
                  />
                </Field>
              </div>

              <div className="grid gap-4 lg:grid-cols-[minmax(0,1fr)_minmax(260px,320px)]">
                <Field label="Output speed">
                  <div className="grid grid-cols-2 gap-2 sm:grid-cols-4">
                    {OUTPUT_SPEED_OPTIONS.map((option) => (
                      <button
                        key={option.value}
                        type="button"
                        className={`speed-chip ${draft.outputSpeed === option.value ? 'speed-chip-active' : ''}`}
                        onClick={() => updateField('outputSpeed', option.value)}
                      >
                        {option.label}
                      </button>
                    ))}
                  </div>
                  {draft.outputSpeed === 'custom' ? (
                    <div className="mt-3">
                      <label
                        className="mb-2 block text-xs font-medium uppercase tracking-[0.2em] text-slate-500"
                        htmlFor="custom-characters-per-second"
                      >
                        Characters per second
                      </label>
                      <input
                        id="custom-characters-per-second"
                        className="field-input"
                        type="number"
                        min={1}
                        max={240}
                        value={draft.customCharactersPerSecond}
                        onChange={(event) =>
                          updateField(
                            'customCharactersPerSecond',
                            Number(event.target.value || 0),
                          )
                        }
                      />
                    </div>
                  ) : null}
                </Field>

                <Field label="Presentation">
                  <div className="grid gap-3">
                    <ToggleRow
                      label="Save history"
                      description="Store the local answer snapshot for later review."
                      checked={draft.saveHistory}
                      onChange={(checked) =>
                        updateField('saveHistory', checked)
                      }
                    />
                    <ToggleRow
                      label="Launch to tray"
                      description="Hide the window on launch and keep the app in the tray."
                      checked={draft.launchToTray}
                      onChange={(checked) =>
                        updateField('launchToTray', checked)
                      }
                    />
                  </div>
                </Field>
              </div>

              <div className="grid gap-3 lg:grid-cols-[minmax(0,1fr)_220px]">
                <Field label="Theme">
                  <div className="grid grid-cols-3 gap-2">
                    {(['system', 'light', 'dark'] as const).map((theme) => (
                      <button
                        key={theme}
                        type="button"
                        className={`speed-chip ${draft.theme === theme ? 'speed-chip-active' : ''}`}
                        onClick={() => updateField('theme', theme)}
                      >
                        {theme}
                      </button>
                    ))}
                  </div>
                </Field>

                <Field label="Summary">
                  <div className="rounded-md border border-slate-200 bg-slate-50 px-3 py-2 text-sm text-slate-700">
                    <p className="font-medium text-slate-900">
                      {draft.defaultLanguage.toUpperCase()}
                    </p>
                    <p className="mt-1 text-xs text-slate-500">
                      {draft.providerModel} · {currentSpeedLabel}
                    </p>
                  </div>
                </Field>
              </div>

              <div className="flex flex-wrap items-center gap-2 border-t border-slate-200 pt-4">
                <button
                  className="inline-flex items-center gap-2 rounded-md border border-slate-200 bg-white px-3 py-2 text-sm font-medium text-slate-700 transition hover:bg-slate-50 disabled:cursor-not-allowed disabled:opacity-60"
                  type="button"
                  onClick={() => void hydrate()}
                  disabled={isBusy}
                >
                  <RefreshCw className="h-4 w-4" />
                  Reload
                </button>
                <button
                  className="inline-flex items-center gap-2 rounded-md border border-slate-200 bg-white px-3 py-2 text-sm font-medium text-slate-700 transition hover:bg-slate-50 disabled:cursor-not-allowed disabled:opacity-60"
                  type="button"
                  onClick={() => void handleReset()}
                  disabled={isSaving}
                >
                  <RotateCcw className="h-4 w-4" />
                  Reset defaults
                </button>
                <button
                  className="inline-flex items-center gap-2 rounded-md border border-emerald-600 bg-emerald-600 px-3 py-2 text-sm font-medium text-white transition hover:bg-emerald-700 disabled:cursor-not-allowed disabled:opacity-60"
                  type="submit"
                  disabled={!isDirty || isSaving}
                >
                  <Save className="h-4 w-4" />
                  {isSaving ? 'Saving...' : 'Save settings'}
                </button>
              </div>
            </form>
          </section>

          <section className={panelClassName}>
            <SectionTitle
              icon={<Activity className="h-4 w-4" />}
              title="Runtime snapshot"
              subtitle="A live summary of the current local state that the later capture and AI flows will extend."
            />

            <div className="mt-5 grid gap-3">
              <SummaryLine label="Startup status" value={state.status} />
              <SummaryLine label="Tray status" value={state.trayStatus} />
              <SummaryLine
                label="Screenshot state"
                value={state.screenshotState}
              />
              <SummaryLine
                label="AI result state"
                value={state.aiResultState}
              />
              <SummaryLine
                label="Settings path"
                value={state.settingsPath || 'Not resolved yet'}
              />
              <SummaryLine
                label="Window visible"
                value={String(state.windowVisible)}
              />
            </div>

            <div className="mt-5 rounded-lg border border-slate-200 bg-slate-50 p-4">
              <div className="flex items-center justify-between gap-2">
                <div>
                  <h3 className="text-sm font-semibold text-slate-900">
                    Placeholder result surface
                  </h3>
                  <p className="mt-1 text-xs leading-5 text-slate-500">
                    This area is intentionally empty for stage 1. Later stages
                    will mount capture, recognition, and answer output here.
                  </p>
                </div>
                <ChevronRight className="h-4 w-4 text-slate-400" />
              </div>

              <pre className="mt-4 overflow-auto rounded-md border border-slate-200 bg-white px-3 py-3 text-xs leading-5 text-slate-700">
                {JSON.stringify(
                  {
                    status: state.status,
                    trayStatus: state.trayStatus,
                    screenshotState: state.screenshotState,
                    aiResultState: state.aiResultState,
                    settings: state.settings,
                    settingsPath: state.settingsPath,
                    startupWarning: state.startupWarning,
                  },
                  null,
                  2,
                )}
              </pre>
            </div>
          </section>
        </div>

        <footer className="pb-2 text-xs text-slate-500">
          Stage 1 only: app shell, tray, settings persistence, and error
          reporting. No capture or AI integration is wired yet.
        </footer>
      </main>
    </div>
  );
}

// Wraps a form control with the compact label treatment used throughout settings.
function Field({
  label,
  labelFor,
  children,
}: {
  label: string;
  labelFor?: string;
  children: ReactNode;
}) {
  const labelClassName =
    'mb-2 block text-xs font-medium uppercase tracking-[0.2em] text-slate-500';

  return (
    <div className="block">
      {labelFor ? (
        <label className={labelClassName} htmlFor={labelFor}>
          {label}
        </label>
      ) : (
        <div className={labelClassName}>{label}</div>
      )}
      {children}
    </div>
  );
}

// Renders a settings boolean as a labeled checkbox row with explanatory copy.
function ToggleRow({
  label,
  description,
  checked,
  onChange,
}: {
  label: string;
  description: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}) {
  return (
    <label className="flex cursor-pointer items-start gap-3 rounded-md border border-slate-200 bg-slate-50 px-3 py-3 transition hover:bg-white">
      <input
        className="mt-0.5 h-4 w-4 rounded border-slate-300 text-emerald-600 focus:ring-emerald-500"
        type="checkbox"
        checked={checked}
        onChange={(event) => onChange(event.target.checked)}
      />
      <span className="min-w-0">
        <span className="block text-sm font-medium text-slate-900">
          {label}
        </span>
        <span className="mt-1 block text-xs leading-5 text-slate-500">
          {description}
        </span>
      </span>
    </label>
  );
}

// Maps backend status into the visible header health indicator.
function StatusPill({
  tone,
  children,
}: {
  tone: AppState['status'];
  children: ReactNode;
}) {
  const className =
    tone === 'ready'
      ? 'border-emerald-200 bg-emerald-50 text-emerald-700'
      : tone === 'warning'
        ? 'border-amber-200 bg-amber-50 text-amber-800'
        : tone === 'error'
          ? 'border-rose-200 bg-rose-50 text-rose-700'
          : 'border-slate-200 bg-slate-50 text-slate-600';

  return (
    <span
      className={`inline-flex items-center gap-2 rounded-full border px-3 py-1 text-xs font-semibold uppercase tracking-[0.18em] ${className}`}
    >
      {children}
    </span>
  );
}

// Standardizes compact icon buttons used by the shell command area.
function ActionButton({
  icon,
  label,
  onClick,
  disabled = false,
  emphasis = 'secondary',
}: {
  icon: ReactNode;
  label: string;
  onClick: () => void;
  disabled?: boolean;
  emphasis?: 'secondary' | 'primary';
}) {
  const base =
    'inline-flex items-center gap-2 rounded-md border px-3 py-2 text-sm font-medium transition disabled:cursor-not-allowed disabled:opacity-60';
  const tone =
    emphasis === 'primary'
      ? 'border-slate-900 bg-slate-900 text-white hover:bg-slate-800'
      : 'border-slate-200 bg-white text-slate-700 hover:bg-slate-50';

  return (
    <button
      type="button"
      className={`${base} ${tone}`}
      onClick={onClick}
      disabled={disabled}
    >
      {icon}
      {label}
    </button>
  );
}

// Provides a shared section header for settings and runtime panels.
function SectionTitle({
  icon,
  title,
  subtitle,
}: {
  icon: ReactNode;
  title: string;
  subtitle: string;
}) {
  return (
    <div className="flex items-start gap-3">
      <div className="mt-0.5 inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-md border border-slate-200 bg-slate-50 text-slate-600">
        {icon}
      </div>
      <div>
        <h2 className="text-base font-semibold text-slate-950">{title}</h2>
        <p className="mt-1 text-sm leading-6 text-slate-600">{subtitle}</p>
      </div>
    </div>
  );
}

// Shows a short dashboard metric in the shell header.
function InfoTile({ title, value }: { title: string; value: string }) {
  return (
    <div className="rounded-md border border-slate-200 bg-slate-50 px-3 py-3">
      <div className="text-xs font-medium uppercase tracking-[0.2em] text-slate-500">
        {title}
      </div>
      <div className="mt-2 text-sm font-semibold text-slate-900">{value}</div>
    </div>
  );
}

// Formats runtime snapshot rows while keeping long values from breaking the panel.
function SummaryLine({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-center justify-between gap-3 rounded-md border border-slate-200 bg-white px-3 py-2">
      <span className="text-xs font-medium uppercase tracking-[0.18em] text-slate-500">
        {label}
      </span>
      <span className="max-w-[60%] truncate text-sm font-medium text-slate-900">
        {value}
      </span>
    </div>
  );
}

// Normalizes transient notices so backend warnings and errors are easy to compare.
function NoticeBanner({
  tone,
  children,
}: {
  tone: BannerTone;
  children: ReactNode;
}) {
  const className =
    tone === 'warning'
      ? 'border-amber-200 bg-amber-50 text-amber-900'
      : tone === 'error'
        ? 'border-rose-200 bg-rose-50 text-rose-900'
        : 'border-emerald-200 bg-emerald-50 text-emerald-900';

  return (
    <div
      className={`flex items-start gap-2 rounded-lg border px-4 py-3 text-sm ${className}`}
    >
      {children}
    </div>
  );
}

export default App;
