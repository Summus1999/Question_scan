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
  listenGlobalShortcutTriggeredMock,
  listenOpenSettingsMock,
  listenRuntimeStateChangedMock,
} = vi.hoisted(() => ({
  loadAppStateMock: vi.fn(),
  saveSettingsMock: vi.fn(),
  resetSettingsMock: vi.fn(),
  showMainWindowMock: vi.fn(),
  hideMainWindowMock: vi.fn(),
  toggleMainWindowMock: vi.fn(),
  listenGlobalShortcutTriggeredMock: vi.fn(),
  listenOpenSettingsMock: vi.fn(),
  listenRuntimeStateChangedMock: vi.fn(),
}));

vi.mock('./lib/api', () => ({
  loadAppState: loadAppStateMock,
  saveSettings: saveSettingsMock,
  resetSettings: resetSettingsMock,
  showMainWindow: showMainWindowMock,
  hideMainWindow: hideMainWindowMock,
  toggleMainWindow: toggleMainWindowMock,
  listenGlobalShortcutTriggered: listenGlobalShortcutTriggeredMock,
  listenOpenSettings: listenOpenSettingsMock,
  listenRuntimeStateChanged: listenRuntimeStateChangedMock,
}));

describe('App shell', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    listenGlobalShortcutTriggeredMock.mockResolvedValue(vi.fn());
    listenOpenSettingsMock.mockResolvedValue(vi.fn());
    listenRuntimeStateChangedMock.mockResolvedValue(vi.fn());
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
    expect(screen.getByLabelText('全局快捷键')).toHaveValue('Ctrl+Shift+Q');
    expect(screen.getByText('启用全局快捷键')).toBeInTheDocument();
    expect(screen.getByText('运行状态快照')).toBeInTheDocument();
    expect(screen.getByText('设置')).toBeInTheDocument();
    expect(screen.getByText('结果面板占位')).toBeInTheDocument();
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
    expect(screen.getByText(/"screenshotState": "idle"/)).toBeInTheDocument();
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
    expect(
      screen.getByText(/"screenshotState": "cropping"/),
    ).toBeInTheDocument();
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
    expect(screen.getByText(/"screenshotState": "ready"/)).toBeInTheDocument();
    expect(screen.getByText(/"manualCropSelection"/)).toBeInTheDocument();
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
});
