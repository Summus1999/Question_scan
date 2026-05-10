import { fireEvent, render, screen, waitFor } from '@testing-library/react';
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

    expect(screen.getByLabelText('模型')).toHaveValue('qwen-max');
    expect(screen.getByLabelText('界面语言')).toHaveValue('zhCn');
    expect(screen.getByText('运行状态快照')).toBeInTheDocument();
    expect(screen.getByText('设置')).toBeInTheDocument();
    expect(screen.getByText('结果面板占位')).toBeInTheDocument();
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
});
