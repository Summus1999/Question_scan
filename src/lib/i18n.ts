/**
 * 国际化文案模块
 *
 * 职责：提供中英文两套完整的界面文案，所有 UI 显示文本都从这里获取。
 * 使用 getMessages(locale) 获取当前语言对应的文案对象。
 */
import type {
  AiResultState,
  AppStatus,
  OutputSpeed,
  RagImportKind,
  RagPromptContextKind,
  RagPromptContextSourceType,
  ScreenshotState,
  ThemePreference,
  TrayStatus,
  UiLocale,
} from './types';

type AppMessages = {
  actions: {
    clearCache: string;
    clearCacheNotice: string;
    clearHistory: string;
    clearHistoryNotice: string;
    deleteHistoryEntry: string;
    deleteHistoryEntryNotice: string;
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
  dataManagement: {
    clearRagIndex: string;
    clearRagIndexNotice: string;
    deleteRagImport: string;
    deleteRagImportNotice: string;
    disableHistoryIndexing: string;
    disableHistoryIndexingNotice: string;
    historyIndexingAlreadyDisabled: string;
    importKindLabels: Record<RagImportKind, string>;
    importUpdatedAt: string;
    loading: string;
    noHistoryEntries: string;
    noRagImports: string;
    ragIndexSkippedEmpty: string;
    ragIndexSkippedLocalRagDisabled: string;
    ragIndexSkippedUnknown: string;
    ragIndexStats: string;
    ragPrivacyDescription: string;
    ragPrivacyTitle: string;
    rebuildRagIndex: string;
    rebuildRagIndexNotice: string;
    refreshHistory: string;
    refreshRagImports: string;
    title: string;
    untitledHistoryEntry: string;
  };
  appStatus: Record<AppStatus, string>;
  aiResultState: Record<AiResultState, string>;
  fields: {
    apiKey: string;
    charactersPerSecond: string;
    defaultLanguage: string;
    globalShortcut: string;
    globalShortcutEnabled: string;
    interfaceLanguage: string;
    model: string;
    outputSpeed: string;
    platformFormat: string;
    presentation: string;
    providerName: string;
    providerBaseUrl: string;
    requestTimeoutSeconds: string;
    streamingEnabled: string;
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
  platformFormat: Record<'acm' | 'leetcode' | 'generic', string>;
  notices: {
    backendNotResponding: string;
    resetFailed: string;
    resetSucceeded: string;
    saveFailed: string;
    saveSucceeded: string;
    windowActionFailed: string;
    shortcutListenerFailed: string;
    shortcutTriggered: string;
  };
  outputSpeed: Record<OutputSpeed, string>;
  result: {
    body: string;
    title: string;
  };
  resultPanel: {
    clearResult: string;
    clearResultNotice: string;
    codeCopied: string;
    copyCode: string;
    copyFullAnswer: string;
    copyFullAnswerNotice: string;
    failedStatus: string;
    fullAnswerCopied: string;
    generatingStatus: string;
    idleHint: string;
    recognizingStatus: string;
    regenerate: string;
    regenerateNotice: string;
    ragContextAlgorithmTags: string;
    ragContextCollapse: string;
    ragContextExpand: string;
    ragContextIgnore: string;
    ragContextIgnoredNotice: string;
    ragContextKindLabels: Record<RagPromptContextKind, string>;
    ragContextNoVisibleItems: string;
    ragContextReason: string;
    ragContextScore: string;
    ragContextSkipped: string;
    ragContextSourceLabels: Record<RagPromptContextSourceType, string>;
    ragContextSubtitle: string;
    ragContextTags: string;
    ragContextTitle: string;
    ragContextTokens: string;
    ragContextUsed: string;
    screenshotStatus: string;
    switchLanguage: string;
    switchLanguageNotice: string;
    completeStatus: string;
  };
  runtime: {
    aiResultState: string;
    settingsPath: string;
    shortcutTriggers: string;
    screenshotState: string;
    startupStatus: string;
    subtitle: string;
    title: string;
    trayStatus: string;
    windowVisible: string;
  };
  screenshotState: Record<ScreenshotState, string>;
  privacy: {
    dataFlowDescription: string;
    dataFlowTitle: string;
    screenshotWarning: string;
    subtitle: string;
    title: string;
  };
  settings: {
    launchToTrayDescription: string;
    launchToTrayLabel: string;
    localRagDescription: string;
    localRagEnabledDescription: string;
    localRagEnabledLabel: string;
    localRagTitle: string;
    ragCodeTemplatesDescription: string;
    ragCodeTemplatesLabel: string;
    ragHistoryIndexingDescription: string;
    ragHistoryIndexingLabel: string;
    ragMaxRecallItemsDescription: string;
    ragMaxRecallItemsLabel: string;
    ragSimilarProblemsDescription: string;
    ragSimilarProblemsLabel: string;
    ragUserNotesDescription: string;
    ragUserNotesLabel: string;
    saveHistoryDescription: string;
    saveHistoryLabel: string;
    streamingEnabledDescription: string;
    streamingEnabledLabel: string;
    shortcutEnabledDescription: string;
    shortcutEnabledLabel: string;
    subtitle: string;
    title: string;
  };
  theme: Record<ThemePreference, string>;
  trayStatus: Record<TrayStatus, string>;
};

const zhCn: AppMessages = {
  actions: {
    clearCache: '清空缓存',
    clearCacheNotice: '临时缓存已清空。',
    clearHistory: '清空全部历史',
    clearHistoryNotice: '全部历史记录已清空。',
    deleteHistoryEntry: '删除',
    deleteHistoryEntryNotice: '历史记录已删除。',
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
  dataManagement: {
    clearRagIndex: '清除 RAG 索引',
    clearRagIndexNotice: 'RAG 索引已清除。',
    deleteRagImport: '删除导入资料',
    deleteRagImportNotice: '导入资料已删除。',
    disableHistoryIndexing: '关闭历史入库',
    disableHistoryIndexingNotice: '历史入库已关闭。',
    historyIndexingAlreadyDisabled: '历史入库已关闭。',
    importKindLabels: {
      solution: '题解',
      note: '笔记',
      template: '模板',
    },
    importUpdatedAt: '更新于',
    loading: '加载中...',
    noHistoryEntries: '暂无历史记录',
    noRagImports: '暂无导入资料',
    ragIndexSkippedEmpty: '没有可入库资料，索引保持为空。',
    ragIndexSkippedLocalRagDisabled: '本地知识增强已关闭，未重建索引。',
    ragIndexSkippedUnknown: '已跳过：',
    ragIndexStats:
      'embedding 清除 {clearedEmbedding} 条，重建 {rebuiltEmbedding} 条；历史文档清除 {clearedHistoryDocuments} 条，重建 {rebuiltHistoryDocuments} 条；历史分块清除 {clearedHistoryChunks} 条。',
    ragPrivacyDescription:
      '这些操作只处理本地 RAG 索引和导入资料；普通历史和导入原文会按各自入口单独管理。',
    ragPrivacyTitle: 'RAG 隐私控制',
    rebuildRagIndex: '重建 RAG 索引',
    rebuildRagIndexNotice: 'RAG 索引重建完成。',
    refreshHistory: '刷新历史记录',
    refreshRagImports: '刷新导入资料',
    title: '数据管理',
    untitledHistoryEntry: '无标题',
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
    apiKey: 'API Key',
    charactersPerSecond: '每秒字符数',
    defaultLanguage: '默认编程语言',
    globalShortcut: '全局快捷键',
    globalShortcutEnabled: '全局快捷键开关',
    interfaceLanguage: '界面语言',
    model: '模型',
    outputSpeed: '输出速度',
    platformFormat: '平台格式',
    presentation: '展示方式',
    providerName: '服务商名称',
    providerBaseUrl: '服务 Base URL',
    requestTimeoutSeconds: '请求超时时间（秒）',
    streamingEnabled: '启用流式输出',
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
  platformFormat: {
    acm: 'ACM (标准输入输出)',
    leetcode: 'LeetCode (函数签名)',
    generic: '通用 (函数模式)',
  },
  notices: {
    backendNotResponding: '桌面后端没有响应。',
    resetFailed: '重置设置失败。',
    resetSucceeded: '设置已恢复为默认本地配置。',
    saveFailed: '保存设置失败。',
    saveSucceeded: '设置已保存到本地配置。',
    windowActionFailed: '窗口操作失败。',
    shortcutListenerFailed: '快捷键事件监听失败，请重新加载应用。',
    shortcutTriggered: '全局快捷键已触发，截图流程将在后续阶段接入。',
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
  resultPanel: {
    clearResult: '清空结果',
    clearResultNotice: '结果已清空。',
    codeCopied: '代码已复制',
    copyCode: '复制代码',
    copyFullAnswer: '复制完整答案',
    copyFullAnswerNotice: '完整答案已复制。',
    failedStatus: '生成失败',
    fullAnswerCopied: '完整答案已复制',
    generatingStatus: '生成中...',
    idleHint: '按快捷键截图后，结果将显示在这里',
    recognizingStatus: '识别题目中...',
    regenerate: '重新生成',
    regenerateNotice: '正在重新生成...',
    ragContextAlgorithmTags: '算法标签',
    ragContextCollapse: '收起',
    ragContextExpand: '展开',
    ragContextIgnore: '忽略',
    ragContextIgnoredNotice: '已忽略上下文',
    ragContextKindLabels: {
      problemStatement: '题面',
      solution: '题解',
      note: '笔记',
      template: '模板',
      historySummary: '历史摘要',
      metadata: '元数据',
      solutionMode: '解题模式',
    },
    ragContextNoVisibleItems: '本次命中的上下文已全部忽略。',
    ragContextReason: '原因',
    ragContextScore: '置信度',
    ragContextSkipped: '未注入',
    ragContextSourceLabels: {
      leetcodeIndex: '相似题',
      userImport: '用户资料',
      history: '历史记录',
      codeTemplate: '本地模板',
      solutionMode: '解题模式',
    },
    ragContextSubtitle:
      '这些内容只作为本地参考；如果和截图识别内容冲突，以当前截图为准。',
    ragContextTags: '标签',
    ragContextTitle: '本地召回上下文',
    ragContextTokens: 'Token 估算',
    ragContextUsed: '已注入',
    screenshotStatus: '截图中...',
    switchLanguage: '切换语言',
    switchLanguageNotice: '已切换语言，正在重新生成...',
    completeStatus: '生成完成',
  },
  runtime: {
    aiResultState: 'AI 结果状态',
    settingsPath: '设置路径',
    shortcutTriggers: '快捷键触发次数',
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
  privacy: {
    dataFlowDescription:
      '1. 你按下快捷键后，软件截取当前屏幕。\n2. 软件在本地裁剪出题目区域，生成临时图片。\n3. 临时图片和你的配置一起发送给所配置的 AI 服务。\n4. AI 返回结果后，临时图片会被删除（除非你启用了保存历史）。\n5. 历史记录只保存在本地，包含题目文本、语言、模型和结果。',
    dataFlowTitle: '数据流向',
    screenshotWarning:
      '截图会发送给所配置的 AI 服务。请确保你使用的是可信的服务商，并且只在授权环境中使用。',
    subtitle: '了解你的数据如何被使用和存储。',
    title: '隐私说明',
  },
  settings: {
    launchToTrayDescription: '启动后隐藏窗口，并让应用保留在托盘。',
    launchToTrayLabel: '启动到托盘',
    localRagDescription:
      '只使用本机历史、导入资料、模板和轻量题目元数据增强解题上下文。',
    localRagEnabledDescription:
      '默认关闭；关闭后截图到 AI 的主流程继续按原方式运行。',
    localRagEnabledLabel: '启用本地知识增强',
    localRagTitle: '本地知识增强',
    ragCodeTemplatesDescription:
      '允许导入的代码模板参与模板选择和 prompt 上下文。',
    ragCodeTemplatesLabel: '代码模板检索',
    ragHistoryIndexingDescription:
      '普通历史开启时同步写入并召回历史 RAG 记录；常规历史快照仍由独立开关控制。',
    ragHistoryIndexingLabel: '历史入库',
    ragMaxRecallItemsDescription:
      '限制每次检索和 prompt 注入最多使用的本地上下文条数。',
    ragMaxRecallItemsLabel: '最大召回条数',
    ragSimilarProblemsDescription:
      '允许 LeetCode 轻量元数据提供相似题、题型和标签提示。',
    ragSimilarProblemsLabel: '相似题提示',
    ragUserNotesDescription: '允许导入的题解和笔记参与本地检索。',
    ragUserNotesLabel: '用户笔记检索',
    saveHistoryDescription: '保存本地答案快照，便于稍后回看。',
    saveHistoryLabel: '保存历史',
    streamingEnabledDescription:
      '在服务端支持时启用增量输出，更快看到逐步返回的内容。',
    streamingEnabledLabel: '启用流式输出',
    shortcutEnabledDescription: '允许应用在后台响应配置的全局快捷键。',
    shortcutEnabledLabel: '启用全局快捷键',
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
    clearCache: 'Clear cache',
    clearCacheNotice: 'Temporary cache cleared.',
    clearHistory: 'Clear all history',
    clearHistoryNotice: 'All history entries cleared.',
    deleteHistoryEntry: 'Delete',
    deleteHistoryEntryNotice: 'History entry deleted.',
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
  dataManagement: {
    clearRagIndex: 'Clear RAG index',
    clearRagIndexNotice: 'RAG index cleared.',
    deleteRagImport: 'Delete import',
    deleteRagImportNotice: 'Imported document deleted.',
    disableHistoryIndexing: 'Disable history indexing',
    disableHistoryIndexingNotice: 'History indexing disabled.',
    historyIndexingAlreadyDisabled: 'History indexing is already disabled.',
    importKindLabels: {
      solution: 'Solution',
      note: 'Note',
      template: 'Template',
    },
    importUpdatedAt: 'Updated',
    loading: 'Loading...',
    noHistoryEntries: 'No history entries',
    noRagImports: 'No imported documents',
    ragIndexSkippedEmpty:
      'No eligible material found; the index remains empty.',
    ragIndexSkippedLocalRagDisabled:
      'Local knowledge is disabled, so the index was not rebuilt.',
    ragIndexSkippedUnknown: 'Skipped: ',
    ragIndexStats:
      'Embeddings cleared: {clearedEmbedding}, rebuilt: {rebuiltEmbedding}; history documents cleared: {clearedHistoryDocuments}, rebuilt: {rebuiltHistoryDocuments}; history chunks cleared: {clearedHistoryChunks}.',
    ragPrivacyDescription:
      'These actions only manage the local RAG index and imported documents; normal history and imported source text keep their own controls.',
    ragPrivacyTitle: 'RAG privacy controls',
    rebuildRagIndex: 'Rebuild RAG index',
    rebuildRagIndexNotice: 'RAG index rebuilt.',
    refreshHistory: 'Refresh history',
    refreshRagImports: 'Refresh imports',
    title: 'Data management',
    untitledHistoryEntry: 'Untitled',
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
    apiKey: 'API key',
    charactersPerSecond: 'Characters per second',
    defaultLanguage: 'Default language',
    globalShortcut: 'Global shortcut',
    globalShortcutEnabled: 'Global shortcut toggle',
    interfaceLanguage: 'Interface language',
    model: 'Model',
    outputSpeed: 'Output speed',
    platformFormat: 'Platform format',
    presentation: 'Presentation',
    providerName: 'Provider name',
    providerBaseUrl: 'Provider base URL',
    requestTimeoutSeconds: 'Request timeout (seconds)',
    streamingEnabled: 'Enable streaming output',
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
  platformFormat: {
    acm: 'ACM (stdin/stdout)',
    leetcode: 'LeetCode (function)',
    generic: 'Generic (function)',
  },
  notices: {
    backendNotResponding: 'The desktop backend did not respond.',
    resetFailed: 'Failed to reset settings.',
    resetSucceeded: 'Settings were reset to the default local profile.',
    saveFailed: 'Failed to save settings.',
    saveSucceeded: 'Settings saved to the local config store.',
    windowActionFailed: 'Window action failed.',
    shortcutListenerFailed:
      'Shortcut event listening failed. Reload the app and try again.',
    shortcutTriggered:
      'Global shortcut triggered. The capture flow will be wired in a later stage.',
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
  resultPanel: {
    clearResult: 'Clear result',
    clearResultNotice: 'Result cleared.',
    codeCopied: 'Code copied',
    copyCode: 'Copy code',
    copyFullAnswer: 'Copy full answer',
    copyFullAnswerNotice: 'Full answer copied.',
    failedStatus: 'Generation failed',
    fullAnswerCopied: 'Full answer copied',
    generatingStatus: 'Generating...',
    idleHint: 'Press the shortcut to capture; results will appear here',
    recognizingStatus: 'Recognizing question...',
    regenerate: 'Regenerate',
    regenerateNotice: 'Regenerating...',
    ragContextAlgorithmTags: 'Algorithm tags',
    ragContextCollapse: 'Collapse',
    ragContextExpand: 'Expand',
    ragContextIgnore: 'Ignore',
    ragContextIgnoredNotice: 'Ignored contexts',
    ragContextKindLabels: {
      problemStatement: 'Problem statement',
      solution: 'Solution',
      note: 'Note',
      template: 'Template',
      historySummary: 'History summary',
      metadata: 'Metadata',
      solutionMode: 'Solution mode',
    },
    ragContextNoVisibleItems:
      'All matched contexts are ignored for this panel.',
    ragContextReason: 'Reason',
    ragContextScore: 'Confidence',
    ragContextSkipped: 'Skipped',
    ragContextSourceLabels: {
      leetcodeIndex: 'Similar problem',
      userImport: 'User import',
      history: 'History record',
      codeTemplate: 'Local template',
      solutionMode: 'Solution mode',
    },
    ragContextSubtitle:
      'These local hits are references only; if they conflict with the screenshot, use the screenshot.',
    ragContextTags: 'Tags',
    ragContextTitle: 'Local recalled context',
    ragContextTokens: 'Token estimate',
    ragContextUsed: 'Used',
    screenshotStatus: 'Capturing...',
    switchLanguage: 'Switch language',
    switchLanguageNotice: 'Language switched, regenerating...',
    completeStatus: 'Generation complete',
  },
  runtime: {
    aiResultState: 'AI result state',
    settingsPath: 'Settings path',
    shortcutTriggers: 'Shortcut triggers',
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
  privacy: {
    dataFlowDescription:
      '1. When you press the shortcut, the app captures your current screen.\n2. The app crops the question region locally and creates a temporary image.\n3. The temporary image and your configuration are sent to the configured AI service.\n4. After the AI responds, the temporary image is deleted (unless you enabled history).\n5. History is stored locally only, containing question text, language, model, and result.',
    dataFlowTitle: 'Data flow',
    screenshotWarning:
      'Screenshots are sent to the configured AI service. Make sure you use a trusted provider and only use this in authorized environments.',
    subtitle: 'Understand how your data is used and stored.',
    title: 'Privacy',
  },
  settings: {
    launchToTrayDescription:
      'Hide the window on launch and keep the app in the tray.',
    launchToTrayLabel: 'Launch to tray',
    localRagDescription:
      'Use only local history, imports, templates, and lightweight problem metadata to enrich solving context.',
    localRagEnabledDescription:
      'Off by default; when disabled, the screenshot-to-AI flow keeps working normally.',
    localRagEnabledLabel: 'Enable local knowledge',
    localRagTitle: 'Local knowledge',
    ragCodeTemplatesDescription:
      'Allow imported code templates to participate in template selection and prompt context.',
    ragCodeTemplatesLabel: 'Code template retrieval',
    ragHistoryIndexingDescription:
      'Write and recall history RAG records when history is saved; normal history saving still uses Save history.',
    ragHistoryIndexingLabel: 'History indexing',
    ragMaxRecallItemsDescription:
      'Limit how many local context items each retrieval and prompt injection can use.',
    ragMaxRecallItemsLabel: 'Max recall items',
    ragSimilarProblemsDescription:
      'Allow lightweight LeetCode metadata to provide similar problem, pattern, and tag hints.',
    ragSimilarProblemsLabel: 'Similar problem hints',
    ragUserNotesDescription:
      'Allow imported solutions and notes to participate in local retrieval.',
    ragUserNotesLabel: 'User note retrieval',
    saveHistoryDescription: 'Store the local answer snapshot for later review.',
    saveHistoryLabel: 'Save history',
    streamingEnabledDescription:
      'Enable incremental output when the provider supports streaming responses.',
    streamingEnabledLabel: 'Enable streaming output',
    shortcutEnabledDescription:
      'Allow the app to respond to the configured shortcut in the background.',
    shortcutEnabledLabel: 'Enable global shortcut',
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

/** 根据语言标识获取对应的文案对象，未知语言回退到中文。 */
// Returns a complete message set and falls back to Chinese for unknown persisted values.
export function getMessages(locale: UiLocale): AppMessages {
  return APP_MESSAGES[locale] ?? APP_MESSAGES.zhCn;
}
