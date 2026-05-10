import { invoke } from '@tauri-apps/api/core';
import type { AppSettings, AppState } from './types';

export const BACKEND_COMMANDS = {
  loadAppState: 'load_app_state',
  saveSettings: 'save_settings',
  resetSettings: 'reset_settings',
  showMainWindow: 'show_main_window',
  hideMainWindow: 'hide_main_window',
  toggleMainWindow: 'toggle_main_window',
} as const;

type BackendCommandSpec = {
  [BACKEND_COMMANDS.loadAppState]: {
    payload: undefined;
    response: AppState;
  };
  [BACKEND_COMMANDS.saveSettings]: {
    payload: { settings: AppSettings };
    response: AppState;
  };
  [BACKEND_COMMANDS.resetSettings]: {
    payload: undefined;
    response: AppState;
  };
  [BACKEND_COMMANDS.showMainWindow]: {
    payload: undefined;
    response: AppState;
  };
  [BACKEND_COMMANDS.hideMainWindow]: {
    payload: undefined;
    response: AppState;
  };
  [BACKEND_COMMANDS.toggleMainWindow]: {
    payload: undefined;
    response: AppState;
  };
};

export type BackendCommandName = keyof BackendCommandSpec;
export type BackendCommandPayload<TCommand extends BackendCommandName> =
  BackendCommandSpec[TCommand]['payload'];
export type BackendCommandResponse<TCommand extends BackendCommandName> =
  BackendCommandSpec[TCommand]['response'];

// Centralizes command names, payloads, and response types so wrapper drift is easy to catch.
function invokeBackend<TCommand extends BackendCommandName>(
  command: TCommand,
  ...payload: BackendCommandPayload<TCommand> extends undefined
    ? []
    : [BackendCommandPayload<TCommand>]
): Promise<BackendCommandResponse<TCommand>> {
  if (payload.length === 0) {
    return invoke<BackendCommandResponse<TCommand>>(command);
  }

  return invoke<BackendCommandResponse<TCommand>>(command, payload[0]);
}

// Loads the canonical startup snapshot from Rust; use this when UI state looks stale.
export function loadAppState(): Promise<AppState> {
  return invokeBackend(BACKEND_COMMANDS.loadAppState);
}

// Sends the full settings object to Rust so validation and persistence stay backend-owned.
export function saveSettings(settings: AppSettings): Promise<AppState> {
  return invokeBackend(BACKEND_COMMANDS.saveSettings, { settings });
}

// Restores backend defaults and returns the same snapshot shape as a normal save.
export function resetSettings(): Promise<AppState> {
  return invokeBackend(BACKEND_COMMANDS.resetSettings);
}

// Shows the main Tauri window and reports the refreshed backend snapshot.
export function showMainWindow(): Promise<AppState> {
  return invokeBackend(BACKEND_COMMANDS.showMainWindow);
}

// Hides the main Tauri window without exiting the tray process.
export function hideMainWindow(): Promise<AppState> {
  return invokeBackend(BACKEND_COMMANDS.hideMainWindow);
}

// Toggles visibility through Rust so header controls and tray actions share behavior.
export function toggleMainWindow(): Promise<AppState> {
  return invokeBackend(BACKEND_COMMANDS.toggleMainWindow);
}
