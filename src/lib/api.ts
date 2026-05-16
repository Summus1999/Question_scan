/**
 * Question Scan 前后端 API 桥接模块
 *
 * 职责：封装所有 Tauri 命令调用和事件监听，提供类型安全的接口。
 * 前端代码不应直接调用 `invoke` 或 `listen`，而应通过此模块的函数。
 */
import { invoke } from '@tauri-apps/api/core';
import { type Event, listen, type UnlistenFn } from '@tauri-apps/api/event';
import type {
  AiStreamEventPayload,
  AppSettings,
  AppState,
  HistoryEntry,
  TrayStatus,
} from './types';

/** 后端 Tauri 命令名称常量。集中管理避免硬编码字符串散落。 */
export const BACKEND_COMMANDS = {
  loadAppState: 'load_app_state',
  saveSettings: 'save_settings',
  resetSettings: 'reset_settings',
  showMainWindow: 'show_main_window',
  hideMainWindow: 'hide_main_window',
  toggleMainWindow: 'toggle_main_window',
  setTrayStatus: 'set_tray_status',
  sendAiRequest: 'send_ai_request',
  regenerateWithLanguage: 'regenerate_with_language',
  clearCache: 'clear_cache',
  listHistory: 'list_history',
  deleteHistoryEntry: 'delete_history_entry',
  clearHistory: 'clear_history',
} as const;

/** 前端监听的后端事件名称常量。 */
export const FRONTEND_EVENTS = {
  globalShortcutTriggered: 'question-scan:global-shortcut-triggered',
  openSettings: 'question-scan:open-settings',
  runtimeStateChanged: 'question-scan:runtime-state-changed',
  aiStreamEvent: 'question-scan:ai-stream-event',
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
  [BACKEND_COMMANDS.setTrayStatus]: {
    payload: { status: TrayStatus };
    response: AppState;
  };
  [BACKEND_COMMANDS.sendAiRequest]: {
    payload: {
      instruction: string;
      imageBytes: number[];
      imageMimeType: string;
    };
    response: undefined;
  };
  [BACKEND_COMMANDS.regenerateWithLanguage]: {
    payload: {
      language: string;
    };
    response: undefined;
  };
  [BACKEND_COMMANDS.clearCache]: {
    payload: undefined;
    response: undefined;
  };
  [BACKEND_COMMANDS.listHistory]: {
    payload: undefined;
    response: HistoryEntry[];
  };
  [BACKEND_COMMANDS.deleteHistoryEntry]: {
    payload: { id: string };
    response: boolean;
  };
  [BACKEND_COMMANDS.clearHistory]: {
    payload: undefined;
    response: undefined;
  };
};

export type GlobalShortcutTriggeredPayload = {
  shortcut: string;
  triggerCount: number;
};

export type RuntimeStateChangedPayload = {
  reason: string;
};

export type BackendCommandName = keyof BackendCommandSpec;
export type BackendCommandPayload<TCommand extends BackendCommandName> =
  BackendCommandSpec[TCommand]['payload'];
export type BackendCommandResponse<TCommand extends BackendCommandName> =
  BackendCommandSpec[TCommand]['response'];

/**
 * 统一的 Tauri 命令调用包装器。
 * 通过泛型约束保证命令名、请求体、响应体的类型安全。
 * 如果命令参数类型为 undefined，则不需要传 payload。
 */
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

/** 从 Rust 后端加载应用启动状态快照。UI 状态异常时可调用此函数刷新。 */
export function loadAppState(): Promise<AppState> {
  return invokeBackend(BACKEND_COMMANDS.loadAppState);
}

/** 将完整设置对象发送到 Rust 后端。校验和持久化逻辑由后端负责。 */
export function saveSettings(settings: AppSettings): Promise<AppState> {
  return invokeBackend(BACKEND_COMMANDS.saveSettings, { settings });
}

/** 恢复后端默认设置。返回的状态快照结构与正常保存一致。 */
export function resetSettings(): Promise<AppState> {
  return invokeBackend(BACKEND_COMMANDS.resetSettings);
}

