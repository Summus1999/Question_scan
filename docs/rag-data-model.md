# 本地 RAG 数据模型设计

## 背景和目标

本设计服务于任务 `10.2`，用于定义 Question Scan 后续本地 RAG 能力的数据模型。RAG 只用于授权练习、自测、开放题目环境和个人工作流，帮助用户从自己的题解、错题笔记、代码模板、历史题目和轻量题目索引中召回上下文。

当前应用的设置和历史记录已经采用本地 JSON 文件持久化。RAG 第一版可以先按同样的版本化 JSON 文件落地，保证实现范围可控；当数据量、检索性能或并发写入需求增加后，再迁移到 SQLite。无论底层存储使用 JSON 还是 SQLite，业务层都应先遵守本文的逻辑模型、字段命名、删除语义和隐私边界。

## 范围边界

- RAG 默认关闭，只有用户主动开启后才参与截图解题流程。
- LeetCode 轻量题目索引默认只保存题号、标题、slug、难度、标签、题型和平台来源，不内置未授权完整题面。
- 用户导入资料可以包含完整题解、笔记和模板，但必须记录来源和删除入口。
- 历史入库必须依赖用户同时开启历史保存和历史入库，不保存完整截图。
- RAG 召回结果只能作为辅助上下文，不能覆盖当前截图识别内容和用户确认内容。
- 本设计不增加隐蔽触发、受限环境规避、自动提交、自动填写第三方网页或读取浏览器凭据能力。

## 用例

- 用户导入 Markdown、JSON 或 CSV 题解后，系统能记录来源、导入批次、删除状态，并生成可检索文档和分块。
- 用户开启历史入库后，系统能把识别题目、答案摘要、备注、语言、平台和模型记录成 RAG 文档，不保存完整截图。
- 用户触发截图解题时，系统能按题干、标题、标签和目标语言召回相似题、用户笔记、代码模板和历史记录，并记录召回来源与置信度。
- 用户关闭历史入库或清除索引时，系统能删除相关文档、分块、embedding 和召回记录。
- LeetCode 轻量索引只保存元数据；完整题面只能来自用户截图、用户导入或用户可控历史。
- 当本地索引为空、embedding 生成失败或召回分数低于阈值时，系统跳过 RAG，不阻塞原有截图解题流程。

## 存储策略

### 逻辑模型优先

RAG 模块先定义稳定的逻辑实体：`RagDocument`、`RagChunk`、`RagEmbedding`、`ProblemIndexEntry`、`CodeTemplate`、`HistoryRagLink`、`RetrievalRecord` 和 `RagSettings`。运行时代码不应把业务逻辑绑定到具体文件名或表名，而是通过存储接口读写这些实体。

### JSON 第一版

为了贴合当前仓库已有的本地设置和历史实现，第一版可使用版本化 JSON 文件：

- `%APPDATA%/com.questionscan.desktop/rag/manifest.json`
- `%APPDATA%/com.questionscan.desktop/rag/documents.json`
- `%APPDATA%/com.questionscan.desktop/rag/chunks.json`
- `%APPDATA%/com.questionscan.desktop/rag/embeddings.json`
- `%APPDATA%/com.questionscan.desktop/rag/problem-index.json`
- `%APPDATA%/com.questionscan.desktop/rag/code-templates.json`
- `%APPDATA%/com.questionscan.desktop/rag/history-links.json`
- `%APPDATA%/com.questionscan.desktop/rag/retrieval-log.json`

JSON 写入应沿用历史模块的写时复制策略：先写临时文件，再原子重命名。每个文件必须包含 `schemaVersion`，后续迁移时保留实体 `id`、`createdAt`、`updatedAt` 和来源字段。

当前 `10.5` 的第一版实现先把历史来源的 `RagDocument`、`RagChunk` 和 `HistoryRagLink` 合并保存在 `%APPDATA%/com.questionscan.desktop/rag/history-records.json`，保持写入路径小而可验证。后续接入 embedding 和统一召回时，可以按上面的逻辑文件拆分或迁移到 SQLite，但不得改变删除历史后不可召回的语义。

