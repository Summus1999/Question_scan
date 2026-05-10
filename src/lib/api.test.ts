import { beforeEach, describe, expect, it, vi } from 'vitest';
import {
  BACKEND_COMMANDS,
  hideMainWindow,
  loadAppState,
  resetSettings,
  saveSettings,
  showMainWindow,
  toggleMainWindow,
} from './api';
import { DEFAULT_SETTINGS } from './types';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
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

    expect(invokeMock).toHaveBeenNthCalledWith(1, 'save_settings', {
      settings: DEFAULT_SETTINGS,
    });
    expect(invokeMock).toHaveBeenNthCalledWith(2, 'reset_settings');
    expect(invokeMock).toHaveBeenNthCalledWith(3, 'show_main_window');
    expect(invokeMock).toHaveBeenNthCalledWith(4, 'hide_main_window');
    expect(invokeMock).toHaveBeenNthCalledWith(5, 'toggle_main_window');
  });
});
