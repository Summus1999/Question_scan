import { describe, expect, it } from 'vitest';
import type {
  AiStreamEventPayload,
  HistoryEntry,
  ProblemIndexEntry,
  QuestionRecognitionResult,
  RagImportedDocument,
  RagImportRequest,
  RagIndexMaintenanceResult,
  RagPromptContextRequest,
  RagPromptContextResponse,
  RagSearchRequest,
  RagSearchResponse,
  RagTemplateSelectionRequest,
  RagTemplateSelectionResponse,
  RecognitionConfidenceRoute,
  SaveHistoryEntryRequest,
} from './types';
import {
  AI_RESULT_STATES,
  APP_STATUSES,
  DEFAULT_APP_STATE,
  DEFAULT_RAG_MAX_RECALL_ITEMS,
  DEFAULT_RECOGNITION_CONFIDENCE_THRESHOLDS,
  DEFAULT_SETTINGS,
  LANGUAGE_IDS,
  LANGUAGE_OPTIONS,
  OUTPUT_SPEED_OPTIONS,
  OUTPUT_SPEEDS,
  PROBLEM_DIFFICULTIES,
  PROBLEM_PLATFORMS,
  PROBLEM_TYPES,
  RAG_IMPORT_FORMATS,
  RAG_IMPORT_KINDS,
  RAG_PROMPT_CONTEXT_KINDS,
  RAG_PROMPT_CONTEXT_SOURCE_TYPES,
  RAG_SEARCH_CHUNK_KINDS,
  RAG_SEARCH_SOURCE_TYPES,
  RECOGNITION_CONFIDENCE_ROUTES,
  SCREENSHOT_STATES,
  THEME_PREFERENCES,
  TRAY_STATUSES,
  UI_LOCALES,
} from './types';