### SQLite 迁移目标

当索引规模扩大后，SQLite 可承载同一套逻辑模型。建议表名与实体保持对应：

- `rag_documents`
- `rag_chunks`
- `rag_embeddings`
- `problem_index_entries`
- `code_templates`
- `history_rag_links`
- `retrieval_records`
- `rag_settings`

迁移时不得改变用户数据控制语义。清除索引、删除导入资料、关闭历史入库和重建索引在 JSON 与 SQLite 下必须表现一致。

## 枚举定义

`sourceType`：

- `leetcodeIndex`：轻量题目索引，只存元数据。
- `userImport`：用户导入的题解、笔记或模板。
- `history`：用户开启历史入库后生成的历史文档。
- `template`：用户维护或系统生成的代码模板。
- `manualNote`：用户在应用内手动创建的笔记。

`chunkKind`：

- `problemStatement`：题干或截图识别出的题目文本。
- `solution`：题解思路或答案摘要。
- `note`：用户笔记。
- `template`：代码模板。
- `historySummary`：历史记录摘要。
- `metadata`：题号、标题、标签、难度、平台等元数据。

`privacyScope`：

- `localOnly`：仅本地保存和检索。
- `allowedInPrompt`：允许作为本地上下文注入 AI prompt。
- `metadataOnly`：只允许展示和检索元数据，不注入完整文本。

`indexStatus`：

- `pending`：等待分块或 embedding。
- `indexed`：已入库并可召回。
- `failed`：处理失败，可重试。
- `disabled`：因用户设置关闭而不参与召回。
- `deleted`：已删除，不可召回。

## 实体模型

### RagDocument

代表一份可检索资料的顶层文档。导入题解、用户笔记、历史题目和模板都先抽象为文档，再拆分为分块。

字段：

- `id: string`，稳定唯一标识，建议使用 `ragdoc_` 前缀。
- `sourceType: SourceType`，来源类型。
- `sourceId?: string`，来源系统内的原始 id，例如历史记录 id、导入批次 id、模板 id。
- `sourceUri?: string`，用户导入文件路径、逻辑 URI 或平台 slug；不得保存浏览器 Cookie、账号或凭据。
- `title: string`，文档标题。
- `text?: string`，可选完整文本。轻量题目索引默认不填完整题面。
- `language?: LanguageId`，关联编程语言。
- `platform?: PlatformFormat`，平台格式，例如 LeetCode、ACM、牛客。
- `tags: string[]`，普通标签。
- `algorithmTags: string[]`，算法标签，例如 `dynamic-programming`、`graph`、`two-pointers`。
- `difficulty?: "easy" | "medium" | "hard" | "unknown"`，题目难度。
- `locale?: "zh-CN" | "en-US" | "unknown"`，文本语言。
- `privacyScope: PrivacyScope`，隐私使用范围。
- `checksum: string`，基于规范化内容计算，用于去重和变更检测。
- `createdAt: string`，ISO 8601 时间。
- `updatedAt: string`，ISO 8601 时间。
- `deletedAt?: string`，软删除时间。软删除后不得参与召回。

约束：

- `sourceType = leetcodeIndex` 时，`privacyScope` 默认是 `metadataOnly`。
- `sourceType = history` 时，`sourceId` 必须指向 `HistoryEntry.id`。
- `deletedAt` 存在时，其下分块和 embedding 必须在清理任务中删除或标记不可用。

### RagChunk

代表一个可被 embedding 和召回的文本分块。

字段：

- `id: string`，稳定唯一标识，建议使用 `ragchunk_` 前缀。
- `documentId: string`，所属 `RagDocument.id`。
- `chunkIndex: number`，文档内顺序。
- `kind: ChunkKind`，分块类型。
- `text: string`，分块文本。
- `tokenEstimate: number`，估算 token 数。
- `title?: string`，分块标题或局部标题。
- `language?: LanguageId`，模板或解法语言。
- `tags: string[]`，继承或补充标签。
- `algorithmTags: string[]`，继承或补充算法标签。
- `metadata: Record<string, string | number | boolean>`，少量可序列化元数据。
- `createdAt: string`，ISO 8601 时间。
- `deletedAt?: string`，删除时间。

