import {
  Activity,
  AlertTriangle,
  CheckCircle2,
  ChevronRight,
  Languages,
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
  useRef,
  useState,
} from 'react';
import {
  hideMainWindow,
  listenGlobalShortcutTriggered,
  listenOpenSettings,
  listenRuntimeStateChanged,
  loadAppState,
  resetSettings,
  saveSettings,
  showMainWindow,
  toggleMainWindow,
} from './lib/api';
import { getMessages } from './lib/i18n';
import {
  type AppSettings,
  type AppState,
  DEFAULT_APP_STATE,
  DEFAULT_SETTINGS,
  LANGUAGE_OPTIONS,
  OUTPUT_SPEED_OPTIONS,
  UI_LOCALES,
} from './lib/types';

type BannerTone = 'neutral' | 'warning' | 'error';

const panelClassName =
  'rounded-lg border border-slate-200 bg-white/95 p-5 shadow-sm shadow-slate-200/60';

// Repairs partial settings snapshots so older persisted files still get a UI locale.
function normalizeSettings(settings: AppSettings): AppSettings {
  return {
    ...DEFAULT_SETTINGS,
    ...settings,
    uiLocale: settings.uiLocale ?? DEFAULT_SETTINGS.uiLocale,
  };
}

// Owns the stage-1 UI state bridge between React and the Tauri backend snapshot.
function App() {
  const settingsSectionRef = useRef<HTMLElement | null>(null);
  const [state, setState] = useState<AppState>(DEFAULT_APP_STATE);
  const [draft, setDraft] = useState<AppSettings>(DEFAULT_SETTINGS);
  const [isBusy, setIsBusy] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [notice, setNotice] = useState<{
    tone: BannerTone;
    text: string;
  } | null>(null);

  const messages = useMemo(() => getMessages(draft.uiLocale), [draft.uiLocale]);

  // Converts serialized Tauri command errors into user-facing copy when Rust supplies it.
  const errorMessage = useCallback((error: unknown, fallback: string) => {
    if (error instanceof Error) {
      return error.message;
    }
    if (typeof error === 'string') {
      return error;
    }
    if (
      error &&
      typeof error === 'object' &&
      'message' in error &&
      typeof error.message === 'string'
    ) {
      return error.message;
    }
    return fallback;
  }, []);

  // Keeps every backend snapshot aligned with the current frontend settings contract.
  const applySnapshot = useCallback((snapshot: AppState) => {
    const settings = normalizeSettings(snapshot.settings);
    setState({
      ...snapshot,
      settings,
    });
    setDraft(settings);
  }, []);

  // Reloads backend state and is the first place to inspect if startup hydration fails.
  const hydrate = useCallback(async () => {
    setIsBusy(true);
    setNotice(null);

    try {
      const snapshot = await loadAppState();
      applySnapshot(snapshot);
      const warning = snapshot.startupWarning ?? snapshot.globalShortcutError;
      if (warning) {
        setNotice({ tone: 'warning', text: warning });
      }
    } catch (error) {
      const fallbackMessage = errorMessage(
        error,
        messages.notices.backendNotResponding,
      );
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
  }, [applySnapshot, errorMessage, messages.notices.backendNotResponding]);

  useEffect(() => {
    void hydrate();
  }, [hydrate]);

  useEffect(() => {
    let cancelled = false;
    const unlisteners: Array<() => void> = [];

    async function wireRuntimeEvents() {
      try {
        const [unlistenShortcut, unlistenSettings, unlistenRuntime] =
          await Promise.all([
            listenGlobalShortcutTriggered(async () => {
              await hydrate();
              setNotice({
                tone: 'neutral',
                text: messages.notices.shortcutTriggered,
              });
            }),
            listenOpenSettings(async () => {
              await hydrate();
              settingsSectionRef.current?.scrollIntoView?.({
                block: 'start',
              });
            }),
            listenRuntimeStateChanged(async () => {
              await hydrate();
            }),
          ]);

        if (cancelled) {
          unlistenShortcut();
          unlistenSettings();
          unlistenRuntime();
          return;
        }

        unlisteners.push(unlistenShortcut, unlistenSettings, unlistenRuntime);
      } catch (error) {
        const message = errorMessage(
          error,
          messages.notices.shortcutListenerFailed,
        );
        setNotice({ tone: 'error', text: message });
      }
    }

    void wireRuntimeEvents();

    return () => {
      cancelled = true;
      for (const unlisten of unlisteners) {
        unlisten();
      }
    };
  }, [
    errorMessage,
    hydrate,
    messages.notices.shortcutListenerFailed,
    messages.notices.shortcutTriggered,
  ]);

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
      applySnapshot(snapshot);
      setNotice({
        tone: 'neutral',
        text: getMessages(draft.uiLocale).notices.saveSucceeded,
      });
    } catch (error) {
      const message = errorMessage(error, messages.notices.saveFailed);
      setNotice({ tone: 'error', text: message });
    } finally {
      setIsSaving(false);
    }
  }, [applySnapshot, draft, errorMessage, messages.notices.saveFailed]);

  // Resets both backend settings and the local draft to the default profile.
  const handleReset = useCallback(async () => {
    setIsSaving(true);
    setNotice(null);

    try {
      const snapshot = await resetSettings();
      applySnapshot(snapshot);
      setNotice({
        tone: 'neutral',
        text: getMessages(DEFAULT_SETTINGS.uiLocale).notices.resetSucceeded,
      });
    } catch (error) {
      const message = errorMessage(error, messages.notices.resetFailed);
      setNotice({ tone: 'error', text: message });
    } finally {
      setIsSaving(false);
    }
  }, [applySnapshot, errorMessage, messages.notices.resetFailed]);

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
        applySnapshot(snapshot);
      } catch (error) {
        const message = errorMessage(
          error,
          messages.notices.windowActionFailed,
        );
        setNotice({ tone: 'error', text: message });
      }
    },
    [applySnapshot, errorMessage, messages.notices.windowActionFailed],
  );

  const outputSpeedOptions = useMemo(
    () =>
      OUTPUT_SPEED_OPTIONS.map((option) => ({
        ...option,
        label: messages.outputSpeed[option.value],
      })),
    [messages],
  );

  const currentSpeedLabel =
    messages.outputSpeed[draft.outputSpeed] ?? messages.outputSpeed.normal;

  return (
    <div className="min-h-screen bg-slate-50 text-slate-900">
      <main className="mx-auto flex min-h-screen w-full max-w-7xl flex-col gap-5 px-4 py-4 sm:px-6 lg:px-8">
        <header className={panelClassName}>
          <div className="flex flex-col gap-4 xl:flex-row xl:items-start xl:justify-between">
            <div className="space-y-2">
              <div className="flex items-center gap-2 text-xs font-medium uppercase tracking-[0.28em] text-slate-500">
                <SquareStack className="h-4 w-4" />
                {messages.header.eyebrow}
              </div>
              <div className="space-y-1">
                <h1 className="text-2xl font-semibold text-slate-950">
                  {messages.header.title}
                </h1>
                <p className="max-w-3xl text-sm leading-6 text-slate-600">
                  {messages.header.subtitle}
                </p>
              </div>
            </div>

            <div className="flex flex-wrap items-center gap-2">
              <StatusPill tone={state.status}>
                {messages.appStatus[state.status]}
              </StatusPill>
              <ActionButton
                icon={<Monitor className="h-4 w-4" />}
                label={
                  state.windowVisible
                    ? messages.actions.hideWindow
                    : messages.actions.showWindow
                }
                onClick={() =>
                  void handleWindowAction(state.windowVisible ? 'hide' : 'show')
                }
              />
              <ActionButton
                icon={<RefreshCw className="h-4 w-4" />}
                label={messages.actions.reload}
                onClick={() => void hydrate()}
                disabled={isBusy}
              />
              <ActionButton
                icon={<RotateCcw className="h-4 w-4" />}
                label={messages.actions.reset}
                onClick={() => void handleReset()}
                disabled={isSaving}
              />
              <ActionButton
                icon={<Save className="h-4 w-4" />}
                label={
                  isSaving ? messages.actions.saving : messages.actions.save
                }
                onClick={persist}
                disabled={!isDirty || isSaving}
                emphasis="primary"
              />
            </div>
          </div>

          <div className="mt-4 grid gap-3 md:grid-cols-3">
            <InfoTile
              title={messages.info.window}
              value={
                state.windowVisible
                  ? messages.info.visible
                  : messages.info.hidden
              }
            />
            <InfoTile
              title={messages.info.tray}
              value={messages.trayStatus[state.trayStatus]}
            />
            <InfoTile title={messages.info.version} value={state.version} />
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
          <section className={panelClassName} ref={settingsSectionRef}>
            <SectionTitle
              icon={<Settings2 className="h-4 w-4" />}
              title={messages.settings.title}
              subtitle={messages.settings.subtitle}
            />

            <form
              className="mt-5 grid gap-4"
              onSubmit={(event) => {
                event.preventDefault();
                void persist();
              }}
            >
              <div className="grid gap-4 lg:grid-cols-2">
                <Field
                  label={messages.fields.interfaceLanguage}
                  labelFor="ui-locale"
                >
                  <div className="relative">
                    <Languages className="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" />
                    <select
                      id="ui-locale"
                      className="field-input pl-9"
                      value={draft.uiLocale}
                      onChange={(event) =>
                        updateField(
                          'uiLocale',
                          event.target.value as AppSettings['uiLocale'],
                        )
                      }
                    >
                      {UI_LOCALES.map((locale) => (
                        <option key={locale} value={locale}>
                          {messages.localeOptions[locale]}
                        </option>
                      ))}
                    </select>
                  </div>
                </Field>

                <Field label={messages.fields.model} labelFor="provider-model">
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

                <Field
                  label={messages.fields.providerBaseUrl}
                  labelFor="provider-base-url"
                >
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

                <Field
                  label={messages.fields.defaultLanguage}
                  labelFor="default-language"
                >
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

                <Field
                  label={messages.fields.globalShortcut}
                  labelFor="global-shortcut"
                >
                  <input
                    id="global-shortcut"
                    className="field-input"
                    value={draft.globalShortcut}
                    onChange={(event) =>
                      updateField('globalShortcut', event.target.value)
                    }
                    placeholder="Ctrl+Shift+Q"
                    disabled={!draft.globalShortcutEnabled}
                  />
                </Field>
              </div>

              <div className="grid gap-4 lg:grid-cols-[minmax(0,1fr)_minmax(260px,320px)]">
                <Field label={messages.fields.outputSpeed}>
                  <div className="grid grid-cols-2 gap-2 sm:grid-cols-4">
                    {outputSpeedOptions.map((option) => (
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
                        {messages.fields.charactersPerSecond}
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

                <Field label={messages.fields.presentation}>
                  <div className="grid gap-3">
                    <ToggleRow
                      label={messages.settings.shortcutEnabledLabel}
                      description={messages.settings.shortcutEnabledDescription}
                      checked={draft.globalShortcutEnabled}
                      onChange={(checked) =>
                        updateField('globalShortcutEnabled', checked)
                      }
                    />
                    <ToggleRow
                      label={messages.settings.saveHistoryLabel}
                      description={messages.settings.saveHistoryDescription}
                      checked={draft.saveHistory}
                      onChange={(checked) =>
                        updateField('saveHistory', checked)
                      }
                    />
                    <ToggleRow
                      label={messages.settings.launchToTrayLabel}
                      description={messages.settings.launchToTrayDescription}
                      checked={draft.launchToTray}
                      onChange={(checked) =>
                        updateField('launchToTray', checked)
                      }
                    />
                  </div>
                </Field>
              </div>

              <div className="grid gap-3 lg:grid-cols-[minmax(0,1fr)_220px]">
                <Field label={messages.fields.theme}>
                  <div className="grid grid-cols-3 gap-2">
                    {(['system', 'light', 'dark'] as const).map((theme) => (
                      <button
                        key={theme}
                        type="button"
                        className={`speed-chip ${draft.theme === theme ? 'speed-chip-active' : ''}`}
                        onClick={() => updateField('theme', theme)}
                      >
                        {messages.theme[theme]}
                      </button>
                    ))}
                  </div>
                </Field>

                <Field label={messages.fields.summary}>
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
                  {messages.actions.reload}
                </button>
                <button
                  className="inline-flex items-center gap-2 rounded-md border border-slate-200 bg-white px-3 py-2 text-sm font-medium text-slate-700 transition hover:bg-slate-50 disabled:cursor-not-allowed disabled:opacity-60"
                  type="button"
                  onClick={() => void handleReset()}
                  disabled={isSaving}
                >
                  <RotateCcw className="h-4 w-4" />
                  {messages.actions.resetDefaults}
                </button>
                <button
                  className="inline-flex items-center gap-2 rounded-md border border-emerald-600 bg-emerald-600 px-3 py-2 text-sm font-medium text-white transition hover:bg-emerald-700 disabled:cursor-not-allowed disabled:opacity-60"
                  type="submit"
                  disabled={!isDirty || isSaving}
                >
                  <Save className="h-4 w-4" />
                  {isSaving
                    ? messages.actions.savingEllipsis
                    : messages.actions.saveSettings}
                </button>
              </div>
            </form>
          </section>

          <section className={panelClassName}>
            <SectionTitle
              icon={<Activity className="h-4 w-4" />}
              title={messages.runtime.title}
              subtitle={messages.runtime.subtitle}
            />

            <div className="mt-5 grid gap-3">
              <SummaryLine
                label={messages.runtime.startupStatus}
                value={messages.appStatus[state.status]}
              />
              <SummaryLine
                label={messages.runtime.trayStatus}
                value={messages.trayStatus[state.trayStatus]}
              />
              <SummaryLine
                label={messages.runtime.screenshotState}
                value={messages.screenshotState[state.screenshotState]}
              />
              <SummaryLine
                label={messages.runtime.aiResultState}
                value={messages.aiResultState[state.aiResultState]}
              />
              <SummaryLine
                label={messages.runtime.settingsPath}
                value={state.settingsPath || messages.info.notResolved}
              />
              <SummaryLine
                label={messages.runtime.windowVisible}
                value={
                  state.windowVisible
                    ? messages.info.visible
                    : messages.info.hidden
                }
              />
              <SummaryLine
                label={messages.fields.globalShortcutEnabled}
                value={
                  state.settings.globalShortcutEnabled
                    ? messages.appStatus.ready
                    : messages.info.hidden
                }
              />
              <SummaryLine
                label={messages.fields.globalShortcut}
                value={
                  state.globalShortcutRegistered
                    ? state.settings.globalShortcut
                    : state.globalShortcutError || messages.info.notResolved
                }
              />
              <SummaryLine
                label={messages.runtime.shortcutTriggers}
                value={String(state.globalShortcutTriggerCount)}
              />
            </div>

            <div className="mt-5 rounded-lg border border-slate-200 bg-slate-50 p-4">
              <div className="flex items-center justify-between gap-2">
                <div>
                  <h3 className="text-sm font-semibold text-slate-900">
                    {messages.result.title}
                  </h3>
                  <p className="mt-1 text-xs leading-5 text-slate-500">
                    {messages.result.body}
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
                    globalShortcutRegistered: state.globalShortcutRegistered,
                    globalShortcutError: state.globalShortcutError,
                    globalShortcutTriggerCount:
                      state.globalShortcutTriggerCount,
                  },
                  null,
                  2,
                )}
              </pre>
            </div>
          </section>
        </div>

        <footer className="pb-2 text-xs text-slate-500">
          {messages.footer}
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
