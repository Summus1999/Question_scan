import { render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import App from './App';
import { DEFAULT_APP_STATE, DEFAULT_SETTINGS } from './lib/types';

const {
  loadAppStateMock,
  saveSettingsMock,
  resetSettingsMock,
  showMainWindowMock,
  hideMainWindowMock,
  toggleMainWindowMock,
} = vi.hoisted(() => ({
  loadAppStateMock: vi.fn(),
  saveSettingsMock: vi.fn(),
  resetSettingsMock: vi.fn(),
  showMainWindowMock: vi.fn(),
  hideMainWindowMock: vi.fn(),
  toggleMainWindowMock: vi.fn(),
}));

vi.mock('./lib/api', () => ({
  loadAppState: loadAppStateMock,
  saveSettings: saveSettingsMock,
  resetSettings: resetSettingsMock,
  showMainWindow: showMainWindowMock,
  hideMainWindow: hideMainWindowMock,
  toggleMainWindow: toggleMainWindowMock,
}));

describe('App shell', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders the stage 1 shell after loading the app snapshot', async () => {
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

    expect(screen.getByText('Desktop shell')).toBeInTheDocument();

    await waitFor(() => {
      expect(screen.getByLabelText('Provider base URL')).toHaveValue(
        DEFAULT_SETTINGS.providerBaseUrl,
      );
    });

    expect(screen.getByLabelText('Model')).toHaveValue('qwen-max');
    expect(screen.getByText('Runtime snapshot')).toBeInTheDocument();
    expect(screen.getByText('Settings')).toBeInTheDocument();
    expect(screen.getByText('Placeholder result surface')).toBeInTheDocument();
  });
});