约束：

- 单个分块应有明确上限，第一版建议 `tokenEstimate <= 512`。
- `metadata` 不得存储 API Key、用户 token、浏览器凭据或完整截图路径。
- `kind = metadata` 的分块可参与关键词检索，但默认不注入完整 prompt。

### RagEmbedding

代表某个分块在指定 embedding provider 和模型下的向量。

字段：

- `id: string`，稳定唯一标识，建议使用 `ragemb_` 前缀。
- `chunkId: string`，所属 `RagChunk.id`。
- `provider: string`，embedding 服务名，例如 `OpenAI-compatible` 或本地 provider 名称。
- `model: string`，embedding 模型名。
- `dimension: number`，向量维度。
- `vector: number[]`，向量值。JSON 第一版可直接保存数组，SQLite 迁移时可使用 blob 或扩展表。
- `vectorHash: string`，向量和模型参数哈希，用于缓存命中。
- `createdAt: string`，ISO 8601 时间。
- `lastUsedAt?: string`，最近召回时间。

约束：

- 同一 `chunkId + provider + model` 只保留一条当前 embedding。
- 分块文本、provider、model 或维度变化后必须重新生成。
- embedding 生成失败不能阻塞主解题流程。

### ProblemIndexEntry

代表轻量题目索引项，用于标题、slug、标签和题型召回。

字段：

- `id: string`，稳定唯一标识，建议使用 `problemidx_` 前缀。
- `platform: PlatformFormat`，题目平台。
- `problemNumber?: string`，题号。
- `slug: string`，平台 slug。
- `title: string`，题目标题。
- `difficulty: "easy" | "medium" | "hard" | "unknown"`，难度。
- `tags: string[]`，平台标签。
- `algorithmTags: string[]`，规范化算法标签。
- `locale?: "zh-CN" | "en-US" | "unknown"`，标题语言。
- `source: string`，索引来源说明，例如内置轻量索引版本或用户导入文件。
- `sourceLicense?: string`，来源授权说明。
- `updatedAt: string`，ISO 8601 时间。
- `deletedAt?: string`，删除时间。

约束：

- 不保存未授权完整题面。
- 可以生成 `RagDocument` 的 `metadata` 分块，但 `privacyScope` 必须是 `metadataOnly`。
- 后续更新索引时，以 `platform + slug` 作为幂等键。

### CodeTemplate

代表可被召回的代码模板或解题套路。

字段：

- `id: string`，稳定唯一标识，建议使用 `codetpl_` 前缀。
- `title: string`，模板标题。
- `language: LanguageId`，模板语言。
- `platform?: PlatformFormat`，适用平台格式。
- `algorithmTags: string[]`，适用算法标签。
- `templateText: string`，模板内容。
- `sourceDocumentId?: string`，来自导入文档时指向 `RagDocument.id`。
- `createdAt: string`，ISO 8601 时间。
- `updatedAt: string`，ISO 8601 时间。
- `deletedAt?: string`，删除时间。

约束：

- 模板召回必须优先匹配用户选择的目标语言。
- 模板可以同时作为 `RagDocument` 和 `RagChunk` 存在，但展示层应以 `CodeTemplate.id` 作为模板操作入口。

### HistoryRagLink

连接现有历史记录与 RAG 文档，避免把历史存储和 RAG 存储强耦合。

字段：

- `historyEntryId: string`，现有 `HistoryEntry.id`。
- `documentId: string`，对应 `RagDocument.id`。
- `indexedAt: string`，入库时间。
- `indexStatus: IndexStatus`，当前状态。
- `disabledReason?: string`，关闭或失败原因。

约束：

- 历史记录被删除时，相关 `RagDocument`、`RagChunk` 和 `RagEmbedding` 必须删除或标记不可召回。
- 关闭历史入库后，不再创建新的 `HistoryRagLink`，已有链接按用户选择保留、清除或重建。

