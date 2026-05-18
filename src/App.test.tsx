import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
} from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import App from './App';
import { DEFAULT_APP_STATE, DEFAULT_SETTINGS } from './lib/types';

const {
  loadAppStateMock,
  saveSettingsMock,
  resetSettingsMock,
  clearCacheMock,
  clearHistoryMock,
  deleteHistoryEntryMock,
  listHistoryMock,
  saveHistoryEntryMock,
  listRagImportsMock,
  deleteRagImportMock,
  clearRagIndexMock,
  rebuildRagIndexMock,
  regenerateWithLanguageMock,
  showMainWindowMock,
  hideMainWindowMock,
  toggleMainWindowMock,
  listenGlobalShortcutTriggeredMock,
  listenOpenSettingsMock,
  listenRuntimeStateChangedMock,
  listenAiStreamEventMock,
} = vi.hoisted(() => ({
  loadAppStateMock: vi.fn(),
  saveSettingsMock: vi.fn(),
  resetSettingsMock: vi.fn(),
  clearCacheMock: vi.fn(),
  clearHistoryMock: vi.fn(),
  deleteHistoryEntryMock: vi.fn(),
  listHistoryMock: vi.fn(),
  saveHistoryEntryMock: vi.fn(),
  listRagImportsMock: vi.fn(),
  deleteRagImportMock: vi.fn(),
  clearRagIndexMock: vi.fn(),
  rebuildRagIndexMock: vi.fn(),
  regenerateWithLanguageMock: vi.fn(),
  showMainWindowMock: vi.fn(),
  hideMainWindowMock: vi.fn(),
  toggleMainWindowMock: vi.fn(),
  listenGlobalShortcutTriggeredMock: vi.fn(),
  listenOpenSettingsMock: vi.fn(),
  listenRuntimeStateChangedMock: vi.fn(),
  listenAiStreamEventMock: vi.fn(),
}));

vi.mock('./lib/api', () => ({
  loadAppState: loadAppStateMock,
  saveSettings: saveSettingsMock,
  resetSettings: resetSettingsMock,
  clearCache: clearCacheMock,
  clearHistory: clearHistoryMock,
  deleteHistoryEntry: deleteHistoryEntryMock,
  listHistory: listHistoryMock,
  saveHistoryEntry: saveHistoryEntryMock,
  listRagImports: listRagImportsMock,
  deleteRagImport: deleteRagImportMock,
  clearRagIndex: clearRagIndexMock,
  rebuildRagIndex: rebuildRagIndexMock,
  regenerateWithLanguage: regenerateWithLanguageMock,
  showMainWindow: showMainWindowMock,
  hideMainWindow: hideMainWindowMock,
  toggleMainWindow: toggleMainWindowMock,
  listenGlobalShortcutTriggered: listenGlobalShortcutTriggeredMock,
  listenOpenSettings: listenOpenSettingsMock,
  listenRuntimeStateChanged: listenRuntimeStateChangedMock,
  listenAiStreamEvent: listenAiStreamEventMock,
}));

