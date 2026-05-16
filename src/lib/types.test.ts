import { describe, expect, it } from 'vitest';
import type {
  AiStreamEventPayload,
  QuestionRecognitionResult,
  RecognitionConfidenceRoute,
} from './types';
import {
  AI_RESULT_STATES,
  APP_STATUSES,
  DEFAULT_APP_STATE,
  DEFAULT_RECOGNITION_CONFIDENCE_THRESHOLDS,
  DEFAULT_SETTINGS,
  LANGUAGE_IDS,
  LANGUAGE_OPTIONS,
  OUTPUT_SPEED_OPTIONS,
  OUTPUT_SPEEDS,
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
    const done = { type: 'done' as const };
    const error = {
      type: 'error' as const,
      code: 'invalidApiKey',
      message: 'The API key was rejected.',
    };

    const _chunkPayload: AiStreamEventPayload = chunk;
    const _donePayload: AiStreamEventPayload = done;
    const _errorPayload: AiStreamEventPayload = error;

    expect(chunk).toEqual({ type: 'chunk', content: 'hello' });
    expect(done).toEqual({ type: 'done' });
    expect(error).toMatchObject({
      type: 'error',
      code: 'invalidApiKey',
    });
  });
});