### RetrievalRecord

记录一次召回行为，用于结果面板来源展示、调试和隐私审计。

字段：

- `id: string`，稳定唯一标识，建议使用 `retr_` 前缀。
- `requestId: string`，截图解题请求 id 或流水线 id。
- `queryTextHash: string`，查询文本哈希，不默认保存完整查询。
- `queryTitle?: string`，可选题目标题。
- `platform?: PlatformFormat`，识别出的平台格式。
- `language?: LanguageId`，目标语言。
- `matchedChunkIds: string[]`，命中的分块 id。
- `scores: Record<string, number>`，以 `chunkId` 为键的分数。
- `sourceTypes: Record<string, SourceType>`，以 `chunkId` 为键的来源类型。
- `usedInPrompt: string[]`，实际注入 prompt 的分块 id。
- `skippedReason?: string`，跳过 RAG 的原因。
- `createdAt: string`，ISO 8601 时间。

约束：

- 第一版默认只保存哈希和命中元数据，避免长期保存完整查询文本。
- 清除 RAG 索引时，应同步清除召回记录，除非用户明确选择保留诊断日志。

### RagSettings

RAG 设置可以作为 `AppSettings` 的嵌套字段，也可以先独立保存在 `manifest.json` 中。

字段：

- `enabled: boolean`，本地知识增强总开关，默认 `false`。
- `historyIndexingEnabled: boolean`，历史入库开关，默认 `false`。
- `userImportEnabled: boolean`，用户导入资料检索开关，默认 `true`。
- `templateRetrievalEnabled: boolean`，代码模板检索开关，默认 `true`。
- `problemIndexEnabled: boolean`，轻量题目索引开关，默认 `true`。
- `maxTopK: number`，最大召回条数，第一版建议默认 `5`。
- `minScore: number`，最低置信度阈值，第一版建议默认 `0.72`。
- `maxContextTokens: number`，注入 prompt 的最大本地上下文预算，第一版建议默认 `1200`。
- `embeddingProvider: string`，embedding provider 名称。
- `embeddingModel: string`，embedding 模型名。
- `retrievalLogEnabled: boolean`，召回记录开关，默认 `true`，但只保存哈希和元数据。

约束：

- `enabled = false` 时，RAG 不参与主流程。
- `historyIndexingEnabled = false` 时，不新增历史 RAG 文档。
- 所有开关必须能从设置页关闭，并能反映到下一次截图解题流程。

## 关系

- `RagDocument 1:N RagChunk`：一份文档拆成多个分块。
- `RagChunk 1:0..N RagEmbedding`：同一分块可对应多个 provider 或模型的 embedding。
- `HistoryEntry 1:0..1 RagDocument`：通过 `HistoryRagLink` 建立弱关联。
- `ProblemIndexEntry 1:0..1 RagDocument`：轻量题目索引可映射为元数据文档。
- `CodeTemplate 0..1:1 RagDocument`：模板可以来自独立创建，也可以从导入文档提取。
- `RetrievalRecord N:N RagChunk`：一次召回可命中多个分块，一个分块可被多次召回。

## 召回输入输出契约

### 输入

`RagQuery`：

- `requestId: string`
- `recognizedTitle?: string`
- `recognizedText?: string`
- `platform?: PlatformFormat`
- `targetLanguage: LanguageId`
- `tags?: string[]`
- `algorithmTags?: string[]`
- `maxTopK: number`
- `minScore: number`
- `maxContextTokens: number`

### 输出

`RagContextItem`：

- `chunkId: string`
- `documentId: string`
- `sourceType: SourceType`
- `title: string`
- `snippet: string`
- `score: number`
- `kind: ChunkKind`
- `language?: LanguageId`
- `platform?: PlatformFormat`
- `usedInPrompt: boolean`
- `reason: string`

### 排序原则

