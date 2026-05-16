/**
 * Question Scan 前端共享类型定义模块
 *
 * 职责：定义前后端共享的类型、枚举、常量和默认状态。
 * 这是整个项目的类型契约中心，修改这里会影响前后两端。
 */

/** 支持的编程语言列表。顺序影响 UI 下拉框的展示顺序。 */
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

/** 结果面板展示速度选项。 */
export const OUTPUT_SPEEDS = ['fast', 'normal', 'slow', 'custom'] as const;
/** 主题偏好选项。 */
export const THEME_PREFERENCES = ['system', 'light', 'dark'] as const;
/** 界面语言选项：中文 / 英文。 */
export const UI_LOCALES = ['zhCn', 'enUs'] as const;
/** 题目平台格式：ACM 标准输入输出 / LeetCode 函数签名 / 通用。 */
export const PLATFORM_FORMATS = ['acm', 'leetcode', 'generic'] as const;
/** 应用启动状态。 */
export const APP_STATUSES = ['loading', 'ready', 'warning', 'error'] as const;
/** 托盘状态机：空闲 → 截图中 → 识别中 → 生成中 → 完成/失败。 */
export const TRAY_STATUSES = [
  'idle',
  'screenshot',
  'recognizing',
  'generating',
  'complete',
  'failed',
] as const;
/** 截图流程状态：空闲 → 捕获中 → 裁剪中 → 手动框选中 → 就绪 → 失败。 */
export const SCREENSHOT_STATES = [
  'idle',
  'capturing',
  'cropping',
  'selecting',
  'ready',
  'failed',
] as const;
/** AI 结果状态：空闲 → 加载中 → 流式输出中 → 完成 → 失败。 */
export const AI_RESULT_STATES = [
  'idle',
  'loading',
  'streaming',
  'complete',
  'failed',
] as const;
/** 识别置信度路由：自动接受 / 需要确认 / 手动兜底。 */
export const RECOGNITION_CONFIDENCE_ROUTES = [
  'autoAccept',
  'needsConfirmation',
  'manualFallback',
] as const;

export type LanguageId = (typeof LANGUAGE_IDS)[number];
export type OutputSpeed = (typeof OUTPUT_SPEEDS)[number];
export type ThemePreference = (typeof THEME_PREFERENCES)[number];
export type UiLocale = (typeof UI_LOCALES)[number];
export type PlatformFormat = (typeof PLATFORM_FORMATS)[number];
export type AppStatus = (typeof APP_STATUSES)[number];
export type TrayStatus = (typeof TRAY_STATUSES)[number];
export type ScreenshotState = (typeof SCREENSHOT_STATES)[number];
export type AiResultState = (typeof AI_RESULT_STATES)[number];
export type RecognitionConfidenceRoute =
  (typeof RECOGNITION_CONFIDENCE_ROUTES)[number];

/** 题目区域边界框（像素坐标）。 */
export interface QuestionBoundingBox {
  x: number;
  y: number;
  width: number;
  height: number;
}

/**
 * 视觉模型返回的题目识别结果。
 * confidence: 0~1，越高表示模型对识别结果越有信心。
 * title / questionText 可能为 null，表示模型未提取到对应内容。
 */
export interface QuestionRecognitionResult {
  boundingBox: QuestionBoundingBox;
  confidence: number;
  title: string | null;
  questionText: string | null;
  reason: string;
}

/** 用户手动框选的裁剪区域（视口坐标，归一化后使用）。 */
export interface CropSelectionRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

/**
 * 识别置信度阈值配置。
 * 默认：autoAcceptMinConfidence = 0.85（≥此值自动接受）
 *       confirmationMinConfidence = 0.55（≥此值需要用户确认，<此值进入手动兜底）
 */
export interface RecognitionConfidenceThresholds {
  autoAcceptMinConfidence: number;
  confirmationMinConfidence: number;
}

/** 下拉框选项的通用类型。 */
export type SelectOption<TValue extends string> = Readonly<{
  value: TValue;
  label: string;
}>;

/** 运行时状态快照（不含设置本身）。 */
export interface RuntimeState {
  status: AppStatus;
  trayStatus: TrayStatus;
  screenshotState: ScreenshotState;
  aiResultState: AiResultState;
}

/** 单条历史记录。 */
export interface HistoryEntry {
  id: string;
  timestamp: string;
  recognizedTitle: string | null;
  recognizedText: string | null;
  language: LanguageId;
  platform: PlatformFormat;
  model: string;
  result: string;
  userNote: string | null;
}

/**
 * 用户设置完整结构。
 * 注意：providerApiKey 从前端保存时会传入后端，后端存入 Windows Credential Locker。
 * 前端重新加载时，该字段会被脱敏为空字符串。
 */
export interface AppSettings {
  providerName: string;
  providerBaseUrl: string;
  providerApiKey: string;
  providerModel: string;
  requestTimeoutSeconds: number;
  streamingEnabled: boolean;
  defaultLanguage: LanguageId;
  platformFormat: PlatformFormat;
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

/**
 * 应用完整状态快照，由后端 `load_app_state` 命令返回。
 * 这是前端所有 UI 状态的单一数据源。
 */
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

// ------------------------------------------------------------------------------
// AI 流式事件载荷定义（由 Rust 后端通过 Tauri 事件通道发射）。
// 前端通过 listenAiStreamEvent 监听这些事件。
// ------------------------------------------------------------------------------

/** AI 流式输出：收到一个文本片段。 */
export type AiStreamChunkPayload = {
  type: 'chunk';
  content: string;
};

/** AI 流式输出：模型响应已完整结束。 */
export type AiStreamDonePayload = {
  type: 'done';
};

/** AI 流式输出：请求过程中发生错误。 */
export type AiStreamErrorPayload = {
  type: 'error';
  code: string;
  message: string;
};

/** AI 流式事件联合类型。前端监听此类型的事件流。 */
export type AiStreamEventPayload =
  | AiStreamChunkPayload
  | AiStreamDonePayload
  | AiStreamErrorPayload;

/** 语言下拉框选项列表。C++20 排在首位作为默认推荐。 */
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

/** 输出速度下拉框选项。标签会通过 i18n 在运行时替换为对应语言。 */
export const OUTPUT_SPEED_OPTIONS = [
  { value: 'fast', label: 'Fast' },
  { value: 'normal', label: 'Normal' },
  { value: 'slow', label: 'Slow' },
  { value: 'custom', label: 'Custom' },
] as const satisfies ReadonlyArray<SelectOption<OutputSpeed>>;

/**
 * 默认设置值。
 * 注意：修改默认值后，需要同时更新 settings.rs 中的 Rust 端默认值，保持前后端一致。
 */
export const DEFAULT_SETTINGS: AppSettings = {
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
};

/**
 * 默认识别置信度阈值。
 * 自动接受 ≥ 0.85，确认区间 [0.55, 0.85)，低于 0.55 进入手动框选兜底。
 */
export const DEFAULT_RECOGNITION_CONFIDENCE_THRESHOLDS: RecognitionConfidenceThresholds =
  {
    autoAcceptMinConfidence: 0.85,
    confirmationMinConfidence: 0.55,
  };

/** 应用启动时的默认状态。hydrate 完成后会被后端快照覆盖。 */
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