describe('shared frontend types', () => {
  it('keeps language and output option values aligned with their shared ids', () => {
    expect(LANGUAGE_OPTIONS.map((option) => option.value).sort()).toEqual(
      [...LANGUAGE_IDS].sort(),
    );
    expect(OUTPUT_SPEED_OPTIONS.map((option) => option.value)).toEqual([
      ...OUTPUT_SPEEDS,
    ]);
  });

  it('keeps stage 1 defaults aligned with backend settings defaults', () => {
    expect(DEFAULT_SETTINGS).toEqual({
      providerName: 'OpenAI-compatible',
      providerBaseUrl: 'https://api.openai.com/v1',
      providerApiKey: '',
      providerModel: 'gpt-4o-mini',
      requestTimeoutSeconds: 60,
      streamingEnabled: true,
      defaultLanguage: 'cpp20',
      platformFormat: 'acm',
      outputSpeed: 'normal',
      customCharactersPerSecond: 24,
      globalShortcut: 'Ctrl+Shift+Q',
      globalShortcutEnabled: true,
      screenshotMaxLongEdge: 1920,
      screenshotJpegQuality: 85,
      saveHistory: false,
      launchToTray: false,
      localRagEnabled: false,
      ragHistoryIndexingEnabled: false,
      ragUserNotesRetrievalEnabled: true,
      ragCodeTemplatesRetrievalEnabled: true,
      ragSimilarProblemsEnabled: true,
      ragMaxRecallItems: DEFAULT_RAG_MAX_RECALL_ITEMS,
      theme: 'system',
      uiLocale: 'zhCn',
    });
  });

  it('exposes stable runtime state values for the shell and future workflows', () => {
    expect(APP_STATUSES).toEqual(['loading', 'ready', 'warning', 'error']);
    expect(TRAY_STATUSES).toEqual([
      'idle',
      'screenshot',
      'recognizing',
      'generating',
      'complete',
      'failed',
    ]);
    expect(SCREENSHOT_STATES).toEqual([
      'idle',
      'capturing',
      'cropping',
      'selecting',
      'ready',
      'failed',
    ]);
    expect(AI_RESULT_STATES).toEqual([
      'idle',
      'loading',
      'streaming',
      'complete',
      'failed',
    ]);
    expect(THEME_PREFERENCES).toEqual(['system', 'light', 'dark']);
    expect(UI_LOCALES).toEqual(['zhCn', 'enUs']);
    expect(DEFAULT_APP_STATE).toMatchObject({
      status: 'loading',
      trayStatus: 'idle',
      screenshotState: 'idle',
      aiResultState: 'idle',
      windowVisible: true,
      globalShortcutRegistered: false,
      globalShortcutError: null,
      globalShortcutTriggerCount: 0,
    });
  });

  it('keeps the recognition result contract aligned with the backend handoff shape', () => {
    const recognition = {
      boundingBox: {
        x: 48,
        y: 96,
        width: 720,
        height: 420,
      },
      confidence: 0.91,
      title: 'Longest Substring Without Repeating Characters',
      questionText:
        'Given a string s, find the length of the longest substring without repeating characters.',
      reason:
        'The text block groups the title, statement, and examples into one algorithm problem panel.',
    } satisfies QuestionRecognitionResult;

    expect(recognition.boundingBox).toEqual({
      x: 48,
      y: 96,
      width: 720,
      height: 420,
    });
    expect(recognition.title).toBe(
      'Longest Substring Without Repeating Characters',
    );
    expect(recognition.questionText).toContain(
      'longest substring without repeating characters',
    );
    expect(recognition.reason).toContain('algorithm problem panel');
    expect(Object.keys(recognition)).toEqual([
      'boundingBox',
      'confidence',
      'title',
      'questionText',
      'reason',
    ]);
  });

  it('keeps the recognition confidence routes and defaults aligned with the backend contract', () => {
    const route = 'needsConfirmation' satisfies RecognitionConfidenceRoute;

    expect(RECOGNITION_CONFIDENCE_ROUTES).toEqual([
      'autoAccept',
      'needsConfirmation',
      'manualFallback',
    ]);
    expect(DEFAULT_RECOGNITION_CONFIDENCE_THRESHOLDS).toEqual({
      autoAcceptMinConfidence: 0.85,
      confirmationMinConfidence: 0.55,
    });
    expect(route).toBe('needsConfirmation');
  });

  it('keeps the AI stream event payload shapes aligned with the backend event contract', () => {
    const chunk = { type: 'chunk' as const, content: 'hello' };
    const ragContext = {
      type: 'ragContext' as const,
      items: [
        {
          id: 'problemidxchunk_leetcode_1',
          sourceType: 'leetcodeIndex' as const,
          sourceId: 'two-sum',
          title: 'Two Sum',
          snippet: 'LeetCode lightweight metadata only.',
          score: 0.87,
          kind: 'metadata' as const,
          language: null,
          platform: 'leetcode' as const,
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
    };
    const done = { type: 'done' as const };
    const error = {
      type: 'error' as const,
      code: 'invalidApiKey',
      message: 'The API key was rejected.',
    };

    const payloads: AiStreamEventPayload[] = [chunk, ragContext, done, error];

    expect(chunk).toEqual({ type: 'chunk', content: 'hello' });
    expect(payloads.map((payload) => payload.type)).toEqual([
      'chunk',
      'ragContext',
      'done',
      'error',
    ]);
    expect(ragContext).toMatchObject({
      type: 'ragContext',
      usedItemCount: 1,
    });
    expect(ragContext.items[0].sourceType).toBe('leetcodeIndex');
    expect(done).toEqual({ type: 'done' });
    expect(error).toMatchObject({
      type: 'error',
      code: 'invalidApiKey',
    });
  });

  it('keeps the lightweight problem index contract metadata-only', () => {
    const entry = {
      id: 'leetcode_200',
      platform: 'leetcode',
      problemNumber: '200',
      slug: 'number-of-islands',
      title: 'Number of Islands',
      difficulty: 'medium',
      tags: ['Array', 'Depth-First Search', 'Graph'],
      algorithmTags: ['array', 'dfs', 'graph'],
      problemType: 'graph',
      source: 'leetcode-lightweight-seed-v1',
    } satisfies ProblemIndexEntry;

    expect(PROBLEM_PLATFORMS).toEqual(['leetcode']);
    expect(PROBLEM_DIFFICULTIES).toEqual(['easy', 'medium', 'hard', 'unknown']);
    expect(PROBLEM_TYPES).toContain('dynamic-programming');
    expect(PROBLEM_TYPES).toContain('graph');
    expect(Object.keys(entry)).toEqual([
      'id',
      'platform',
      'problemNumber',
      'slug',
      'title',
      'difficulty',
      'tags',
      'algorithmTags',
      'problemType',
      'source',
    ]);
    expect(entry).not.toHaveProperty('problemStatement');
    expect(entry).not.toHaveProperty('solution');
  });

  it('keeps the user RAG import contracts aligned with backend commands', () => {
    const request = {
      sourceName: 'notes.md',
      sourceUri: 'C:/notes/notes.md',
      format: 'markdown',
      kind: 'solution',
      content: '# Two Sum\n\nUse a hash map.',
    } satisfies RagImportRequest;
    const imported = {
      id: 'ragimp_123',
      sourceName: 'notes.md',
      sourceUri: 'C:/notes/notes.md',
      sourceFormat: 'markdown',
      kind: 'solution',
      title: 'Two Sum',
      text: '# Two Sum\n\nUse a hash map.',
      language: 'cpp20',
      platform: 'leetcode',
      tags: ['array', 'hash-table'],
      algorithmTags: ['hash-table'],
      importedAt: '2026-01-01T00:00:00Z',
      updatedAt: '2026-01-01T00:00:00Z',
      deletedAt: null,
    } satisfies RagImportedDocument;

    expect(RAG_IMPORT_FORMATS).toEqual(['markdown', 'json', 'csv']);
    expect(RAG_IMPORT_KINDS).toEqual(['solution', 'note', 'template']);
    expect(Object.keys(request)).toEqual([
      'sourceName',
      'sourceUri',
      'format',
      'kind',
      'content',
    ]);
    expect(Object.keys(imported)).toEqual([
      'id',
      'sourceName',
      'sourceUri',
      'sourceFormat',
      'kind',
      'title',
      'text',
      'language',
      'platform',
      'tags',
      'algorithmTags',
      'importedAt',
      'updatedAt',
      'deletedAt',
    ]);
  });

  it('keeps RAG index maintenance result contracts aligned with backend commands', () => {
    const result = {
      clearedEmbeddingCount: 3,
      rebuiltEmbeddingCount: 5,
      clearedHistoryDocumentCount: 2,
      rebuiltHistoryDocumentCount: 1,
      clearedHistoryChunkCount: 4,
      skippedReason: null,
    } satisfies RagIndexMaintenanceResult;

    const skipped = {
      ...result,
      skippedReason: 'localRagDisabled',
    } satisfies RagIndexMaintenanceResult;

    expect(Object.keys(result)).toEqual([
      'clearedEmbeddingCount',
      'rebuiltEmbeddingCount',
      'clearedHistoryDocumentCount',
      'rebuiltHistoryDocumentCount',
      'clearedHistoryChunkCount',
      'skippedReason',
    ]);
    expect(skipped.skippedReason).toBe('localRagDisabled');
  });

  it('keeps history RAG save contracts aligned with backend commands', () => {
    const request = {
      recognizedTitle: 'Two Sum',
      recognizedText: 'Given nums and target.',
      language: 'cpp20',
      platform: 'leetcode',
      model: 'gpt-4o-mini',
      result: '解题策略:\nUse a hash map.',
      userNote: 'Review duplicate values.',
      tags: ['array'],
      algorithmTags: ['hash-table'],
    } satisfies SaveHistoryEntryRequest;
    const entry = {
      id: 'hist_123',
      timestamp: '2026-01-01T00:00:00Z',
      recognizedTitle: 'Two Sum',
      recognizedText: 'Given nums and target.',
      language: 'cpp20',
      platform: 'leetcode',
      model: 'gpt-4o-mini',
      result: '解题策略:\nUse a hash map.',
      userNote: 'Review duplicate values.',
      tags: ['array'],
      algorithmTags: ['hash-table'],
    } satisfies HistoryEntry;

    expect(Object.keys(request)).toEqual([
      'recognizedTitle',
      'recognizedText',
      'language',
      'platform',
      'model',
      'result',
      'userNote',
      'tags',
      'algorithmTags',
    ]);
    expect(Object.keys(entry)).toEqual([
      'id',
      'timestamp',
      'recognizedTitle',
      'recognizedText',
      'language',
      'platform',
      'model',
      'result',
      'userNote',
      'tags',
      'algorithmTags',
    ]);
  });

  it('keeps local RAG search contracts aligned with backend retrieval results', () => {
    const request = {
      recognizedTitle: 'Two Sum',
      recognizedText: 'Given nums and target, return two indices.',
      examples: ['nums = [2,7,11,15], target = 9'],
      constraints: ['2 <= nums.length <= 10^4'],
      tags: ['array'],
      algorithmTags: ['hash-table'],
      targetLanguage: 'cpp20',
      platform: 'leetcode',
      topK: 5,
      minScore: 0.12,
    } satisfies RagSearchRequest;
    const response = {
      items: [
        {
          chunkId: 'ragimpchunk_123_0',
          documentId: 'ragimp_123',
          sourceType: 'userImport',
          sourceId: 'ragimp_123',
          title: 'Two Sum',
          snippet: 'Use a hash table to find complements.',
          score: 0.87,
          kind: 'solution',
          problemType: null,
          language: 'cpp20',
          platform: 'leetcode',
          tags: ['array'],
          algorithmTags: ['hash-table'],
          reason: 'vector=0.730;boost=0.140;title;algorithmTags',
        },
      ],
      skippedReason: null,
    } satisfies RagSearchResponse;

    expect(RAG_SEARCH_SOURCE_TYPES).toEqual([
      'leetcodeIndex',
      'userImport',
      'history',
    ]);
    expect(RAG_SEARCH_CHUNK_KINDS).toEqual([
      'problemStatement',
      'solution',
      'note',
      'template',
      'historySummary',
      'metadata',
    ]);
    expect(Object.keys(request)).toEqual([
      'recognizedTitle',
      'recognizedText',
      'examples',
      'constraints',
      'tags',
      'algorithmTags',
      'targetLanguage',
      'platform',
      'topK',
      'minScore',
    ]);
    expect(Object.keys(response.items[0])).toEqual([
      'chunkId',
      'documentId',
      'sourceType',
      'sourceId',
      'title',
      'snippet',
      'score',
      'kind',
      'problemType',
      'language',
      'platform',
      'tags',
      'algorithmTags',
      'reason',
    ]);
    expect(response.skippedReason).toBeNull();
  });

  it('keeps RAG template selection contracts aligned with backend recommendations', () => {
    const similarItem = {
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
    } satisfies RagSearchResponse['items'][number];
    const request = {
      targetLanguage: null,
      platform: 'leetcode',
      tags: [],
      algorithmTags: ['dfs', 'graph'],
      similarItems: [similarItem],
      maxSolutionModes: 3,
      maxTemplates: 2,
    } satisfies RagTemplateSelectionRequest;
    const response = {
      targetLanguage: 'cpp20',
      platform: 'leetcode',
      solutionModes: [
        {
          id: 'graph-search',
          title: 'Graph Search',
          description: 'Use BFS, DFS, or union-find.',
          problemType: 'graph',
          algorithmTags: ['graph', 'dfs', 'bfs', 'union-find'],
          score: 0.91,
          reason: 'problemType:graph;algorithmTags',
        },
      ],
      templates: [
        {
          id: 'codetpl_ragimp_cpp_graph',
          sourceDocumentId: 'ragimp_cpp_graph',
          title: 'C++ DFS Template',
          language: 'cpp20',
          platform: 'leetcode',
          algorithmTags: ['graph', 'dfs'],
          matchedModeIds: ['graph-search'],
          snippet: 'class Solution { void dfs(int r, int c) {} };',
          templateText: 'class Solution { void dfs(int r, int c) {} };',
          score: 0.92,
          reason: 'language;platform;algorithmTags;solutionMode',
        },
      ],
      skippedReason: null,
    } satisfies RagTemplateSelectionResponse;

    expect(Object.keys(request)).toEqual([
      'targetLanguage',
      'platform',
      'tags',
      'algorithmTags',
      'similarItems',
      'maxSolutionModes',
      'maxTemplates',
    ]);
    expect(Object.keys(response.solutionModes[0])).toEqual([
      'id',
      'title',
      'description',
      'problemType',
      'algorithmTags',
      'score',
      'reason',
    ]);
    expect(Object.keys(response.templates[0])).toEqual([
      'id',
      'sourceDocumentId',
      'title',
      'language',
      'platform',
      'algorithmTags',
      'matchedModeIds',
      'snippet',
      'templateText',
      'score',
      'reason',
    ]);
    expect(response.solutionModes[0].id).toBe('graph-search');
    expect(response.templates[0].matchedModeIds).toEqual(['graph-search']);
  });

  it('keeps RAG prompt context contracts aligned with backend injection output', () => {
    const similarItem = {
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
    } satisfies RagSearchResponse['items'][number];
    const request = {
      recognizedTitle: 'Two Sum',
      recognizedText: 'Given nums and target.',
      targetLanguage: 'cpp20',
      platform: 'leetcode',
      searchResults: [similarItem],
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
    } satisfies RagPromptContextRequest;
    const response = {
      solutionPrompt:
        'You are an algorithm problem solver.\nLocal recalled context',
      promptSection:
        'Current screenshot recognition\nLocal recalled context\nRules for using local recalled context',
      items: [
        {
          id: 'ragimpchunk_123_0',
          sourceType: 'userImport',
          sourceId: 'ragimp_123',
          title: 'Two Sum note',
          snippet: 'Use a hash table to find complements.',
          score: 0.87,
          kind: 'solution',
          language: 'cpp20',
          platform: 'leetcode',
          tags: ['array'],
          algorithmTags: ['hash-table'],
          usedInPrompt: true,
          skippedReason: null,
          tokenEstimate: 18,
          reason: 'vector=0.730;algorithmTags',
        },
        {
          id: 'problemidxchunk_leetcode_1',
          sourceType: 'leetcodeIndex',
          sourceId: 'two-sum',
          title: 'Two Sum',
          snippet: 'LeetCode lightweight metadata only.',
          score: 0.1,
          kind: 'metadata',
          language: null,
          platform: 'leetcode',
          tags: ['Array'],
          algorithmTags: ['hash-table'],
          usedInPrompt: false,
          skippedReason: 'lowConfidence',
          tokenEstimate: 20,
          reason: 'vector=0.100',
        },
      ],
      usedItemCount: 1,
      tokenEstimate: 18,
      skippedReason: null,
    } satisfies RagPromptContextResponse;

    expect(RAG_PROMPT_CONTEXT_SOURCE_TYPES).toEqual([
      'leetcodeIndex',
      'userImport',
      'history',
      'codeTemplate',
      'solutionMode',
    ]);
    expect(RAG_PROMPT_CONTEXT_KINDS).toEqual([
      'problemStatement',
      'solution',
      'note',
      'template',
      'historySummary',
      'metadata',
      'solutionMode',
    ]);
    expect(Object.keys(request)).toEqual([
      'recognizedTitle',
      'recognizedText',
      'targetLanguage',
      'platform',
      'searchResults',
      'templateContext',
      'maxItems',
      'maxContextTokens',
      'minScore',
    ]);
    expect(Object.keys(response)).toEqual([
      'solutionPrompt',
      'promptSection',
      'items',
      'usedItemCount',
      'tokenEstimate',
      'skippedReason',
    ]);
    expect(Object.keys(response.items[0])).toEqual([
      'id',
      'sourceType',
      'sourceId',
      'title',
      'snippet',
      'score',
      'kind',
      'language',
      'platform',
      'tags',
      'algorithmTags',
      'usedInPrompt',
      'skippedReason',
      'tokenEstimate',
      'reason',
    ]);
    expect(response.items[0].usedInPrompt).toBe(true);
    expect(response.items[1].skippedReason).toBe('lowConfidence');
  });
});