- 先过滤 `deletedAt`、`indexStatus != indexed` 和低于 `minScore` 的内容。
- 当前截图标题和题干匹配权重最高。
- 用户笔记和历史题目高于轻量题目索引元数据。
- 目标语言匹配的代码模板高于其他语言模板。
- 总 token 预算不能超过 `maxContextTokens`。

## Prompt 注入契约

解题 prompt 必须明确分隔当前截图识别内容和本地召回上下文：

```text
当前截图识别内容：
...

本地召回上下文：
- 来源：用户笔记 / 历史题目 / 代码模板 / 轻量题目索引
- 标题：
- 置信度：
- 摘要：
...

规则：
1. 优先相信当前截图识别内容和用户确认内容。
2. 本地召回上下文只作为参考。
3. 如果上下文和当前题目冲突，以当前题目为准。
```

低于阈值的召回项不得注入 prompt。结果面板应展示注入了哪些上下文、哪些被过滤，以及过滤原因。

## 删除和隐私规则

- 删除单条导入资料：删除或软删除对应 `RagDocument`，同步删除其 `RagChunk`、`RagEmbedding` 和关联召回记录。
- 清除 RAG 索引：清空文档、分块、embedding、历史链接和召回记录；轻量题目索引可按用户选择保留或重新下载。
- 关闭历史入库：停止新增 `HistoryRagLink` 和历史文档；已有历史 RAG 数据按用户选择保留或清除。
- 清空历史记录：清空历史数据时，必须清理对应历史 RAG 文档和 embedding。
- 重建索引：保留原始用户导入文档和设置，重新分块并生成 embedding。
- 查询日志：默认不保存完整截图、完整 prompt 或完整查询文本，只保存哈希、命中 id、分数、来源类型和时间。
- 远端 embedding：如果使用远端 embedding provider，设置页必须提示用户被发送的是分块文本，而不是完整截图；用户关闭后不再发送新分块。

## 版本和迁移

每个 RAG 存储文件或表都必须携带 `schemaVersion`。第一版建议：

- `schemaVersion = 1`
- 所有时间使用 ISO 8601 字符串。
- 所有实体 id 使用稳定前缀，迁移到 SQLite 时不重建 id。
- JSON 写入使用临时文件和重命名。
- 迁移失败时保留原文件，并在结果面板或设置页给出可理解错误。

如果从 JSON 迁移到 SQLite：

- 先读取 JSON 并校验 `schemaVersion`。
- 写入 SQLite 临时数据库。
- 校验实体数量、外键关系和 checksum。
- 成功后把 JSON 移动到备份路径。
- 失败时回退到 JSON 读写，不影响主解题流程。

## 后续实现切分建议

- `10.3` 实现 `ProblemIndexEntry` 和轻量索引导入。
- `10.4` 实现 `RagDocument`、导入批次、来源记录和删除入口。
- `10.5` 实现 `HistoryRagLink`，把用户可控历史转为可检索文档。
- `10.6` 实现 `RagChunk`、`RagEmbedding` 和向量检索。
- `10.7` 实现 `CodeTemplate` 和模板召回。
- `10.8` 实现 `RagContextItem` 裁剪和 prompt 注入。
- `10.9` 在结果面板展示 `RetrievalRecord` 的来源、置信度和使用状态。
- `10.10` 和 `10.11` 实现 `RagSettings` 与隐私控制。

## 测试建议

- 题目索引解析：验证题号、slug、标题、难度、标签和来源字段。
- 用户导入校验：验证 Markdown、JSON、CSV 的必填字段、重复导入和 checksum 去重。
- 分块：验证长题解被稳定切分，分块 token 上限生效。
- embedding 缓存：验证相同文本、provider 和模型命中缓存，文本变化后重新生成。
- 删除级联：验证删除导入资料或历史记录后，文档、分块、embedding 和召回记录不再可用。
- 隐私关闭：验证 RAG 总开关和历史入库开关关闭后不新增检索数据。
- 排序稳定性：验证相同输入下 top K 排序可重复，低分项不会注入 prompt。
- 空索引降级：验证索引为空或 embedding 失败时，截图解题流程继续执行。
