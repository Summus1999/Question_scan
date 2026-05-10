export const LANGUAGE_IDS = [
  'cpp17',
  'cpp20',
  'python',
  'java',
  'javascript',
  'typescript',
  'go',
  'rust',
] as const;

export const OUTPUT_SPEEDS = ['fast', 'normal', 'slow', 'custom'] as const;
export const THEME_PREFERENCES = ['system', 'light', 'dark'] as const;
export const UI_LOCALES = ['zhCn', 'enUs'] as const;
export const APP_STATUSES = ['loading', 'ready', 'warning', 'error'] as const;
export const TRAY_STATUSES = [
  'idle',
  'screenshot',
  'recognizing',
  'generating',
  'complete',
  'failed',
] as const;
export const SCREENSHOT_STATES = [
  'idle',
  'capturing',
  'cropping',
  'selecting',
  'ready',
  'failed',
] as const;
export const AI_RESULT_STATES = [
  'idle',
  'loading',
  'streaming',
  'complete',
  'failed',
] as const;

export type LanguageId = (typeof LANGUAGE_IDS)[number];
export type OutputSpeed = (typeof OUTPUT_SPEEDS)[number];
export type ThemePreference = (typeof THEME_PREFERENCES)[number];
export type UiLocale = (typeof UI_LOCALES)[number];
export type AppStatus = (typeof APP_STATUSES)[number];
export type TrayStatus = (typeof TRAY_STATUSES)[number];
export type ScreenshotState = (typeof SCREENSHOT_STATES)[number];
export type AiResultState = (typeof AI_RESULT_STATES)[number];

export type SelectOption<TValue extends string> = Readonly<{
  value: TValue;
  label: string;
}>;

export interface RuntimeState {
  status: AppStatus;
  trayStatus: TrayStatus;
  screenshotState: ScreenshotState;
  aiResultState: AiResultState;
}

export interface AppSettings {
  providerBaseUrl: string;
  providerModel: string;
  defaultLanguage: LanguageId;
  outputSpeed: OutputSpeed;
  customCharactersPerSecond: number;
  globalShortcut: string;
  globalShortcutEnabled: boolean;
  screenshotMaxLongEdge: number;
  screenshotJpegQuality: number;
  saveHistory: boolean;
  launchToTray: boolean;
  theme: ThemePreference;
  uiLocale: UiLocale;
}

export interface AppState extends RuntimeState {
  settings: AppSettings;
  settingsPath: string;
  startupWarning: string | null;
  windowVisible: boolean;
  version: string;
  globalShortcutRegistered: boolean;
  globalShortcutError: string | null;
  globalShortcutTriggerCount: number;
}

export const LANGUAGE_OPTIONS = [
  { value: 'cpp20', label: 'C++20' },
  { value: 'cpp17', label: 'C++17' },
  { value: 'python', label: 'Python' },
  { value: 'java', label: 'Java' },
  { value: 'javascript', label: 'JavaScript' },
  { value: 'typescript', label: 'TypeScript' },
  { value: 'go', label: 'Go' },
  { value: 'rust', label: 'Rust' },
] as const satisfies ReadonlyArray<SelectOption<LanguageId>>;

export const OUTPUT_SPEED_OPTIONS = [
  { value: 'fast', label: 'Fast' },
  { value: 'normal', label: 'Normal' },
  { value: 'slow', label: 'Slow' },
  { value: 'custom', label: 'Custom' },
] as const satisfies ReadonlyArray<SelectOption<OutputSpeed>>;

export const DEFAULT_SETTINGS: AppSettings = {
  providerBaseUrl: 'https://api.openai.com/v1',
  providerModel: 'gpt-4o-mini',
  defaultLanguage: 'cpp20',
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
};

export const DEFAULT_APP_STATE: AppState = {
  status: 'loading',
  trayStatus: 'idle',
  screenshotState: 'idle',
  aiResultState: 'idle',
  settings: DEFAULT_SETTINGS,
  settingsPath: '',
  startupWarning: null,
  windowVisible: true,
  version: '0.1.0',
  globalShortcutRegistered: false,
  globalShortcutError: null,
  globalShortcutTriggerCount: 0,
};
