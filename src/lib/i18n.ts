import type {
  AiResultState,
  AppStatus,
  OutputSpeed,
  ScreenshotState,
  ThemePreference,
  TrayStatus,
  UiLocale,
} from './types';

type AppMessages = {
  actions: {
    hideWindow: string;
    reload: string;
    reset: string;
    resetDefaults: string;
    save: string;
    saveSettings: string;
    saving: string;
    savingEllipsis: string;
    showWindow: string;
  };
  appStatus: Record<AppStatus, string>;
  aiResultState: Record<AiResultState, string>;
  fields: {
    charactersPerSecond: string;
    defaultLanguage: string;
    globalShortcut: string;
    interfaceLanguage: string;
    model: string;
    outputSpeed: string;
    presentation: string;
    providerBaseUrl: string;
    summary: string;
    theme: string;
  };
  footer: string;
  header: {
    eyebrow: string;
    subtitle: string;
    title: string;
  };
  info: {
    hidden: string;
    notResolved: string;
    tray: string;
    version: string;
    visible: string;
    window: string;
  };
  localeOptions: Record<UiLocale, string>;
  notices: {
    backendNotResponding: string;
    resetFailed: string;
    resetSucceeded: string;
    saveFailed: string;
    saveSucceeded: string;
    windowActionFailed: string;
  };
  outputSpeed: Record<OutputSpeed, string>;
  result: {
    body: string;
    title: string;
  };
  runtime: {
    aiResultState: string;
    settingsPath: string;
    screenshotState: string;
    startupStatus: string;
    subtitle: string;
    title: string;
    trayStatus: string;
    windowVisible: string;
  };
  screenshotState: Record<ScreenshotState, string>;
  settings: {
    launchToTrayDescription: string;
    launchToTrayLabel: string;
    saveHistoryDescription: string;
    saveHistoryLabel: string;
    subtitle: string;
    title: string;
  };
  theme: Record<ThemePreference, string>;
  trayStatus: Record<TrayStatus, string>;
};

const zhCn: AppMessages = {
  actions: {
    hideWindow: '隐藏窗口',
    reload: '重新加载',
    reset: '重置',
    resetDefaults: '恢复默认',
    save: '保存',
    saveSettings: '保存设置',
    saving: '保存中',
    savingEllipsis: '保存中...',
    showWindow: '显示窗口',
  },
  appStatus: {
    loading: '加载中',
    ready: '就绪',
    warning: '警告',
    error: '错误',
  },
  aiResultState: {
    idle: '空闲',
    loading: '请求中',
    streaming: '输出中',
    complete: '完成',
    failed: '失败',
  },
  fields: {
    charactersPerSecond: '每秒字符数',
    defaultLanguage: '默认编程语言',
    globalShortcut: '全局快捷键',
    interfaceLanguage: '界面语言',
    model: '模型',
    outputSpeed: '输出速度',
    presentation: '展示方式',
    providerBaseUrl: '服务 Base URL',
    summary: '摘要',
    theme: '主题',
  },
  footer:
    '阶段 1：应用外壳、托盘、设置持久化和错误提示。截图和 AI 请求尚未接入。',
  header: {
    eyebrow: 'Question Scan',
    subtitle:
      '托盘驱动的工作区、本地设置存储，以及后续 MVP 功能会扩展的答案面板骨架。',
    title: '桌面应用底座',
  },
  info: {
    hidden: '已隐藏',
    notResolved: '尚未解析',
    tray: '托盘',
    version: '版本',
    visible: '可见',
    window: '窗口',
  },
  localeOptions: {
    zhCn: '中文',
    enUs: 'English',
  },
  notices: {
    backendNotResponding: '桌面后端没有响应。',
    resetFailed: '重置设置失败。',
    resetSucceeded: '设置已恢复为默认本地配置。',
    saveFailed: '保存设置失败。',
    saveSucceeded: '设置已保存到本地配置。',
    windowActionFailed: '窗口操作失败。',
  },
  outputSpeed: {
    fast: '快速',
    normal: '正常',
    slow: '慢速',
    custom: '自定义',
  },
  result: {
    body: '此区域在阶段 1 保持占位。后续会在这里接入截图、识别和答案输出。',
    title: '结果面板占位',
  },
  runtime: {
    aiResultState: 'AI 结果状态',
    settingsPath: '设置路径',
    screenshotState: '截图状态',
    startupStatus: '启动状态',
    subtitle: '当前本地状态的实时摘要，后续截图和 AI 流程会继续扩展这里。',
    title: '运行状态快照',
    trayStatus: '托盘状态',
    windowVisible: '窗口可见',
  },
  screenshotState: {
    idle: '空闲',
    capturing: '截图中',
    cropping: '裁剪中',
    selecting: '等待框选',
    ready: '就绪',
    failed: '失败',
  },
  settings: {
    launchToTrayDescription: '启动后隐藏窗口，并让应用保留在托盘。',
    launchToTrayLabel: '启动到托盘',
    saveHistoryDescription: '保存本地答案快照，便于稍后回看。',
    saveHistoryLabel: '保存历史',
    subtitle: '本地默认值、服务占位配置，以及后续阶段会扩展的外壳级选项。',
    title: '设置',
  },
  theme: {
    system: '跟随系统',
    light: '浅色',
    dark: '深色',
  },
  trayStatus: {
    idle: '空闲',
    screenshot: '截图中',
    recognizing: '识别中',
    generating: '生成中',
    complete: '完成',
    failed: '失败',
  },
};

