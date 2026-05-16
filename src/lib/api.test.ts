import { beforeEach, describe, expect, it, vi } from 'vitest';
import {
  BACKEND_COMMANDS,
  FRONTEND_EVENTS,
  hideMainWindow,
  listenAiStreamEvent,
  listenGlobalShortcutTriggered,
  listenOpenSettings,
  listenRuntimeStateChanged,
  loadAppState,
  resetSettings,
  saveSettings,
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