describe('App shell', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    listenGlobalShortcutTriggeredMock.mockResolvedValue(vi.fn());
    listenOpenSettingsMock.mockResolvedValue(vi.fn());
    listenRuntimeStateChangedMock.mockResolvedValue(vi.fn());
    listenAiStreamEventMock.mockResolvedValue(vi.fn());
    clearCacheMock.mockResolvedValue(undefined);
    clearHistoryMock.mockResolvedValue(undefined);
    deleteHistoryEntryMock.mockResolvedValue(true);
    listHistoryMock.mockResolvedValue([]);
    saveHistoryEntryMock.mockResolvedValue(null);
    listRagImportsMock.mockResolvedValue([]);
    deleteRagImportMock.mockResolvedValue(true);
    clearRagIndexMock.mockResolvedValue({
      clearedEmbeddingCount: 3,
      rebuiltEmbeddingCount: 0,
      clearedHistoryDocumentCount: 2,
      rebuiltHistoryDocumentCount: 0,
      clearedHistoryChunkCount: 4,
      skippedReason: null,
    });
    rebuildRagIndexMock.mockResolvedValue({
      clearedEmbeddingCount: 3,
      rebuiltEmbeddingCount: 5,
      clearedHistoryDocumentCount: 2,
      rebuiltHistoryDocumentCount: 1,
      clearedHistoryChunkCount: 4,
      skippedReason: null,
    });
    regenerateWithLanguageMock.mockResolvedValue(undefined);
  });

  it('renders the stage 1 shell in Chinese by default after loading the app snapshot', async () => {
    loadAppStateMock.mockResolvedValue({
      ...DEFAULT_APP_STATE,
      status: 'ready',
      settings: {
        ...DEFAULT_SETTINGS,
        providerModel: 'qwen-max',
      },
      settingsPath:
        'C:/Users/summus/AppData/Roaming/com.question-scan.desktop/settings.json',
    });

    render(<App />);

    expect(screen.getByText('桌面应用底座')).toBeInTheDocument();

    await waitFor(() => {
      expect(screen.getByLabelText('服务 Base URL')).toHaveValue(
        DEFAULT_SETTINGS.providerBaseUrl,
      );
    });

    expect(screen.getByLabelText('服务商名称')).toHaveValue(
      DEFAULT_SETTINGS.providerName,
    );
    expect(screen.getByLabelText('模型')).toHaveValue('qwen-max');
    expect(screen.getByLabelText('请求超时时间（秒）')).toHaveValue(
      DEFAULT_SETTINGS.requestTimeoutSeconds,
    );
    expect(screen.getByLabelText('API Key')).toHaveValue(
      DEFAULT_SETTINGS.providerApiKey,
    );
    expect(screen.getByLabelText(/启用流式输出/)).toBeChecked();
    expect(screen.getByLabelText('界面语言')).toHaveValue('zhCn');
    expect(screen.getByLabelText('全局快捷键')).toHaveValue('Ctrl+Shift+Q');
    expect(screen.getByText('启用全局快捷键')).toBeInTheDocument();
    expect(screen.getByLabelText(/启用本地知识增强/)).not.toBeChecked();
    expect(screen.getByLabelText('最大召回条数')).toHaveValue(5);
    expect(screen.getByText('运行状态快照')).toBeInTheDocument();
    expect(screen.getByText('设置')).toBeInTheDocument();
    expect(screen.getByTestId('result-empty-state')).toBeInTheDocument();
  });

  it('shows the crop overlay when the backend enters manual selection mode', async () => {
    loadAppStateMock.mockResolvedValue({
      ...DEFAULT_APP_STATE,
      status: 'ready',
      screenshotState: 'selecting',
      settings: DEFAULT_SETTINGS,
    });

    render(<App />);

    await waitFor(() => {
      expect(screen.getByText('手动框选')).toBeInTheDocument();
    });

    expect(screen.getByText('拖拽开始框选')).toBeInTheDocument();
  });

  it('cancels manual crop selection and returns the capture state to idle', async () => {
    loadAppStateMock.mockResolvedValue({
      ...DEFAULT_APP_STATE,
      status: 'ready',
      screenshotState: 'selecting',
      settings: DEFAULT_SETTINGS,
    });

    render(<App />);

    await waitFor(() => {
      expect(screen.getByText('手动框选')).toBeInTheDocument();
    });

    fireEvent.click(screen.getByRole('button', { name: '取消' }));

    expect(screen.queryByText('手动框选')).not.toBeInTheDocument();
    expect(screen.getByText('已取消手动框选。')).toBeInTheDocument();
    expect(screen.getByTestId('result-empty-state')).toBeInTheDocument();
  });

  it('retries automatic recognition from manual fallback', async () => {
    loadAppStateMock.mockResolvedValue({
      ...DEFAULT_APP_STATE,
      status: 'ready',
      screenshotState: 'selecting',
      settings: DEFAULT_SETTINGS,
    });

    render(<App />);

    await waitFor(() => {
      expect(screen.getByText('手动框选')).toBeInTheDocument();
    });

    fireEvent.click(screen.getByRole('button', { name: '重新自动识别' }));

    expect(screen.queryByText('手动框选')).not.toBeInTheDocument();
    expect(
      screen.getByText('已清空手动选区，重新进入自动识别状态。'),
    ).toBeInTheDocument();
    expect(screen.getByTestId('result-empty-state')).toBeInTheDocument();
  });

  it('confirms the selected manual crop for the next crop step', async () => {
    loadAppStateMock.mockResolvedValue({
      ...DEFAULT_APP_STATE,
      status: 'ready',
      screenshotState: 'selecting',
      settings: DEFAULT_SETTINGS,
    });

    render(<App />);

    await waitFor(() => {
      expect(screen.getByText('手动框选')).toBeInTheDocument();
    });

    const overlay = screen.getByLabelText('手动框选');
    fireEvent.pointerDown(overlay, {
      button: 0,
      clientX: 80,
      clientY: 96,
      pointerId: 1,
    });
    fireEvent.pointerUp(overlay, {
      clientX: 300,
      clientY: 216,
      pointerId: 1,
    });
    fireEvent.click(screen.getByRole('button', { name: '确认裁剪' }));

    expect(screen.queryByText('手动框选')).not.toBeInTheDocument();
    expect(
      screen.getByText(
        '已确认裁剪区域，后续会使用这个选区生成高清裁剪图。 (220 x 120)',
      ),
    ).toBeInTheDocument();
    expect(screen.getByTestId('result-empty-state')).toBeInTheDocument();
  });

  it('switches the visible shell language to English and persists the locale', async () => {
    loadAppStateMock.mockResolvedValue({
      ...DEFAULT_APP_STATE,
      status: 'ready',
      settings: DEFAULT_SETTINGS,
    });
    saveSettingsMock.mockImplementation(async (settings) => ({
      ...DEFAULT_APP_STATE,
      status: 'ready',
      settings,
    }));

    render(<App />);

    await waitFor(() => {
      expect(screen.getByLabelText('界面语言')).toHaveValue('zhCn');
    });

    fireEvent.change(screen.getByLabelText('界面语言'), {
      target: { value: 'enUs' },
    });

    expect(screen.getByText('Desktop shell')).toBeInTheDocument();
    expect(screen.getByLabelText('Interface language')).toHaveValue('enUs');

    fireEvent.click(screen.getByRole('button', { name: 'Save settings' }));

    await waitFor(() => {
      expect(saveSettingsMock).toHaveBeenCalledWith({
        ...DEFAULT_SETTINGS,
        uiLocale: 'enUs',
      });
    });
  });

  it('persists a disabled custom shortcut setting', async () => {
    loadAppStateMock.mockResolvedValue({
      ...DEFAULT_APP_STATE,
      status: 'ready',
      settings: DEFAULT_SETTINGS,
    });
    saveSettingsMock.mockImplementation(async (settings) => ({
      ...DEFAULT_APP_STATE,
      status: 'ready',
      settings,
    }));

    render(<App />);

    await waitFor(() => {
      expect(screen.getByLabelText('全局快捷键')).toHaveValue('Ctrl+Shift+Q');
    });

    fireEvent.change(screen.getByLabelText('全局快捷键'), {
      target: { value: 'Alt+Shift+S' },
    });
    fireEvent.click(screen.getByLabelText(/启用全局快捷键/));
    fireEvent.click(screen.getByRole('button', { name: '保存设置' }));

    await waitFor(() => {
      expect(saveSettingsMock).toHaveBeenCalledWith({
        ...DEFAULT_SETTINGS,
        globalShortcut: 'Alt+Shift+S',
        globalShortcutEnabled: false,
      });
    });
  });

  it('persists provider settings while the backend redacts the API key snapshot', async () => {
    loadAppStateMock.mockResolvedValue({
      ...DEFAULT_APP_STATE,
      status: 'ready',
      settings: DEFAULT_SETTINGS,
    });
    saveSettingsMock.mockImplementation(async (settings) => ({
      ...DEFAULT_APP_STATE,
      status: 'ready',
      settings: {
        ...settings,
        providerApiKey: '',
      },
    }));

    render(<App />);

    await waitFor(() => {
      expect(screen.getByLabelText('服务商名称')).toHaveValue(
        DEFAULT_SETTINGS.providerName,
      );
    });

    fireEvent.change(screen.getByLabelText('服务商名称'), {
      target: { value: 'Example AI' },
    });
    fireEvent.change(screen.getByLabelText('API Key'), {
      target: { value: 'sk-test-123' },
    });
    fireEvent.change(screen.getByLabelText('请求超时时间（秒）'), {
      target: { value: '90' },
    });
    fireEvent.click(screen.getByLabelText(/启用流式输出/));
    fireEvent.click(screen.getByRole('button', { name: '保存设置' }));

    await waitFor(() => {
      expect(saveSettingsMock).toHaveBeenCalledWith({
        ...DEFAULT_SETTINGS,
        providerName: 'Example AI',
        providerApiKey: 'sk-test-123',
        requestTimeoutSeconds: 90,
        streamingEnabled: false,
      });
    });
    expect(screen.getByLabelText('API Key')).toHaveValue('');
  });

  it('persists local RAG settings and clamps max recall items from the settings page', async () => {
    loadAppStateMock.mockResolvedValue({
      ...DEFAULT_APP_STATE,
      status: 'ready',
      settings: DEFAULT_SETTINGS,
    });
    saveSettingsMock.mockImplementation(async (settings) => ({
      ...DEFAULT_APP_STATE,
      status: 'ready',
      settings,
    }));

    render(<App />);

    await waitFor(() => {
      expect(screen.getByLabelText(/启用本地知识增强/)).not.toBeChecked();
    });

    fireEvent.click(screen.getByLabelText(/启用本地知识增强/));
    fireEvent.click(screen.getByLabelText(/历史入库/));
    fireEvent.click(screen.getByLabelText(/用户笔记检索/));
    fireEvent.click(screen.getByLabelText(/代码模板检索/));
    fireEvent.change(screen.getByLabelText('最大召回条数'), {
      target: { value: '999' },
    });
    fireEvent.click(screen.getByRole('button', { name: '保存设置' }));

    await waitFor(() => {
      expect(saveSettingsMock).toHaveBeenCalledWith({
        ...DEFAULT_SETTINGS,
        localRagEnabled: true,
        ragHistoryIndexingEnabled: true,
        ragUserNotesRetrievalEnabled: false,
        ragCodeTemplatesRetrievalEnabled: false,
        ragMaxRecallItems: 20,
      });
    });
  });

  it('runs RAG index clear and rebuild actions from data management controls', async () => {
    loadAppStateMock.mockResolvedValue({
      ...DEFAULT_APP_STATE,
      status: 'ready',
      settings: {
        ...DEFAULT_SETTINGS,
        localRagEnabled: true,
      },
    });

    render(<App />);

    await waitFor(() => {
      expect(
        screen.getByRole('button', { name: '清除 RAG 索引' }),
      ).toBeInTheDocument();
    });

    fireEvent.click(screen.getByRole('button', { name: '清除 RAG 索引' }));

    await waitFor(() => {
      expect(clearRagIndexMock).toHaveBeenCalledTimes(1);
    });
    expect(screen.getByText(/RAG 索引已清除/)).toBeInTheDocument();
    expect(screen.getByText(/embedding 清除 3 条/)).toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: '重建 RAG 索引' }));

    await waitFor(() => {
      expect(rebuildRagIndexMock).toHaveBeenCalledTimes(1);
    });
    expect(screen.getByText(/RAG 索引重建完成/)).toBeInTheDocument();
    expect(screen.getByText(/重建 5 条/)).toBeInTheDocument();
  });

  it('loads and deletes a single user RAG imported document', async () => {
    loadAppStateMock.mockResolvedValue({
      ...DEFAULT_APP_STATE,
      status: 'ready',
      settings: {
        ...DEFAULT_SETTINGS,
        localRagEnabled: true,
      },
    });
    listRagImportsMock.mockResolvedValue([
      {
        id: 'ragimp_123',
        sourceName: 'notes.md',
        sourceUri: 'C:/notes/notes.md',
        sourceFormat: 'markdown',
        kind: 'solution',
        title: 'Two Sum note',
        text: '# Two Sum\n\nUse a hash map.',
        language: 'cpp20',
        platform: 'leetcode',
        tags: ['array'],
        algorithmTags: ['hash-table'],
        importedAt: '2026-01-01T00:00:00Z',
        updatedAt: '2026-01-01T00:00:00Z',
        deletedAt: null,
      },
    ]);

    render(<App />);

    await waitFor(() => {
      expect(
        screen.getByRole('button', { name: '刷新导入资料' }),
      ).toBeInTheDocument();
    });

    fireEvent.click(screen.getByRole('button', { name: '刷新导入资料' }));

    await waitFor(() => {
      expect(listRagImportsMock).toHaveBeenCalledTimes(1);
    });
    expect(screen.getByText('Two Sum note')).toBeInTheDocument();

    fireEvent.click(
      screen.getByRole('button', {
        name: '删除导入资料: Two Sum note',
      }),
    );

    await waitFor(() => {
      expect(deleteRagImportMock).toHaveBeenCalledWith('ragimp_123');
    });
    expect(screen.queryByText('Two Sum note')).not.toBeInTheDocument();
    expect(screen.getByText('导入资料已删除。')).toBeInTheDocument();
  });

  it('disables history indexing immediately from privacy controls without disabling normal history', async () => {
    loadAppStateMock.mockResolvedValue({
      ...DEFAULT_APP_STATE,
      status: 'ready',
      settings: {
        ...DEFAULT_SETTINGS,
        saveHistory: true,
        localRagEnabled: true,
        ragHistoryIndexingEnabled: true,
      },
    });
    saveSettingsMock.mockImplementation(async (settings) => ({
      ...DEFAULT_APP_STATE,
      status: 'ready',
      settings,
    }));

    render(<App />);

    await waitFor(() => {
      expect(
        screen.getByRole('button', { name: '关闭历史入库' }),
      ).toBeEnabled();
    });

    fireEvent.click(screen.getByRole('button', { name: '关闭历史入库' }));

    await waitFor(() => {
      expect(saveSettingsMock).toHaveBeenCalledWith({
        ...DEFAULT_SETTINGS,
        saveHistory: true,
        localRagEnabled: true,
        ragHistoryIndexingEnabled: false,
      });
    });
    expect(screen.getByLabelText(/保存历史/)).toBeChecked();
    expect(screen.getByText('历史入库已关闭。')).toBeInTheDocument();
  });

  it('shows backend shortcut conflict messages returned from Rust', async () => {
    loadAppStateMock.mockResolvedValue({
      ...DEFAULT_APP_STATE,
      status: 'ready',
      settings: DEFAULT_SETTINGS,
    });
    saveSettingsMock.mockRejectedValue({
      code: 'globalShortcutRegistrationFailed',
      message:
        'The global shortcut `Ctrl+Shift+Q` could not be registered. It may already be used by the system or another app.',
    });

    render(<App />);

    await waitFor(() => {
      expect(screen.getByLabelText('全局快捷键')).toHaveValue('Ctrl+Shift+Q');
    });

    fireEvent.change(screen.getByLabelText('模型'), {
      target: { value: 'changed-model' },
    });
    fireEvent.click(screen.getByRole('button', { name: '保存设置' }));

    await waitFor(() => {
      expect(screen.getByText(/could not be registered/)).toBeInTheDocument();
    });
  });

  it('saves a history RAG snapshot when streaming completes and history is enabled', async () => {
    let streamHandler:
      | ((event: {
          payload: { type: 'chunk'; content: string } | { type: 'done' };
        }) => void)
      | null = null;
    loadAppStateMock.mockResolvedValue({
      ...DEFAULT_APP_STATE,
      status: 'ready',
      settings: {
        ...DEFAULT_SETTINGS,
        saveHistory: true,
        platformFormat: 'leetcode',
        providerModel: 'gpt-4o-mini',
      },
    });
    listenAiStreamEventMock.mockImplementation(async (handler) => {
      streamHandler = handler;
      return vi.fn();
    });
    saveHistoryEntryMock.mockResolvedValue({
      id: 'hist_123',
      timestamp: '2026-01-01T00:00:00Z',
      recognizedTitle: null,
      recognizedText: null,
      language: 'cpp20',
      platform: 'leetcode',
      model: 'gpt-4o-mini',
      result: '解题策略:\nUse a hash map.',
      userNote: null,
      tags: [],
      algorithmTags: [],
    });

    render(<App />);

    await waitFor(() => {
      expect(streamHandler).not.toBeNull();
    });
    await waitFor(() => {
      expect(screen.getByLabelText(/保存历史/)).toBeChecked();
    });

    act(() => {
      streamHandler?.({
        payload: { type: 'chunk', content: '解题策略:\nUse a hash map.' },
      });
      streamHandler?.({ payload: { type: 'done' } });
    });

    await waitFor(() => {
      expect(saveHistoryEntryMock).toHaveBeenCalledWith({
        recognizedTitle: null,
        recognizedText: null,
        language: 'cpp20',
        platform: 'leetcode',
        model: 'gpt-4o-mini',
        result: '解题策略:\nUse a hash map.',
        userNote: null,
        tags: [],
        algorithmTags: [],
      });
    });
  });

  it('shows RAG context emitted by the screenshot pipeline in the result panel', async () => {
    let streamHandler:
      | ((event: {
          payload:
            | {
                type: 'ragContext';
                items: Array<{
                  id: string;
                  sourceType: 'leetcodeIndex';
                  sourceId: string;
                  title: string;
                  snippet: string;
                  score: number;
                  kind: 'metadata';
                  language: null;
                  platform: 'leetcode';
                  tags: string[];
                  algorithmTags: string[];
                  usedInPrompt: boolean;
                  skippedReason: null;
                  tokenEstimate: number;
                  reason: string;
                }>;
                usedItemCount: number;
                tokenEstimate: number;
                skippedReason: null;
              }
            | { type: 'chunk'; content: string };
        }) => void)
      | null = null;
    loadAppStateMock.mockResolvedValue({
      ...DEFAULT_APP_STATE,
      status: 'ready',
      settings: {
        ...DEFAULT_SETTINGS,
        localRagEnabled: true,
      },
    });
    listenAiStreamEventMock.mockImplementation(async (handler) => {
      streamHandler = handler;
      return vi.fn();
    });

    render(<App />);

    await waitFor(() => {
      expect(streamHandler).not.toBeNull();
    });

    act(() => {
      streamHandler?.({
        payload: {
          type: 'ragContext',
          items: [
            {
              id: 'problemidxchunk_leetcode_1',
              sourceType: 'leetcodeIndex',
              sourceId: 'two-sum',
              title: 'Two Sum',
              snippet: 'LeetCode lightweight metadata only.',
              score: 0.88,
              kind: 'metadata',
              language: null,
              platform: 'leetcode',
              tags: ['Array'],
              algorithmTags: ['hash-table'],
              usedInPrompt: true,
              skippedReason: null,
              tokenEstimate: 20,
              reason: 'title;algorithmTags',
            },
          ],
          usedItemCount: 1,
          tokenEstimate: 20,
          skippedReason: null,
        },
      });
      streamHandler?.({
        payload: { type: 'chunk', content: '解题策略:\nUse a hash map.' },
      });
    });

    expect(screen.getByText('本地召回上下文')).toBeInTheDocument();
    expect(screen.getByText('Two Sum')).toBeInTheDocument();
    expect(screen.getByText('已注入')).toBeInTheDocument();
  });
});
