import {
  Activity,
  AlertTriangle,
  CheckCircle2,
  Languages,
  Monitor,
  RefreshCw,
  RotateCcw,
  Save,
  Settings2,
  Shield,
  SquareStack,
  Trash2,
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
import { CropOverlay } from './components/CropOverlay';
import { ResultPanel } from './components/ResultPanel';
import { useTypewriter } from './hooks/useTypewriter';
import {
  clearCache,
  clearHistory,
  deleteHistoryEntry,
  hideMainWindow,
  listenAiStreamEvent,
  listenGlobalShortcutTriggered,
  listenOpenSettings,
  listenRuntimeStateChanged,
  listHistory,
  loadAppState,
  regenerateWithLanguage,
  resetSettings,
  saveSettings,
  showMainWindow,
  toggleMainWindow,
} from './lib/api';
import { getMessages } from './lib/i18n';
import {
  type AppSettings,
  type AppState,
  type CropSelectionRect,
  DEFAULT_APP_STATE,
  DEFAULT_SETTINGS,
  type HistoryEntry,
  LANGUAGE_OPTIONS,
  type LanguageId,
  OUTPUT_SPEED_OPTIONS,
  PLATFORM_FORMATS,
  UI_LOCALES,
} from './lib/types';

type BannerTone = 'neutral' | 'warning' | 'error';

const panelClassName =
  'rounded-lg border border-slate-200 bg-white/95 p-5 shadow-sm shadow-slate-200/60';

/**
 * 修复可能不完整的设置快照。
 * 旧版本持久化文件可能缺少 uiLocale 字段，此函数确保回退到默认值。
 */
function normalizeSettings(settings: AppSettings): AppSettings {
  return {
    ...DEFAULT_SETTINGS,
    ...settings,
    uiLocale: settings.uiLocale ?? DEFAULT_SETTINGS.uiLocale,
  };
}

/**
 * Question Scan 主应用组件。
 *
 * 职责：作为 React 与 Tauri 后端状态之间的桥梁，管理所有 UI 状态。
 * 包含：设置表单、结果面板、历史记录、AI 流式输出、手动框选覆盖层。
 */
function App() {
  const settingsSectionRef = useRef<HTMLElement | null>(null);
  const [state, setState] = useState<AppState>(DEFAULT_APP_STATE);
  const [draft, setDraft] = useState<AppSettings>(DEFAULT_SETTINGS);
  const [cropSelection, setCropSelection] = useState<CropSelectionRect | null>(
    null,
  );
  const [, setConfirmedCropSelection] = useState<CropSelectionRect | null>(
    null,
  );
  const [isBusy, setIsBusy] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [notice, setNotice] = useState<{
    tone: BannerTone;
    text: string;
  } | null>(null);
  const [aiError, setAiError] = useState<string | null>(null);
  const [currentLanguage, setCurrentLanguage] = useState<LanguageId>(
    DEFAULT_SETTINGS.defaultLanguage,
  );
  const [historyEntries, setHistoryEntries] = useState<HistoryEntry[]>([]);
  const [isHistoryLoading, setIsHistoryLoading] = useState(false);

  const typewriter = useTypewriter({
    speed: draft.outputSpeed,
    customCharactersPerSecond: draft.customCharactersPerSecond,
  });

  const messages = useMemo(() => getMessages(draft.uiLocale), [draft.uiLocale]);
  const cropOverlayMessages = useMemo(
    () =>
      draft.uiLocale === 'enUs'
        ? {
            description:
              'When auto recognition falls back, drag across the question area to keep a crop for the manual flow.',
            idleHint: 'Drag to start a selection',
            selectionLabel: 'Selected region',
            cancelLabel: 'Cancel',
            retryLabel: 'Retry auto recognition',
            confirmLabel: 'Confirm crop',
            confirmDisabledHint: 'Select a question region before confirming.',
            cancelNotice: 'Manual selection canceled.',
            retryNotice:
              'Manual selection cleared. Auto recognition is ready to retry.',
            confirmNotice:
              'Crop confirmed. This region is ready for the high-resolution crop step.',
            title: 'Manual question selection',
          }
        : {
            description:
              '当自动识别进入兜底时，在题目区域上拖拽就可以保留一个裁剪框。',
            idleHint: '拖拽开始框选',
            selectionLabel: '当前选区',
            cancelLabel: '取消',
            retryLabel: '重新自动识别',
            confirmLabel: '确认裁剪',
            confirmDisabledHint: '请先框选题目区域再确认。',
            cancelNotice: '已取消手动框选。',
            retryNotice: '已清空手动选区，重新进入自动识别状态。',
            confirmNotice: '已确认裁剪区域，后续会使用这个选区生成高清裁剪图。',
            title: '手动框选',
          },
    [draft.uiLocale],
  );

  /**
   * 将 Tauri 命令错误转换为用户可读的文案。
   * 优先使用 Rust 返回的错误消息，否则使用 fallback。
   */
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

  /**
   * 将后端状态快照同步到前端。
   * 设置表单、运行时面板等所有 UI 都依赖此函数刷新。
   */
  const applySnapshot = useCallback((snapshot: AppState) => {
    const settings = normalizeSettings(snapshot.settings);
    setState({
      ...snapshot,
      settings,
    });
    setDraft(settings);
  }, []);

  /**
   * 应用启动时的状态水合函数。
   * 从后端加载初始状态，如果失败则回退到默认状态并显示错误提示。
   */
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
    if (state.screenshotState === 'selecting') {
      setConfirmedCropSelection(null);
      return;
    }

    setCropSelection(null);
  }, [state.screenshotState]);

  const handleCancelManualCrop = useCallback(() => {
    setCropSelection(null);
    setConfirmedCropSelection(null);
    setState((current) => ({
      ...current,
      trayStatus: 'idle',
      screenshotState: 'idle',
      aiResultState: 'idle',
    }));
    setNotice({
      tone: 'neutral',
      text: cropOverlayMessages.cancelNotice,
    });
  }, [cropOverlayMessages.cancelNotice]);

  const handleRetryAutoRecognition = useCallback(() => {
    setCropSelection(null);
    setConfirmedCropSelection(null);
    setState((current) => ({
      ...current,
      trayStatus: 'recognizing',
      screenshotState: 'cropping',
      aiResultState: 'idle',
    }));
    setNotice({
      tone: 'neutral',
      text: cropOverlayMessages.retryNotice,
    });
  }, [cropOverlayMessages.retryNotice]);

  const handleConfirmManualCrop = useCallback(
    (selection: CropSelectionRect) => {
      setCropSelection(null);
      setConfirmedCropSelection(selection);
      setState((current) => ({
        ...current,
        trayStatus: 'recognizing',
        screenshotState: 'ready',
        aiResultState: 'idle',
      }));
      setNotice({
        tone: 'neutral',
        text: `${cropOverlayMessages.confirmNotice} (${selection.width} x ${selection.height})`,
      });
    },
    [cropOverlayMessages.confirmNotice],
  );

  /**
   * AI 流式输出事件监听。
   * 监听后端发射的 chunk/done/error 事件，驱动打字机效果和状态变更。
   */
  useEffect(() => {
    let cancelled = false;
    let unlistenStream: (() => void) | null = null;

    async function wireStreamEvents() {
      try {
        unlistenStream = await listenAiStreamEvent((event) => {
          if (cancelled) return;
          const payload = event.payload;

          switch (payload.type) {
            case 'chunk': {
              typewriter.append(payload.content);
              setState((current) => ({
                ...current,
                aiResultState: 'streaming',
                trayStatus: 'generating',
              }));
              break;
            }
            case 'done': {
              setState((current) => ({
                ...current,
                aiResultState: 'complete',
                trayStatus: 'complete',
              }));
              break;
            }
            case 'error': {
              setAiError(payload.message);
              setState((current) => ({
                ...current,
                aiResultState: 'failed',
                trayStatus: 'failed',
              }));
              break;
            }
          }
        });
      } catch (error) {
        const message = errorMessage(error, 'Failed to listen to AI stream');
        setNotice({ tone: 'error', text: message });
      }
    }

    void wireStreamEvents();

    return () => {
      cancelled = true;
      if (unlistenStream) {
        unlistenStream();
      }
    };
  }, [errorMessage, typewriter]);

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

  // ---------------------------------------------------------------------------
  // 结果面板动作处理器
  // ---------------------------------------------------------------------------

  /** 从完整答案中提取 fenced code block 并复制到剪贴板。 */
  const handleCopyCode = useCallback(() => {
    // Extract code blocks from fullText
    const codeBlockRegex = /```[\w]*\n([\s\S]*?)```/g;
    const matches: string[] = [];
    let match: RegExpExecArray | null = null;
    do {
      match = codeBlockRegex.exec(typewriter.fullText);
      if (match) {
        matches.push(match[1].trim());
      }
    } while (match !== null);
    const codeToCopy = matches.length > 0 ? matches.join('\n\n') : '';
    if (codeToCopy) {
      void navigator.clipboard.writeText(codeToCopy);
      setNotice({
        tone: 'neutral',
        text: messages.resultPanel.codeCopied,
      });
    }
  }, [typewriter.fullText, messages.resultPanel.codeCopied]);

  /** 复制完整答案（含 Markdown 格式）到剪贴板。 */
  const handleCopyFullAnswer = useCallback(() => {
    if (typewriter.fullText) {
      void navigator.clipboard.writeText(typewriter.fullText);
      setNotice({
        tone: 'neutral',
        text: messages.resultPanel.copyFullAnswerNotice,
      });
    }
  }, [typewriter.fullText, messages.resultPanel.copyFullAnswerNotice]);

  /** 清空结果面板，重置打字机和错误状态。 */
  const handleClearResult = useCallback(() => {
    typewriter.reset();
    setAiError(null);
    setState((current) => ({
      ...current,
      aiResultState: 'idle',
      trayStatus: 'idle',
    }));
    setNotice({
      tone: 'neutral',
      text: messages.resultPanel.clearResultNotice,
    });
  }, [typewriter, messages.resultPanel.clearResultNotice]);

  /** 重新生成答案（使用当前设置）。 */
  const handleRegenerate = useCallback(() => {
    typewriter.reset();
    setAiError(null);
    setState((current) => ({
      ...current,
      aiResultState: 'loading',
      trayStatus: 'generating',
    }));
    setNotice({
      tone: 'neutral',
      text: messages.resultPanel.regenerateNotice,
    });
    // TODO: Trigger actual AI request regeneration when screenshot flow is wired
  }, [typewriter, messages.resultPanel.regenerateNotice]);

  /**
   * 切换语言并重新生成答案。
   * 调用后端 regenerate_with_language 命令复用同一张截图数据。
   */
  const handleSwitchLanguage = useCallback(
    async (language: LanguageId) => {
      setCurrentLanguage(language);
      typewriter.reset();
      setAiError(null);
      setState((current) => ({
        ...current,
        aiResultState: 'loading',
        trayStatus: 'generating',
      }));
      setNotice({
        tone: 'neutral',
        text: messages.resultPanel.switchLanguageNotice,
      });

      try {
        await regenerateWithLanguage(language);
      } catch (error) {
        const message = errorMessage(
          error,
          'Failed to regenerate with new language',
        );
        setAiError(message);
        setState((current) => ({
          ...current,
          aiResultState: 'failed',
          trayStatus: 'failed',
        }));
        setNotice({
          tone: 'error',
          text: message,
        });
      }
    },
    [typewriter, messages.resultPanel.switchLanguageNotice, errorMessage],
  );

  const loadHistory = useCallback(async () => {
    if (!draft.saveHistory) {
      setHistoryEntries([]);
      return;
    }
    setIsHistoryLoading(true);
    try {
      const entries = await listHistory();
      setHistoryEntries(entries);
    } catch (error) {
      const message = errorMessage(error, 'Failed to load history');
      setNotice({ tone: 'error', text: message });
    } finally {
      setIsHistoryLoading(false);
    }
  }, [draft.saveHistory, errorMessage]);

  const handleClearCache = useCallback(async () => {
    try {
      await clearCache();
      setNotice({
        tone: 'neutral',
        text: messages.actions.clearCacheNotice,
      });
    } catch (error) {
      const message = errorMessage(error, 'Failed to clear cache');
      setNotice({ tone: 'error', text: message });
    }
  }, [errorMessage, messages.actions.clearCacheNotice]);

  const handleDeleteHistoryEntry = useCallback(
    async (id: string) => {
      try {
        await deleteHistoryEntry(id);
        setHistoryEntries((current) =>
          current.filter((entry) => entry.id !== id),
        );
        setNotice({
          tone: 'neutral',
          text: messages.actions.deleteHistoryEntryNotice,
        });
      } catch (error) {
        const message = errorMessage(error, 'Failed to delete history entry');
        setNotice({ tone: 'error', text: message });
      }
    },
    [errorMessage, messages.actions.deleteHistoryEntryNotice],
  );

  const handleClearHistory = useCallback(async () => {
    try {
      await clearHistory();
      setHistoryEntries([]);
      setNotice({
        tone: 'neutral',
        text: messages.actions.clearHistoryNotice,
      });
    } catch (error) {
      const message = errorMessage(error, 'Failed to clear history');
      setNotice({ tone: 'error', text: message });
    }
  }, [errorMessage, messages.actions.clearHistoryNotice]);

  const isDirty = useMemo(
    () => JSON.stringify(draft) !== JSON.stringify(state.settings),
    [draft, state.settings],
  );

  /**
   * 更新设置表单中的单个字段。
   * 泛型约束保证 key 和 value 的类型匹配 AppSettings 契约。
   */
  const updateField = useCallback(
    <K extends keyof AppSettings>(key: K, value: AppSettings[K]) => {
      setDraft((current) => ({
        ...current,
        [key]: value,
      }));
    },
    [],
  );

  /**
   * 保存设置草稿到后端，并用返回的快照重新同步 UI。
   * 保存成功后会显示提示，失败则显示错误信息。
   */
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

  /**
   * 重置所有设置为默认值。
   * 同时重置后端持久化数据和前端草稿状态。
   */
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

  /**
   * 统一的窗口显隐控制。
   * 托盘菜单和标题栏按钮共用此函数，保证行为一致。
   */
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

                <Field
                  label={messages.fields.providerName}
                  labelFor="provider-name"
                >
                  <input
                    id="provider-name"
                    className="field-input"
                    value={draft.providerName}
                    onChange={(event) =>
                      updateField('providerName', event.target.value)
                    }
                    placeholder="OpenAI-compatible"
                  />
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
                  label={messages.fields.requestTimeoutSeconds}
                  labelFor="request-timeout-seconds"
                >
                  <input
                    id="request-timeout-seconds"
                    className="field-input"
                    type="number"
                    min={1}
                    step={1}
                    value={draft.requestTimeoutSeconds}
                    onChange={(event) =>
                      updateField(
                        'requestTimeoutSeconds',
                        Number(event.target.value || 0),
                      )
                    }
                  />
                </Field>

                <Field
                  label={messages.fields.apiKey}
                  labelFor="provider-api-key"
                >
                  <input
                    id="provider-api-key"
                    className="field-input"
                    type="password"
                    value={draft.providerApiKey}
                    onChange={(event) =>
                      updateField('providerApiKey', event.target.value)
                    }
                    autoComplete="off"
                    spellCheck={false}
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
                  label={messages.fields.platformFormat}
                  labelFor="platform-format"
                >
                  <select
                    id="platform-format"
                    className="field-input"
                    value={draft.platformFormat}
                    onChange={(event) =>
                      updateField(
                        'platformFormat',
                        event.target.value as AppSettings['platformFormat'],
                      )
                    }
                  >
                    {PLATFORM_FORMATS.map((format) => (
                      <option key={format} value={format}>
                        {messages.platformFormat[format]}
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
                      label={messages.settings.streamingEnabledLabel}
                      description={
                        messages.settings.streamingEnabledDescription
                      }
                      checked={draft.streamingEnabled}
                      onChange={(checked) =>
                        updateField('streamingEnabled', checked)
                      }
                    />
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

              <div className="mt-6 rounded-lg border border-slate-200 bg-slate-50/60 p-4">
                <SectionTitle
                  icon={<Shield className="h-4 w-4" />}
                  title={messages.privacy.title}
                  subtitle={messages.privacy.subtitle}
                />
                <div className="mt-3 space-y-3">
                  <div className="rounded-md border border-amber-200 bg-amber-50 px-3 py-2 text-sm text-amber-800">
                    {messages.privacy.screenshotWarning}
                  </div>
                  <div>
                    <h4 className="text-sm font-medium text-slate-900">
                      {messages.privacy.dataFlowTitle}
                    </h4>
                    <p className="mt-1 whitespace-pre-line text-sm leading-6 text-slate-600">
                      {messages.privacy.dataFlowDescription}
                    </p>
                  </div>
                </div>
              </div>

              <div className="mt-6 rounded-lg border border-slate-200 bg-slate-50/60 p-4">
                <h3 className="text-sm font-semibold text-slate-900">
                  {draft.uiLocale === 'zhCn' ? '数据管理' : 'Data management'}
                </h3>
                <div className="mt-3 flex flex-wrap gap-2">
                  <button
                    type="button"
                    className="inline-flex items-center gap-2 rounded-md border border-slate-200 bg-white px-3 py-2 text-sm font-medium text-slate-700 transition hover:bg-slate-50"
                    onClick={() => void handleClearCache()}
                  >
                    <Trash2 className="h-4 w-4" />
                    {messages.actions.clearCache}
                  </button>
                  {draft.saveHistory && (
                    <button
                      type="button"
                      className="inline-flex items-center gap-2 rounded-md border border-slate-200 bg-white px-3 py-2 text-sm font-medium text-slate-700 transition hover:bg-slate-50"
                      onClick={() => void handleClearHistory()}
                    >
                      <Trash2 className="h-4 w-4" />
                      {messages.actions.clearHistory}
                    </button>
                  )}
                </div>
                {draft.saveHistory && (
                  <div className="mt-3">
                    <button
                      type="button"
                      className="text-sm text-slate-500 underline hover:text-slate-700"
                      onClick={() => void loadHistory()}
                      disabled={isHistoryLoading}
                    >
                      {isHistoryLoading
                        ? 'Loading...'
                        : draft.uiLocale === 'zhCn'
                          ? '刷新历史记录'
                          : 'Refresh history'}
                    </button>
                    {historyEntries.length > 0 && (
                      <ul className="mt-2 space-y-2">
                        {historyEntries.map((entry) => (
                          <li
                            key={entry.id}
                            className="flex items-center justify-between rounded-md border border-slate-200 bg-white px-3 py-2"
                          >
                            <div className="min-w-0">
                              <p className="truncate text-sm font-medium text-slate-900">
                                {entry.recognizedTitle ??
                                  (draft.uiLocale === 'zhCn'
                                    ? '无标题'
                                    : 'Untitled')}
                              </p>
                              <p className="text-xs text-slate-500">
                                {entry.language} · {entry.platform} ·{' '}
                                {entry.model}
                              </p>
                            </div>
                            <button
                              type="button"
                              className="ml-2 rounded-md p-1 text-slate-400 transition hover:bg-slate-100 hover:text-slate-600"
                              onClick={() =>
                                void handleDeleteHistoryEntry(entry.id)
                              }
                              aria-label={messages.actions.deleteHistoryEntry}
                            >
                              <Trash2 className="h-4 w-4" />
                            </button>
                          </li>
                        ))}
                      </ul>
                    )}
                    {historyEntries.length === 0 && !isHistoryLoading && (
                      <p className="mt-2 text-sm text-slate-500">
                        {draft.uiLocale === 'zhCn'
                          ? '暂无历史记录'
                          : 'No history entries'}
                      </p>
                    )}
                  </div>
                )}
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

            <div className="mt-5 flex h-[480px] flex-col">
              <ResultPanel
                state={state.aiResultState}
                fullText={typewriter.fullText}
                displayedText={typewriter.displayedText}
                error={aiError}
                currentLanguage={currentLanguage}
                messages={{
                  title: messages.result.title,
                  idleHint: messages.resultPanel.idleHint,
                  screenshotStatus: messages.resultPanel.screenshotStatus,
                  recognizingStatus: messages.resultPanel.recognizingStatus,
                  generatingStatus: messages.resultPanel.generatingStatus,
                  completeStatus: messages.resultPanel.completeStatus,
                  failedStatus: messages.resultPanel.failedStatus,
                  copyCode: messages.resultPanel.copyCode,
                  copyCodeSuccess: messages.resultPanel.codeCopied,
                  copyFullAnswer: messages.resultPanel.copyFullAnswer,
                  copyFullAnswerSuccess:
                    messages.resultPanel.copyFullAnswerNotice,
                  clearResult: messages.resultPanel.clearResult,
                  regenerate: messages.resultPanel.regenerate,
                  switchLanguage: messages.resultPanel.switchLanguage,
                  codeCopied: messages.resultPanel.codeCopied,
                  answerCopied: messages.resultPanel.fullAnswerCopied,
                  resultCleared: messages.resultPanel.clearResultNotice,
                }}
                onCopyCode={handleCopyCode}
                onCopyFullAnswer={handleCopyFullAnswer}
                onClearResult={handleClearResult}
                onRegenerate={handleRegenerate}
                onSwitchLanguage={handleSwitchLanguage}
              />
            </div>
          </section>
        </div>

        <footer className="pb-2 text-xs text-slate-500">
          {messages.footer}
        </footer>
      </main>

      <CropOverlay
        active={state.screenshotState === 'selecting'}
        title={cropOverlayMessages.title}
        description={cropOverlayMessages.description}
        idleHint={cropOverlayMessages.idleHint}
        selectionLabel={cropOverlayMessages.selectionLabel}
        cancelLabel={cropOverlayMessages.cancelLabel}
        retryLabel={cropOverlayMessages.retryLabel}
        confirmLabel={cropOverlayMessages.confirmLabel}
        confirmDisabledHint={cropOverlayMessages.confirmDisabledHint}
        selection={cropSelection}
        onSelectionChange={setCropSelection}
        onSelectionComplete={setCropSelection}
        onCancel={handleCancelManualCrop}
        onRetryRecognition={handleRetryAutoRecognition}
        onConfirmSelection={handleConfirmManualCrop}
      />
    </div>
  );
}

/**
 * 设置表单字段包装组件。
 * 统一标签样式和布局，用于所有设置输入项。
 */
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

/**
 * 设置开关行组件。
 * 将布尔值渲染为带说明文字的复选框行。
 */
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

/**
 * 状态指示器胶囊组件。
 * 根据后端状态显示不同颜色的健康指示标签。
 */
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

/**
 * 操作按钮组件。
 * 标题栏和工具栏共用的图标按钮，支持 primary/secondary 两种强调级别。
 */
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

/**
 * 区块标题组件。
 * 设置区和运行状态面板的共享标题样式。
 */
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

/**
 * 信息卡片组件。
 * 在标题栏展示简短的仪表板指标（如窗口状态、托盘状态、版本号）。
 */
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

/**
 * 运行时状态行组件。
 * 格式化运行时快照的键值对，防止长文本撑破面板布局。
 */
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

/**
 * 通知横幅组件。
 * 统一处理后端警告、错误和成功提示的样式和图标。
 */
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
