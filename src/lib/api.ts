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
  ProblemIndexEntry,
  RagImportedDocument,
  RagImportRequest,
  RagIndexMaintenanceResult,
  RagPromptContextRequest,
  RagPromptContextResponse,
  RagSearchRequest,
  RagSearchResponse,
  RagTemplateSelectionRequest,
  RagTemplateSelectionResponse,
  SaveHistoryEntryRequest,
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
  saveHistoryEntry: 'save_history_entry',
  deleteHistoryEntry: 'delete_history_entry',
  clearHistory: 'clear_history',
  listLeetcodeProblemIndex: 'list_leetcode_problem_index',
  importRagDocuments: 'import_rag_documents',
  listRagImports: 'list_rag_imports',
  deleteRagImport: 'delete_rag_import',
  clearRagIndex: 'clear_rag_index',
  rebuildRagIndex: 'rebuild_rag_index',
  searchRagContext: 'search_rag_context',
  selectRagTemplateContext: 'select_rag_template_context',
  buildRagPromptContext: 'build_rag_prompt_context',
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
  [BACKEND_COMMANDS.saveHistoryEntry]: {
    payload: { request: SaveHistoryEntryRequest };
    response: HistoryEntry | null;
  };
  [BACKEND_COMMANDS.deleteHistoryEntry]: {
    payload: { id: string };
    response: boolean;
  };
  [BACKEND_COMMANDS.clearHistory]: {
    payload: undefined;
    response: undefined;
  };
  [BACKEND_COMMANDS.listLeetcodeProblemIndex]: {
    payload: undefined;
    response: ProblemIndexEntry[];
  };
  [BACKEND_COMMANDS.importRagDocuments]: {
    payload: { request: RagImportRequest };
    response: RagImportedDocument[];
  };
  [BACKEND_COMMANDS.listRagImports]: {
    payload: undefined;
    response: RagImportedDocument[];
  };
  [BACKEND_COMMANDS.deleteRagImport]: {
    payload: { id: string };
    response: boolean;
  };
  [BACKEND_COMMANDS.clearRagIndex]: {
    payload: undefined;
    response: RagIndexMaintenanceResult;
  };
  [BACKEND_COMMANDS.rebuildRagIndex]: {
    payload: undefined;
    response: RagIndexMaintenanceResult;
  };
  [BACKEND_COMMANDS.searchRagContext]: {
    payload: { request: RagSearchRequest };
    response: RagSearchResponse;
  };
  [BACKEND_COMMANDS.selectRagTemplateContext]: {
    payload: { request: RagTemplateSelectionRequest };
    response: RagTemplateSelectionResponse;
  };
  [BACKEND_COMMANDS.buildRagPromptContext]: {
    payload: { request: RagPromptContextRequest };
    response: RagPromptContextResponse;
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

/** 保存一条历史记录，并在后端同步建立历史 RAG 可检索记录。 */
export function saveHistoryEntry(
  request: SaveHistoryEntryRequest,
): Promise<HistoryEntry | null> {
  return invokeBackend(BACKEND_COMMANDS.saveHistoryEntry, { request });
}

/** 根据 ID 删除单条历史记录。 */
export function deleteHistoryEntry(id: string): Promise<boolean> {
  return invokeBackend(BACKEND_COMMANDS.deleteHistoryEntry, { id });
}

/** 清空全部历史记录。 */
export function clearHistory(): Promise<void> {
  return invokeBackend(BACKEND_COMMANDS.clearHistory);
}

/** 获取内置 LeetCode 轻量题目索引。 */
export function listLeetcodeProblemIndex(): Promise<ProblemIndexEntry[]> {
  return invokeBackend(BACKEND_COMMANDS.listLeetcodeProblemIndex);
}

/** 导入用户自己的 Markdown、JSON 或 CSV RAG 资料。 */
export function importRagDocuments(
  request: RagImportRequest,
): Promise<RagImportedDocument[]> {
  return invokeBackend(BACKEND_COMMANDS.importRagDocuments, { request });
}

/** 获取未删除的用户导入资料。 */
export function listRagImports(): Promise<RagImportedDocument[]> {
  return invokeBackend(BACKEND_COMMANDS.listRagImports);
}

/** 删除单条用户导入资料。 */
export function deleteRagImport(id: string): Promise<boolean> {
  return invokeBackend(BACKEND_COMMANDS.deleteRagImport, { id });
}

/** 清除本地 RAG 索引，不删除导入原文、普通历史或内置轻量元数据。 */
export function clearRagIndex(): Promise<RagIndexMaintenanceResult> {
  return invokeBackend(BACKEND_COMMANDS.clearRagIndex);
}

/** 基于当前允许的 RAG 来源重建本地索引。 */
export function rebuildRagIndex(): Promise<RagIndexMaintenanceResult> {
  return invokeBackend(BACKEND_COMMANDS.rebuildRagIndex);
}

/** 执行本地 RAG 检索，返回 top K 相似题、笔记、模板或历史上下文。 */
export function searchRagContext(
  request: RagSearchRequest,
): Promise<RagSearchResponse> {
  return invokeBackend(BACKEND_COMMANDS.searchRagContext, { request });
}

/** 根据相似题、算法标签和默认语言选择可用解题模式与代码模板。 */
export function selectRagTemplateContext(
  request: RagTemplateSelectionRequest,
): Promise<RagTemplateSelectionResponse> {
  return invokeBackend(BACKEND_COMMANDS.selectRagTemplateContext, { request });
}

/** 构建压缩后的 RAG prompt 上下文，并返回已注入上下文的解题 prompt。 */
export function buildRagPromptContext(
  request: RagPromptContextRequest,
): Promise<RagPromptContextResponse> {
  return invokeBackend(BACKEND_COMMANDS.buildRagPromptContext, { request });
}
