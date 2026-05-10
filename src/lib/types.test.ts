import { describe, expect, it } from 'vitest';
import {
  AI_RESULT_STATES,
  APP_STATUSES,
  DEFAULT_APP_STATE,
  DEFAULT_SETTINGS,
  LANGUAGE_IDS,
  LANGUAGE_OPTIONS,
  OUTPUT_SPEED_OPTIONS,
  OUTPUT_SPEEDS,
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
      providerBaseUrl: 'https://api.openai.com/v1',
      providerModel: 'gpt-4o-mini',
      defaultLanguage: 'cpp20',
      outputSpeed: 'normal',
      customCharactersPerSecond: 24,
      globalShortcut: 'Ctrl+Shift+Q',
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
    });
  });
});
