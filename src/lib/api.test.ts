import { beforeEach, describe, expect, it, vi } from 'vitest';
import {
  BACKEND_COMMANDS,
  buildRagPromptContext,
  clearRagIndex,
  deleteRagImport,
  FRONTEND_EVENTS,
  hideMainWindow,
  importRagDocuments,
  listenAiStreamEvent,
  listenGlobalShortcutTriggered,
  listenOpenSettings,
  listenRuntimeStateChanged,
  listLeetcodeProblemIndex,
  listRagImports,
  loadAppState,
  rebuildRagIndex,
  resetSettings,
  saveHistoryEntry,
  saveSettings,
  searchRagContext,
  selectRagTemplateContext,
  sendAiRequest,
  setTrayStatus,
  showMainWindow,
  toggleMainWindow,
} from './api';
import { DEFAULT_SETTINGS } from './types';

const { invokeMock, listenMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
  listenMock: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: listenMock,
}));

describe('Tauri API wrappers', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('keeps the frontend command registry aligned with backend command names', () => {
    expect(Object.values(BACKEND_COMMANDS)).toEqual([
      'load_app_state',
      'save_settings',
      'reset_settings',
      'show_main_window',
      'hide_main_window',
      'toggle_main_window',
      'set_tray_status',
      'send_ai_request',
      'regenerate_with_language',
      'clear_cache',
      'list_history',
      'save_history_entry',
      'delete_history_entry',
      'clear_history',
      'list_leetcode_problem_index',
      'import_rag_documents',
      'list_rag_imports',
      'delete_rag_import',
      'clear_rag_index',
      'rebuild_rag_index',
      'search_rag_context',
      'select_rag_template_context',
      'build_rag_prompt_context',
    ]);
  });

  it('maps the load command to the backend entry point', async () => {
    invokeMock.mockResolvedValue({ status: 'ready' });

    await loadAppState();

    expect(invokeMock).toHaveBeenCalledWith('load_app_state');
  });

  it('maps the settings command payloads', async () => {
    invokeMock.mockResolvedValue({ status: 'ready' });

    await saveSettings(DEFAULT_SETTINGS);
    await resetSettings();
    await showMainWindow();
    await hideMainWindow();
    await toggleMainWindow();
    await setTrayStatus('generating');
    await sendAiRequest(
      'test instruction',
      new Uint8Array([1, 2, 3]),
      'image/png',
    );
    await saveHistoryEntry({
      recognizedTitle: 'Two Sum',
      recognizedText: 'Given nums and target.',
      language: 'cpp20',
      platform: 'leetcode',
      model: 'gpt-4o-mini',
      result: '解题策略:\nUse a hash map.',
      userNote: null,
      tags: ['array'],
      algorithmTags: ['hash-table'],
    });
    await listLeetcodeProblemIndex();
    await importRagDocuments({
      sourceName: 'notes.md',
      sourceUri: 'C:/notes/notes.md',
      format: 'markdown',
      kind: 'solution',
      content: '# Two Sum\n\nUse a hash map.',
    });
    await listRagImports();
    await deleteRagImport('ragimp_123');
    await clearRagIndex();
    await rebuildRagIndex();
    await searchRagContext({
      recognizedTitle: 'Two Sum',
      recognizedText: 'Given nums and target.',
      examples: ['nums = [2,7,11,15], target = 9'],
      constraints: ['2 <= nums.length <= 10^4'],
      tags: ['array'],
      algorithmTags: ['hash-table'],
      targetLanguage: 'cpp20',
      platform: 'leetcode',
      topK: 5,
      minScore: 0.12,
    });
    await selectRagTemplateContext({
      targetLanguage: null,
      platform: 'leetcode',
      tags: [],
      algorithmTags: ['dfs', 'graph'],
      similarItems: [
        {
          chunkId: 'problemidxchunk_leetcode_200',
          documentId: 'leetcode_200',
          sourceType: 'leetcodeIndex',
          sourceId: 'number-of-islands',
          title: 'Number of Islands',
          snippet: 'LeetCode metadata only.',
          score: 0.82,
          kind: 'metadata',
          problemType: 'graph',
          language: null,
          platform: 'leetcode',
          tags: ['Graph'],
          algorithmTags: ['dfs', 'graph'],
          reason: 'vector=0.700;algorithmTags',
        },
      ],
      maxSolutionModes: 3,
      maxTemplates: 2,
    });
    await buildRagPromptContext({
      recognizedTitle: 'Two Sum',
      recognizedText: 'Given nums and target.',
      targetLanguage: 'cpp20',
      platform: 'leetcode',
      searchResults: [
        {
          chunkId: 'ragimpchunk_123_0',
          documentId: 'ragimp_123',
          sourceType: 'userImport',
          sourceId: 'ragimp_123',
          title: 'Two Sum note',
          snippet: 'Use a hash table to find complements.',
          score: 0.87,
          kind: 'solution',
          problemType: null,
          language: 'cpp20',
          platform: 'leetcode',
          tags: ['array'],
          algorithmTags: ['hash-table'],
          reason: 'vector=0.730;algorithmTags',
        },
      ],
      templateContext: {
        targetLanguage: 'cpp20',
        platform: 'leetcode',
        solutionModes: [
          {
            id: 'hash-table-lookup',
            title: 'Hash Table Lookup',
            description: 'Use a map for O(1) complement lookup.',
            problemType: 'hash-table',
            algorithmTags: ['hash-table'],
            score: 0.9,
            reason: 'algorithmTags',
          },
        ],
        templates: [],
        skippedReason: null,
      },
      maxItems: 3,
      maxContextTokens: 1200,
      minScore: 0.25,
    });

    expect(invokeMock).toHaveBeenNthCalledWith(1, 'save_settings', {
      settings: DEFAULT_SETTINGS,
    });
    expect(invokeMock).toHaveBeenNthCalledWith(2, 'reset_settings');
    expect(invokeMock).toHaveBeenNthCalledWith(3, 'show_main_window');
    expect(invokeMock).toHaveBeenNthCalledWith(4, 'hide_main_window');
    expect(invokeMock).toHaveBeenNthCalledWith(5, 'toggle_main_window');
    expect(invokeMock).toHaveBeenNthCalledWith(6, 'set_tray_status', {
      status: 'generating',
    });
    expect(invokeMock).toHaveBeenNthCalledWith(7, 'send_ai_request', {
      instruction: 'test instruction',
      imageBytes: [1, 2, 3],
      imageMimeType: 'image/png',
    });
    expect(invokeMock).toHaveBeenNthCalledWith(8, 'save_history_entry', {
      request: {
        recognizedTitle: 'Two Sum',
        recognizedText: 'Given nums and target.',
        language: 'cpp20',
        platform: 'leetcode',
        model: 'gpt-4o-mini',
        result: '解题策略:\nUse a hash map.',
        userNote: null,
        tags: ['array'],
        algorithmTags: ['hash-table'],
      },
    });
    expect(invokeMock).toHaveBeenNthCalledWith(
      9,
      'list_leetcode_problem_index',
    );
    expect(invokeMock).toHaveBeenNthCalledWith(10, 'import_rag_documents', {
      request: {
        sourceName: 'notes.md',
        sourceUri: 'C:/notes/notes.md',
        format: 'markdown',
        kind: 'solution',
        content: '# Two Sum\n\nUse a hash map.',
      },
    });
    expect(invokeMock).toHaveBeenNthCalledWith(11, 'list_rag_imports');
    expect(invokeMock).toHaveBeenNthCalledWith(12, 'delete_rag_import', {
      id: 'ragimp_123',
    });
    expect(invokeMock).toHaveBeenNthCalledWith(13, 'clear_rag_index');
    expect(invokeMock).toHaveBeenNthCalledWith(14, 'rebuild_rag_index');
    expect(invokeMock).toHaveBeenNthCalledWith(15, 'search_rag_context', {
      request: {
        recognizedTitle: 'Two Sum',
        recognizedText: 'Given nums and target.',
        examples: ['nums = [2,7,11,15], target = 9'],
        constraints: ['2 <= nums.length <= 10^4'],
        tags: ['array'],
        algorithmTags: ['hash-table'],
        targetLanguage: 'cpp20',
        platform: 'leetcode',
        topK: 5,
        minScore: 0.12,
      },
    });
    expect(invokeMock).toHaveBeenNthCalledWith(
      16,
      'select_rag_template_context',
      {
        request: {
          targetLanguage: null,
          platform: 'leetcode',
          tags: [],
          algorithmTags: ['dfs', 'graph'],
          similarItems: [
            {
              chunkId: 'problemidxchunk_leetcode_200',
              documentId: 'leetcode_200',
              sourceType: 'leetcodeIndex',
              sourceId: 'number-of-islands',
              title: 'Number of Islands',
              snippet: 'LeetCode metadata only.',
              score: 0.82,
              kind: 'metadata',
              problemType: 'graph',
              language: null,
              platform: 'leetcode',
              tags: ['Graph'],
              algorithmTags: ['dfs', 'graph'],
              reason: 'vector=0.700;algorithmTags',
            },
          ],
          maxSolutionModes: 3,
          maxTemplates: 2,
        },
      },
    );
    expect(invokeMock).toHaveBeenNthCalledWith(17, 'build_rag_prompt_context', {
      request: {
        recognizedTitle: 'Two Sum',
        recognizedText: 'Given nums and target.',
        targetLanguage: 'cpp20',
        platform: 'leetcode',
        searchResults: [
          {
            chunkId: 'ragimpchunk_123_0',
            documentId: 'ragimp_123',
            sourceType: 'userImport',
            sourceId: 'ragimp_123',
            title: 'Two Sum note',
            snippet: 'Use a hash table to find complements.',
            score: 0.87,
            kind: 'solution',
            problemType: null,
            language: 'cpp20',
            platform: 'leetcode',
            tags: ['array'],
            algorithmTags: ['hash-table'],
            reason: 'vector=0.730;algorithmTags',
          },
        ],
        templateContext: {
          targetLanguage: 'cpp20',
          platform: 'leetcode',
          solutionModes: [
            {
              id: 'hash-table-lookup',
              title: 'Hash Table Lookup',
              description: 'Use a map for O(1) complement lookup.',
              problemType: 'hash-table',
              algorithmTags: ['hash-table'],
              score: 0.9,
              reason: 'algorithmTags',
            },
          ],
          templates: [],
          skippedReason: null,
        },
        maxItems: 3,
        maxContextTokens: 1200,
        minScore: 0.25,
      },
    });
  });

  it('registers frontend listeners for backend runtime events', async () => {
    const unlisten = vi.fn();
    listenMock.mockResolvedValue(unlisten);
    const handler = vi.fn();

    await listenGlobalShortcutTriggered(handler);
    await listenOpenSettings(handler);
    await listenRuntimeStateChanged(handler);
    await listenAiStreamEvent(handler);

    expect(listenMock).toHaveBeenNthCalledWith(
      1,
      FRONTEND_EVENTS.globalShortcutTriggered,
      handler,
    );
    expect(listenMock).toHaveBeenNthCalledWith(
      2,
      FRONTEND_EVENTS.openSettings,
      handler,
    );
    expect(listenMock).toHaveBeenNthCalledWith(
      3,
      FRONTEND_EVENTS.runtimeStateChanged,
      handler,
    );
    expect(listenMock).toHaveBeenNthCalledWith(
      4,
      FRONTEND_EVENTS.aiStreamEvent,
      handler,
    );
  });
});