/** 显示主窗口，并返回刷新后的后端状态快照。 */
export function showMainWindow(): Promise<AppState> {
  return invokeBackend(BACKEND_COMMANDS.showMainWindow);
}

/** 隐藏主窗口（不退出托盘进程）。 */
export function hideMainWindow(): Promise<AppState> {
  return invokeBackend(BACKEND_COMMANDS.hideMainWindow);
}

/** 切换主窗口显隐状态。标题栏按钮和托盘动作共用此行为。 */
export function toggleMainWindow(): Promise<AppState> {
  return invokeBackend(BACKEND_COMMANDS.toggleMainWindow);
}

/** 更新托盘状态。前端流程（如截图后）可调用此函数同步托盘显示。 */
export function setTrayStatus(status: TrayStatus): Promise<AppState> {
  return invokeBackend(BACKEND_COMMANDS.setTrayStatus, { status });
}

/**
 * 监听全局快捷键触发事件。
 * 当用户在后台按下快捷键时，Rust 会发射此事件，前端应刷新状态并显示窗口。
 */
export function listenGlobalShortcutTriggered(
  handler: (event: Event<GlobalShortcutTriggeredPayload>) => void,
): Promise<UnlistenFn> {
  return listen<GlobalShortcutTriggeredPayload>(
    FRONTEND_EVENTS.globalShortcutTriggered,
    handler,
  );
}

/**
 * 监听"打开设置"事件。
 * 用户从托盘菜单选择"打开设置"时触发，前端应滚动到设置区域。
 */
export function listenOpenSettings(
  handler: (event: Event<RuntimeStateChangedPayload>) => void,
): Promise<UnlistenFn> {
  return listen<RuntimeStateChangedPayload>(
    FRONTEND_EVENTS.openSettings,
    handler,
  );
}

/**
 * 监听运行时状态变更事件。
 * 托盘操作（如启用/禁用快捷键）会触发此事件，前端应同步状态。
 */
export function listenRuntimeStateChanged(
  handler: (event: Event<RuntimeStateChangedPayload>) => void,
): Promise<UnlistenFn> {
  return listen<RuntimeStateChangedPayload>(
    FRONTEND_EVENTS.runtimeStateChanged,
    handler,
  );
}

/**
 * 监听 AI 流式输出事件。
 * 后端通过 SSE 接收模型响应，解析后逐段发射 chunk/done/error 事件给前端。
 */
export function listenAiStreamEvent(
  handler: (event: Event<AiStreamEventPayload>) => void,
): Promise<UnlistenFn> {
  return listen<AiStreamEventPayload>(FRONTEND_EVENTS.aiStreamEvent, handler);
}

/**
 * 发起 AI 多模态请求。
 * 请求启动后，结果通过 listenAiStreamEvent 事件流返回，而非此 Promise。
 */
export function sendAiRequest(
  instruction: string,
  imageBytes: Uint8Array,
  imageMimeType: string,
): Promise<void> {
  return invokeBackend(BACKEND_COMMANDS.sendAiRequest, {
    instruction,
    imageBytes: Array.from(imageBytes),
    imageMimeType,
  });
}

/**
 * 使用相同截图数据，切换语言重新生成答案。
 * 结果通过 listenAiStreamEvent 事件流返回。
 */
export function regenerateWithLanguage(language: string): Promise<void> {
  return invokeBackend(BACKEND_COMMANDS.regenerateWithLanguage, {
    language,
  });
}

/** 清空系统临时目录中的遗留截图文件。 */
export function clearCache(): Promise<void> {
  return invokeBackend(BACKEND_COMMANDS.clearCache);
}

/** 获取已保存的历史记录列表。 */
export function listHistory(): Promise<HistoryEntry[]> {
  return invokeBackend(BACKEND_COMMANDS.listHistory);
}

/** 根据 ID 删除单条历史记录。 */
export function deleteHistoryEntry(id: string): Promise<boolean> {
  return invokeBackend(BACKEND_COMMANDS.deleteHistoryEntry, { id });
}

/** 清空全部历史记录。 */
export function clearHistory(): Promise<void> {
  return invokeBackend(BACKEND_COMMANDS.clearHistory);
}