const enUs: AppMessages = {
  actions: {
    hideWindow: 'Hide window',
    reload: 'Reload',
    reset: 'Reset',
    resetDefaults: 'Reset defaults',
    save: 'Save',
    saveSettings: 'Save settings',
    saving: 'Saving',
    savingEllipsis: 'Saving...',
    showWindow: 'Show window',
  },
  appStatus: {
    loading: 'Loading',
    ready: 'Ready',
    warning: 'Warning',
    error: 'Error',
  },
  aiResultState: {
    idle: 'Idle',
    loading: 'Loading',
    streaming: 'Streaming',
    complete: 'Complete',
    failed: 'Failed',
  },
  fields: {
    charactersPerSecond: 'Characters per second',
    defaultLanguage: 'Default language',
    globalShortcut: 'Global shortcut',
    interfaceLanguage: 'Interface language',
    model: 'Model',
    outputSpeed: 'Output speed',
    presentation: 'Presentation',
    providerBaseUrl: 'Provider base URL',
    summary: 'Summary',
    theme: 'Theme',
  },
  footer:
    'Stage 1 only: app shell, tray, settings persistence, and error reporting. No capture or AI integration is wired yet.',
  header: {
    eyebrow: 'Question Scan',
    subtitle:
      'Tray-backed workspace, local settings storage, and an answer panel scaffold for the rest of the MVP.',
    title: 'Desktop shell',
  },
  info: {
    hidden: 'Hidden',
    notResolved: 'Not resolved yet',
    tray: 'Tray',
    version: 'Version',
    visible: 'Visible',
    window: 'Window',
  },
  localeOptions: {
    zhCn: '中文',
    enUs: 'English',
  },
  notices: {
    backendNotResponding: 'The desktop backend did not respond.',
    resetFailed: 'Failed to reset settings.',
    resetSucceeded: 'Settings were reset to the default local profile.',
    saveFailed: 'Failed to save settings.',
    saveSucceeded: 'Settings saved to the local config store.',
    windowActionFailed: 'Window action failed.',
  },
  outputSpeed: {
    fast: 'Fast',
    normal: 'Normal',
    slow: 'Slow',
    custom: 'Custom',
  },
  result: {
    body: 'This area is intentionally empty for stage 1. Later stages will mount capture, recognition, and answer output here.',
    title: 'Placeholder result surface',
  },
  runtime: {
    aiResultState: 'AI result state',
    settingsPath: 'Settings path',
    screenshotState: 'Screenshot state',
    startupStatus: 'Startup status',
    subtitle:
      'A live summary of the current local state that the later capture and AI flows will extend.',
    title: 'Runtime snapshot',
    trayStatus: 'Tray status',
    windowVisible: 'Window visible',
  },
  screenshotState: {
    idle: 'Idle',
    capturing: 'Capturing',
    cropping: 'Cropping',
    selecting: 'Selecting',
    ready: 'Ready',
    failed: 'Failed',
  },
  settings: {
    launchToTrayDescription:
      'Hide the window on launch and keep the app in the tray.',
    launchToTrayLabel: 'Launch to tray',
    saveHistoryDescription: 'Store the local answer snapshot for later review.',
    saveHistoryLabel: 'Save history',
    subtitle:
      'Local defaults, provider placeholders, and the shell-level options that later stages will expand.',
    title: 'Settings',
  },
  theme: {
    system: 'System',
    light: 'Light',
    dark: 'Dark',
  },
  trayStatus: {
    idle: 'Idle',
    screenshot: 'Screenshot',
    recognizing: 'Recognizing',
    generating: 'Generating',
    complete: 'Complete',
    failed: 'Failed',
  },
};

export const APP_MESSAGES: Record<UiLocale, AppMessages> = {
  zhCn,
  enUs,
};

// Returns a complete message set and falls back to Chinese for unknown persisted values.
export function getMessages(locale: UiLocale): AppMessages {
  return APP_MESSAGES[locale] ?? APP_MESSAGES.zhCn;
}
