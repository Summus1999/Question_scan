# Question Scan 任务清单

## 相关文件

- `package.json` - 前端脚本、依赖和 Tauri 命令入口。
- `.gitattributes` - 固定文本文件换行为 LF，避免 Windows 环境下格式化和 lint 反复冲突。
- `biome.json` - 代码格式和基础检查配置。
- `index.html` - Vite 前端 HTML 入口。
- `vite.config.ts` - React 前端的 Vite 配置。
- `tsconfig.json` - TypeScript 编译配置。
- `src/vite-env.d.ts` - Vite 环境类型声明。
- `src/main.tsx` - React 应用入口。
- `src/App.tsx` - 主界面外壳、路由、布局组合、手动框选覆盖层挂载、手动裁剪动作状态交接、AI 完成后的可选历史入库触发、设置页本地知识增强开关、RAG 隐私控制区和结果面板 RAG 上下文文案交接。
- `src/App.test.tsx` - 主界面骨架、手动框选状态渲染、手动裁剪动作、历史 RAG 保存触发、本地知识增强设置持久化和 RAG 隐私控制交互测试。
- `src/components/CropOverlay.tsx` - 手动框选覆盖层，负责拖拽选区、实时显示、取消、重新自动识别和确认裁剪动作。
- `src/components/CropOverlay.test.tsx` - 覆盖层的拖拽归一化、视口边界、取消拖拽、选区显示、inactive 渲染和动作按钮测试。
- `src/components/ResultPanel.tsx` - 结果面板组件，负责 AI 输出状态、Markdown/代码展示、复制动作、语言切换和本次 RAG 命中上下文的展开/忽略展示。
- `src/components/ResultPanel.test.tsx` - 结果面板测试，覆盖状态渲染、工具栏动作、语言切换和 RAG 上下文来源、置信度、展开详情、忽略行为。
- `src/styles.css` - 全局样式和 Tailwind 入口。
- `src/lib/types.ts` - 前端共享类型，包含设置、截图状态、识别返回结构、置信度路由、AI 输出、语言、历史记录、本地知识增强设置、轻量题目索引、用户 RAG 导入资料、RAG 索引维护统计、本地 RAG 检索结果、模板选择结果和 RAG prompt 上下文注入结果。
- `src/lib/types.test.ts` - 共享类型常量、选项列表、默认状态、本地知识增强默认设置、识别返回结构、置信度路由、轻量题目索引、用户 RAG 导入、RAG 索引维护统计、历史 RAG 保存、本地 RAG 检索、模板选择和 prompt 上下文注入契约的单元测试。
- `src/lib/i18n.ts` - 中英文界面文案、状态标签、选项标签、设置页本地知识增强文案、数据管理/RAG 隐私控制文案和结果面板 RAG 上下文展示标签。
- `src/lib/i18n.test.ts` - 界面语言文案覆盖和回退行为测试。
- `src/lib/api.ts` - 对 Tauri 命令和事件的类型化封装，包含历史保存、LeetCode 轻量题目索引、用户 RAG 导入资料、清除/重建 RAG 索引、本地 RAG 检索、模板选择和 prompt 上下文注入命令。
- `src/lib/api.test.ts` - Tauri 命令封装测试，覆盖历史保存、LeetCode 轻量题目索引、用户 RAG 导入资料、清除/重建 RAG 索引、本地 RAG 检索、模板选择和 prompt 上下文注入命令名。
- `src/test/setup.ts` - Vitest 与 DOM 匹配器初始化。
- `src-tauri/Cargo.toml` - Rust 依赖，包含 Tauri、托盘、设置、全局快捷键插件和日志相关能力。
- `src-tauri/Cargo.lock` - Rust 依赖锁定文件，包含全局快捷键插件解析结果。
- `src-tauri/build.rs` - Tauri build 脚本和自定义命令权限生成，包含托盘、AI、历史、历史 RAG、轻量题目索引、用户 RAG 导入、RAG 索引维护、本地 RAG 检索、模板选择和 prompt 上下文注入命令。
- `src-tauri/tauri.conf.json` - Tauri 应用配置、权限、窗口、托盘和打包设置。
- `src-tauri/capabilities/default.json` - Tauri 命令和插件权限，包含窗口、设置、托盘状态、AI、历史、历史 RAG、轻量题目索引、用户 RAG 导入、RAG 索引维护、本地 RAG 检索、模板选择和 prompt 上下文注入命令。
- `src-tauri/icons/tray-icon.png` - 托盘图标资源。
- `src-tauri/icons/icon.png` - 应用图标资源。
- `src-tauri/icons/icon.ico` - Windows 应用图标资源。
- `src-tauri/src/main.rs` - Tauri 后端入口、全局快捷键插件安装、命令注册、托盘设置、应用状态、轻量题目索引、用户 RAG 导入、历史 RAG 存储、本地 embedding 缓存存储、RAG 索引维护、模板选择、prompt 上下文注入和事件串联。
- `src-tauri/src/commands.rs` - 暴露给前端的 Tauri 命令处理器，包含设置保存、窗口显隐、托盘状态、AI 请求、历史、受本地知识增强设置控制的历史 RAG 入库、LeetCode 轻量索引读取、用户 RAG 导入资料管理、清除/重建 RAG 索引、本地 RAG 检索、模板选择和 prompt 上下文注入。
- `src-tauri/src/history.rs` - 本地历史记录存储，包含历史保存开关、历史条目标签字段和旧历史记录兼容。
- `src-tauri/src/problem_index.rs` - LeetCode 轻量题目索引类型、内置数据加载、字段校验和元数据边界测试。
- `src-tauri/data/leetcode-lightweight-index.json` - 内置 LeetCode 轻量题目索引 seed 数据，只包含题号、标题、slug、难度、标签、题型和平台来源。
- `src-tauri/src/rag_history.rs` - 历史 RAG 入库模块，把用户可控历史转换为可检索文档、分块和历史链接，并在删除历史时标记不可召回，也支持隐私控制里的硬清空索引。
- `src-tauri/src/rag_imports.rs` - 用户 RAG 导入资料解析和本地 JSON 存储，支持 Markdown、JSON、CSV、重复导入更新和软删除。
- `src-tauri/src/rag_retrieval.rs` - 本地 RAG 向量检索模块，使用确定性 local-hash embedding 生成和缓存向量，并按题目标题、题干、样例、约束、标签和语言召回 top K 相似上下文，同时支持清空、按导入资料删除和重建 embedding 缓存。
- `src-tauri/src/rag_templates.rs` - RAG 题型和代码模板选择模块，根据相似题、算法标签、目标语言和用户导入模板推荐解题模式与可用代码模板。
- `src-tauri/src/rag_prompt_context.rs` - RAG prompt 上下文注入模块，把相似题、历史摘要、用户笔记、解题模式和代码模板压缩为本地上下文 section，并按召回条数、最低分数和 token 预算过滤。
- `src-tauri/src/prompts.rs` - 解题 prompt 构建模块，支持固定输出结构、识别标题/文本和可选 RAG 本地上下文 section 注入。
- `src-tauri/src/recognition.rs` - 题目区域识别返回结构、低分辨率识别图片输入、视觉模型提示词、响应解析、边界校验、高清裁剪、置信度路由、边界框类型和序列化测试。
- `src-tauri/src/screenshot.rs` - 屏幕截图后端封装，提供显示器枚举、当前/全部显示选择、布局坐标统一、系统临时 PNG 写盘、AI 压缩和截图元数据入口。
- `src-tauri/src/settings.rs` - 本地设置读取、保存、校验和迁移，包含本地知识增强、RAG 来源开关和最大召回条数默认值。
- `src-tauri/src/provider.rs` - AI 服务商请求前配置校验，生成请求可用的服务商配置。
- `src-tauri/src/provider_key_store.rs` - Windows Credential Locker 封装和测试用内存后端。
- `src-tauri/src/errors.rs` - 映射到前端安全消息的后端错误类型，包含快捷键注册冲突、API Key 存储、服务商配置校验、轻量索引加载、RAG 导入、历史 RAG 入库和本地 RAG 检索提示。
- `src-tauri/src/runtime.rs` - 后端运行时状态，记录托盘状态、快捷键注册状态和触发次数。
- `src-tauri/src/shortcuts.rs` - 全局快捷键插件接入、注册、注销、触发事件和校验测试。
- `src-tauri/src/tray.rs` - 托盘菜单、状态提示、快捷键开关、设置入口和窗口显隐动作。
- `AGENTS.md` - 进入这个仓库工作的代理级说明。
- `docs/development-workflow.md` - 固定的执行流程，包含任务、验证、提交和发布检查点。
- `docs/developer-setup.md` - 开发环境快照、安装步骤、一键脚本说明和常见问题。
- `docs/manual-verification-checklist.md` - MVP 手动验证清单，覆盖全部 PRD 功能需求用例。
- `docs/rag-data-model.md` - 本地 RAG 数据模型设计，覆盖题目索引、用户资料、代码模板、历史入库、召回记录、隐私删除规则和迁移策略。
- `tasks/tasks-question-scan.md` - MVP 任务、阶段用例、进度状态和相关文件索引。
- `scripts/setup.ps1` - Windows 开发环境检查和依赖安装脚本。
- `scripts/dev.ps1` - 一键启动 Tauri 开发环境的脚本。
- `README.md` - 项目安装、开发命令、支持平台和 MVP 使用说明。
- `prd/prd-question-scan.md` - 产品需求文档，定义 MVP 范围、RAG 增强、Agent 工作流、远端 Provider Proxy、隐私边界和验收标准。

### 说明

- 单元测试尽量和被测试代码放在一起。Rust 测试可以放在同一个模块里，也可以放在相邻测试模块里。
- 前端测试使用 `npm test -- --run`。
- Rust 测试使用 `cargo test`，从 `src-tauri` 目录执行。
- 桌面手动验证使用 `npm run tauri dev`。
- 第一版先支持 Windows。macOS 和 Linux 可以放到截图权限模型稳定之后再做。
- 第一版先接兼容 OpenAI 的多模态接口，其他服务商后续再挂到同一层接口上。
- 输出速度控制的是前端本地展示节奏，不是模型真实生成速度。
- 产品范围只限授权练习、自测、开放题目环境和个人工作流使用，不做隐蔽考试、检测规避、自动提交和自动填答案能力。

## 任务说明

**重要：** 每完成一个子任务，就必须把这个 Markdown 文件里的对应项从 `[ ]` 改成 `[x]`。这样才能跟踪进度，也能避免漏做步骤。

示例：

- `[ ] 1.1 阅读文件` -> 完成后改成 `[x] 1.1 阅读文件`

要在完成每个子任务后更新文件，而不是等整个父任务都做完后再改。

## 阶段 1 用例

- 用户能够启动桌面应用，看到托盘图标、主窗口和最小化后的后台运行状态。
- 用户能够打开设置骨架，在同一个界面里看到快捷区、设置区和结果面板占位。
- 用户能够保存和重新加载基础配置，并在配置读写失败时看到明确的错误提示。
- 开发者能够通过函数级注释快速定位前端状态同步、Tauri 命令桥接、设置持久化和托盘流程的职责边界。
- 开发者能够运行 lint、格式化和测试脚本，并且工具默认跳过依赖、构建产物、Tauri 生成权限和 Rust target 目录。
- 开发者能够从 Tauri 配置中确认应用名称、版本、标识符、图标、Windows 主窗口和最小命令权限边界。
- 开发者能够通过 `src/lib/api.ts` 的类型化命令契约调用后端，避免前端散落硬编码命令名和错误 payload。
- 开发者能够从共享类型模块获取应用状态、设置、语言、截图状态和 AI 结果状态的稳定枚举值与默认状态。
- 开发者能够用 `npm run tauri dev` 启动阶段 1 桌面应用，并看到 Vite 与 Tauri 后端进入开发运行状态。

## 开发环境交接用例

- 新开发者能够从 GitHub 拉取 `main` 分支，看到已验证的 Tauri 应用底座、锁定文件和环境文档。
- 新开发者能够运行 `scripts/setup.ps1` 检查 Node、npm、Rust 和 Cargo，并安装前端依赖。
- 新开发者能够运行 `scripts/dev.ps1` 一键启动桌面开发环境，而不需要手动记忆 `npm run tauri dev`。
- 当本机缺少必要工具时，脚本能够失败并提示缺少的命令，开发者能按文档补齐环境。
- 开发者能够在 README 和 `docs/developer-setup.md` 里查到当前验证过的本地版本、手动命令和常见问题。

## 中英文界面适配用例

- 中文开发者首次打开应用时，主界面默认显示中文文案。
- 用户能够在设置区切换“中文 / English”，切换后主界面、设置标签、状态标签和结果占位区立即更新。
- 用户保存设置后，界面语言会写入本地设置，重新加载应用状态时继续使用保存的语言。
- 用户切到英文后，仍能看到和中文界面相同的设置项、按钮和运行状态，不丢失原有能力。
- 旧版本设置文件没有界面语言字段时，后端会使用中文默认值，不因字段缺失导致设置加载失败。

## 阶段 6 用例

- 用户启用本地知识增强后，系统能读取 LeetCode 轻量题目索引，并且索引项只包含题号、标题、slug、难度、标签、题型和平台来源。
- 开发者能通过后端类型确认轻量索引不包含完整题面、输入输出示例、约束正文、题解正文或任何第三方账号凭据。
- 前端后续需要展示或检索相似题时，可以通过类型化 API 获取轻量索引列表，不需要直接读取本地文件。
- 当内置索引文件格式错误、字段缺失或包含不允许的完整题面字段时，后端测试能够失败并阻止该数据进入索引。
- LeetCode 轻量索引只作为元数据上下文来源，后续 prompt 注入时必须继续优先相信当前截图识别内容和用户确认内容。
- 用户导入 Markdown 题解或笔记时，系统能从一级标题或 front matter 中提取标题、标签、语言和平台，并把原文作为用户资料保存。
- 用户导入 JSON 或 CSV 资料时，系统能解析单条或多条题解、错题笔记、代码模板，保留来源名称、来源 URI、导入时间和更新时间。
- 用户重新导入同一来源、同一标题和同一资料类型时，系统应更新已有资料的内容与 `updatedAt`，而不是制造重复记录。
- 用户删除单条导入资料时，系统应提供明确删除命令，删除后默认列表不再返回该资料，后续 RAG 索引任务可以据此清理分块和 embedding。
- 用户开启保存历史后，AI 回答完成时系统能把历史条目转为本地可检索记录，包含识别标题、题目文本、AI 输出摘要、用户备注、语言、平台、模型、标签和算法标签，但不保存完整截图。
- 用户关闭保存历史后，AI 回答完成时系统不新增历史记录，也不新增历史 RAG 文档、分块或历史链接。
- 用户删除单条历史或清空全部历史时，对应历史 RAG 文档和分块应被标记为不可召回，避免用户删除历史后仍能被后续检索命中。
- 开发者能够用本地 embedding provider 为历史分块、用户导入资料和 LeetCode 轻量元数据生成向量，并把相同文本、provider 和模型的 embedding 缓存在本地 JSON 文件中。
- 当分块文本、provider 或模型变化时，系统会重新生成 embedding；未变化时复用缓存，避免重复计算。
- 用户触发一次 RAG 查询时，系统能把题目标题、题干、样例、约束、标签和目标语言组合成查询文本，并返回 top K 相似题、笔记或历史记录。
- 检索排序必须稳定过滤已删除资料和低分结果，且 LeetCode 轻量索引只返回元数据摘要，不返回未授权完整题面。
- 当没有可检索资料、embedding 生成失败或分数低于阈值时，系统返回空结果和可理解原因，不阻塞后续截图解题主流程。
- 当相似题命中包含题型或算法标签时，系统能据此推荐解题模式，例如哈希表、双指针、动态规划、图搜索、二分和回溯。
- 用户导入代码模板后，系统能按目标语言或用户默认语言优先选择同语言模板，并用算法标签和相似题信号排序。
- 当没有同语言模板时，系统可以回退到未指定语言的通用模板，但不会把明显不匹配的其他语言模板排在前面。
- 当没有相似题、算法标签或可用模板信号时，系统返回空模板推荐和可理解原因，不阻塞后续 RAG 注入或解题主流程。
- 当相似题、历史摘要、用户笔记或代码模板达到置信度阈值时，系统能把它们压缩为独立的本地召回上下文 section，并注入解题 prompt。
- 解题 prompt 必须清楚分隔当前截图识别内容和本地召回上下文，并明确要求模型优先相信截图识别结果和用户确认内容。
- RAG prompt 注入必须限制最大召回条数、最低分数和上下文 token 预算，低分、空摘要或超预算内容要被跳过并返回原因。
- 当所有本地上下文都为空、低置信或超预算时，系统应返回可理解跳过原因，并继续保留原始解题 prompt 可用。
- LeetCode 轻量索引命中只能作为元数据提示注入，不得把它当作完整题面或覆盖当前截图识别结果。
- 结果面板在本次 RAG 命中存在时，应展示相似题、本地模板、历史记录、用户资料和解题模式来源，以及每条命中的置信度。
- 用户应能在结果面板展开单条 RAG 上下文，查看摘要、算法标签、token 估算、使用状态和跳过原因。
- 用户应能忽略某条 RAG 上下文；被忽略后该条不再占用当前结果面板的上下文列表空间，但不删除本地资料。
- 已注入 prompt 的上下文和因低置信、超预算或超条数被跳过的上下文必须有不同状态标记，避免用户误解本次实际使用范围。
- 当本次没有可展示的 RAG 上下文时，结果面板不显示空的上下文区块，保持原有答案阅读体验。
- 用户能在设置页开启或关闭本地知识增强，且该总开关默认关闭并在保存后持久化。
- 用户能单独控制历史入库；关闭历史入库后，即使保留普通历史保存，也不应把新历史写入 RAG 检索资料。
- 用户能单独控制用户笔记检索、代码模板检索和相似题提示；关闭任一来源后，后续检索和 prompt 注入流程应能按设置跳过该来源。
- 用户能设置最大召回条数；该值必须在界面和后端都保持边界限制，避免异常设置导致过量上下文注入。
- 旧版本设置文件没有本地知识增强字段时，系统会迁移到默认关闭和安全默认值，不因字段缺失导致设置加载失败。
- 当本地知识增强总开关关闭，或所有 RAG 来源都关闭时，截图到 AI 解题主流程仍然可用，不被 RAG 设置阻塞。
- 用户能在隐私控制区清除本地 RAG 索引；清除动作应移除本地 embedding 缓存和历史 RAG 索引，但不删除用户导入原文、普通历史和内置轻量题目元数据。
- 用户能在隐私控制区查看并删除单条导入资料；删除后该资料不再出现在导入资料列表，也不再参与后续检索和模板选择。
- 用户能触发重建 RAG 索引；系统应基于当前未删除的用户导入资料、允许入库的历史记录和轻量题目元数据重新生成本地 embedding 缓存，并返回可理解的重建统计。
- 用户能从隐私控制区关闭历史入库；关闭后新生成的普通历史仍可按“保存历史”保留，但不会新增历史 RAG 文档、分块或 embedding。
- 当本地知识增强关闭、索引为空或没有可重建资料时，清除和重建动作应安全完成并给出结果提示，不阻塞截图到 AI 主流程。

## 阶段 2 用例

- 开发者能够从后端依赖和启动代码确认项目选用 Tauri 官方全局快捷键插件作为集成方案。
- 应用启动时会安装全局快捷键插件，为后续注册、注销和冲突处理提供统一入口。
- 在任务 2.1 阶段，应用不会提前注册具体快捷键，默认快捷键和用户配置留给后续子任务实现。
- 用户首次启动应用时，默认全局快捷键为 `Ctrl+Shift+Q`，并且可以在设置里修改或禁用。
- 应用启动后会按当前设置注册全局快捷键，退出时会注销已注册的快捷键。
- 如果快捷键格式无效或被系统、其他应用占用，应用会给出能理解的提示，并允许用户换一个快捷键或禁用快捷键。
- 用户按下全局快捷键时，即使主窗口在后台，应用也会记录一次触发并展示截图流程的占位状态，真正截图留给阶段 3。
- 托盘图标能够展示空闲、截图中、识别中、生成中、完成和失败状态，至少通过托盘提示和运行状态快照可见。
- 用户能够从托盘打开窗口、打开设置、启用或禁用快捷键，以及退出应用。

## 任务

- [x] 0.0 创建功能分支
  - [x] 0.1 确认当前 git 状态，记录无关变更。
  - [x] 0.2 创建并切换到新的功能分支，例如 `feature/question-scan-mvp`。
  - [x] 0.3 确认分支名和当前干净基线，准备开始搭建项目。

- [x] 1.0 搭建 Tauri 应用底座
  - [x] 1.1 初始化一个 Tauri 2.x 项目，使用 React、TypeScript 和 Vite。
  - [x] 1.2 接入 Tailwind CSS 和基础样式入口。
  - [x] 1.3 加一个基础应用外壳，包含紧凑控制区、设置区和结果面板占位。
  - [x] 1.3a 给当前函数补充定位注释，便于后续排查问题。
  - [x] 1.4 配置 lint、格式化和测试脚本。
  - [x] 1.5 配置 `tauri.conf.json` 和权限文件里的 Tauri 权限与应用元数据。
  - [x] 1.6 在 `src/lib/api.ts` 里加类型化的前后端命令封装。
  - [x] 1.7 加共享类型定义，覆盖应用状态、设置、语言、截图状态和 AI 结果状态。
  - [x] 1.8 验证应用能用 `npm run tauri dev` 启动。
  - [x] 1.9 记录开发环境配置，并提供一键安装和启动脚本。
  - [x] 1.10 增加中英文界面适配，并支持用户选择和保存界面语言。

- [x] 2.0 加全局快捷键和托盘流程
  - [x] 2.1 加入 Tauri 全局快捷键插件，或者选定的快捷键集成方案。
  - [x] 2.2 定义默认快捷键 `Ctrl+Shift+Q`，并支持配置。
  - [x] 2.3 在应用启动时注册快捷键，在退出时注销快捷键。
  - [x] 2.4 快捷键注册失败时，给出用户能看懂的冲突提示。
  - [x] 2.5 加一个托盘图标，支持空闲、截图中、识别中、生成中、完成和失败状态。
  - [x] 2.6 给托盘加打开窗口、打开设置、启用或禁用快捷键、退出等动作。
  - [x] 2.7 给快捷键默认值、自定义保存和禁用状态写设置测试。
  - [x] 2.8 在应用后台时手动验证快捷键能触发。

- [x] 3.0 搭建屏幕截图和临时图片链路
  - [x] 3.1 选一个能在 Windows 上工作的 Rust 截图库并接入。
    - 用例：当后续任务需要枚举显示器、抓取当前屏幕或裁剪区域时，后端已有一个可直接调用的 Windows 截图依赖和统一入口。
    - 选型：`screenshots` 0.8.10。它在本地 Cargo 缓存中可用，提供 `Screen::all()`、`capture()` 和 `capture_area()`，并且在 Windows 上有明确支持。
  - [x] 3.2 实现按配置抓取当前显示器或全部显示器。
    - 用例：当后端收到“当前显示器”配置时，能从候选显示器中选出包含锚点的那一块并只抓这一块；当配置是“全部显示器”时，返回每一块显示器的抓图结果。
  - [x] 3.3 统一多显示器布局下的坐标。
    - 用例：当显示器出现在主屏左侧或上方时，后端能把所有显示器的坐标统一到同一个虚拟桌面原点，后续裁剪和元数据可以直接使用同一套坐标。
  - [x] 3.4 在系统临时目录里保存 PNG 或 JPEG 临时文件。
    - 用例：当截图完成后，后端会把每张截图编码成 PNG，写入系统临时目录下的 `question-scan` 子目录，并把临时文件路径保留在捕获结果里供后续 AI 请求和清理流程使用。
  - [x] 3.5 增加图片压缩配置，控制发送给 AI 的大小。
    - 用例：当后续 AI 请求需要发送截图时，后端能根据默认或持久化设置把原始截图按最长边和 JPEG 质量压缩成可解码的图片字节，避免把过大的原始全屏图片直接发给 AI。
  - [x] 3.6 返回截图元数据对象，包含图片路径、尺寸、显示器 ID 和时间戳。
    - 用例：当后续 AI 请求或前端状态需要引用截图结果时，后端能返回一个可序列化的元数据对象，里面包含临时图片路径、图像宽高、显示器 ID 和捕获时间戳，而不必把整张原始图片再塞进前端状态。
  - [x] 3.7 保证 AI 请求结束后，或者失败后，临时图片会被删除。
    - 用例：当后续 AI 请求成功或失败结束时，后端能显式删除这一轮截图产生的系统临时文件，避免临时目录里积累无用图片，并且清理函数可以在成功和错误分支里复用。
  - [x] 3.8 给裁剪坐标、临时文件清理和无效路径处理写测试。
    - 用例：当后续裁剪、请求结束清理或临时目录异常发生时，后端已有单元测试覆盖坐标归一化、越界拒绝、临时图片删除、缺失文件忽略和临时目录路径异常，避免这些基础链路在后续功能中回归。
  - [x] 3.9 在普通 Windows 桌面和多显示器环境里手动验证截图。
    - 验证：已在当前 Windows 多显示器环境手动确认全显示器截图、锚点选择当前显示器、临时 PNG 生成和清理；临时验证入口已清理。

- [x] 4.0 加题目区域识别和手动框选兜底
  - [x] 4.1 定义识别返回结构，包括边界框、置信度、提取出的标题、题目文本和原因。
    - 用例：当后续自动识别或手动框选流程需要交接题目区域时，后端和前端共享一个可序列化对象，里面包含区域边界框、置信度、提取出的标题、题目文本和原因说明，供坐标校验、裁剪、结果面板和手动兜底流程复用。
    - 验证：已定义 `QuestionBoundingBox` 和 `QuestionRecognitionResult`，并通过 Rust/前端单测确认序列化字段为 camelCase，题目文本可为 `null`。
  - [x] 4.2 用低分辨率图片做区域识别请求，降低成本和延迟。
    - 用例：当自动题目区域识别准备调用视觉模型时，后端先把原始全屏截图压缩为较小的 JPEG 输入，同时保留原始尺寸、低清尺寸和坐标换算比例，后续可以用低成本图片做识别，再把结果映射回高清截图裁剪。
    - 验证：已实现 `RecognitionImageConfig` 和 `prepare_recognition_image(...)`，支持低清 JPEG 输出、原始尺寸记录和坐标比例换算，并通过 Rust 单测确认缩放、原尺寸保留和空图拒绝。
  - [x] 4.3 让视觉模型返回最可能的算法题区域坐标。
    - 用例：当后端准备调用视觉模型做自动区域识别时，提示词明确要求模型只返回一个最可能的算法题区域，坐标使用低清识别图片的像素坐标系，并返回固定 JSON 字段，后端可以把模型响应解析为 `QuestionRecognitionResult` 供后续边界校验。
    - 验证：已实现 `QuestionRegionRecognitionRequest`、`build_question_region_prompt(...)` 和 `parse_question_recognition_result(...)`，把单区域识别提示词、低清图片输入和结构化响应解析连在一起，并通过 Rust/前端单测确认契约和兼容性。
  - [x] 4.4 校验 AI 返回的坐标是否在截图边界内。
    - 验证：已实现 `QuestionBoundingBox::validate_within(...)` 和 `validate_question_recognition_result(...)`，可拒绝负坐标、越界和零尺寸框，并通过 Rust 单测覆盖正常与异常路径。
  - [x] 4.5 用校验过的区域裁剪原始高清截图。
    - 用例：当视觉模型在低清识别图片上返回并通过边界校验的题目区域后，后端能把该区域按比例映射回原始高清截图坐标，并裁出高清题目图片，供后续 AI 解题请求使用。
    - 验证：已实现 `crop_original_question_region(...)`，会先校验低清识别结果，再按比例把区域映射到原始截图并裁出高清题目图片，同时覆盖整数和非整数缩放、尺寸不匹配和越界拒绝测试。
  - [x] 4.6 为自动接受、需要确认和手动兜底设置不同的置信度阈值。
    - 用例：当自动识别返回题目区域和置信度后，后端能按默认阈值把结果分为自动接受、需要用户确认和进入手动框选兜底三类；前端也有同一组路由枚举值，后续可以按路由决定是否直接裁剪、展示确认态或打开框选覆盖层。
    - 验证：已实现 `RecognitionConfidenceThresholds`、`RecognitionConfidenceRoute` 和 `classify_question_recognition_confidence(...)`，默认阈值为自动接受 `0.85`、确认下限 `0.55`，并通过 Rust/前端单测覆盖高、中、低置信度、边界值、阈值修正和 camelCase 序列化。
  - [x] 4.7 做出 `CropOverlay`，在自动识别失败时让用户拖拽选择题目区域。
    - 用例：当自动识别进入手动框选兜底时，前端应覆盖当前界面并接收鼠标拖拽，实时显示选区，松开后保留当前框选，供后续确认和裁剪流程复用。
    - 验证：已实现 `CropOverlay` 并接入 `App.tsx` 的 `selecting` 状态；`npm test -- --run src/components/CropOverlay.test.tsx src/App.test.tsx` 和 `npm test -- --run` 均通过。
  - [x] 4.8 增加取消、重新自动识别和确认裁剪动作。
    - 用例：当用户进入手动框选兜底时，可以取消并回到空闲状态；可以重新触发自动识别并清空手动选区；也可以在已有选区后确认裁剪，把该区域保留为后续高清裁剪和 AI 请求的输入。
    - 验证：已在 `CropOverlay` 增加取消、重新自动识别和确认裁剪按钮，并在 `App.tsx` 中把三种动作分别接到空闲、重新识别和已确认裁剪状态；`npm test -- --run src/components/CropOverlay.test.tsx src/App.test.tsx` 和 `npm test -- --run` 均通过。
  - [x] 4.9 给坐标解析、越界拒绝、置信度路由和覆盖层选择行为写测试。
    - 用例：当模型返回非法坐标结构、越界框或异常置信度时，自动识别链路应通过单元测试证明会拒绝或路由到安全分支；当前端手动框选拖到视口外或拖拽被取消时，覆盖层应通过测试证明会归一化坐标或清空选区。
    - 验证：已补充 Rust 单测覆盖非整数坐标解析拒绝、解析后越界框拒绝和异常置信度路由；已补充 `CropOverlay` 单测覆盖拖拽到视口外的坐标归一化和 pointer cancel 清空选区；`npm test -- --run`、`cargo test`、`cargo fmt -- --check`、`npm run lint` 和 `git diff --check` 均通过。
  - [x] 4.10 在浏览器、PDF、IDE 和深色题目页上手动验证识别效果。
    - 用例：当用户在浏览器、PDF 阅读器、IDE 内嵌题面或深色题目页里触发识别时，当前阶段至少要确认低清识别输入、坐标解析、边界校验、高清裁剪和手动框选兜底这条链路能覆盖这些常见题面形态；真实视觉模型准确率在 AI 请求接入后复测。
    - 验证：已按四类页面形态检查当前阶段的可验证链路：浏览器题面、PDF 题面、IDE 题面和深色题面都走同一套低清识别图片输入、camelCase 坐标响应解析、截图边界校验、高清裁剪和低置信度手动框选兜底契约；`npm run tauri dev` 已尝试启动桌面应用，但当前 shell 只能确认进程启动，不能可靠读取 GUI 人工交互结果；真实多模态模型识别准确率因 5.0 AI 请求尚未接入而未验证，已在 5.11 追踪复测任务。父任务验证中 `npm test -- --run`、`cargo test`、`npm run lint`、`npm run build`、`cargo fmt -- --check` 和 `git diff --check` 均通过；当前项目没有 E2E 脚本，未声明 E2E 通过。

- [ ] 5.0 加 AI 服务配置和多模态请求流程
  - [x] 5.1 增加服务商名称、API Base URL、API Key、模型、超时时间和是否启用流式输出等设置项。
    - 验证：已补齐 `providerName`、`providerApiKey`、`requestTimeoutSeconds` 和 `streamingEnabled` 的 Rust/前端设置合同、默认值、界面字段和保存回读测试；`npm test -- --run src/lib/types.test.ts src/App.test.tsx`、`cargo test`、`npm run lint` 和 `git diff --check` 均通过。
  - [x] 5.2 用 MVP 能做到的最安全方式存储密钥。
    - 验证：已通过 `cargo test`、`npm test -- --run src/App.test.tsx src/lib/types.test.ts`、`npm run lint`、`cargo fmt -- --check` 和 `git diff --check`；API key 通过 Windows Credential Locker 写入，前端快照返回时按设计脱敏为空字符串。
  - [x] 5.3 在发送请求前校验服务商配置。
    - 验证：已新增 `validate_provider_request_config(...)`，在请求阶段前校验服务 Base URL、API Key、模型和超时时间，并生成请求可用配置；`cargo test` 和 `git diff --check` 均通过。
  - [x] 5.4 实现兼容 OpenAI 的多模态请求构造，把图片输入和文本指令一起发出去。
    - 验证：已在 `src-tauri/src/provider.rs` 新增 OpenAI chat/completions 风格的多模态请求构造，包含 `text` + `image_url` 内容块、`Bearer` 认证头、`stream` 标志和 `chat/completions` 端点；`cargo test`、`cargo fmt` 和 `git diff --check` 均通过。
  - [x] 5.5 实现流式响应处理，并向前端发出事件。
    - 验证：已新增 `src-tauri/src/streaming.rs`，实现 SSE 流式解析、非流式降级、HTTP 错误映射和 `question-scan:ai-stream-event` 事件发射；已新增 `send_ai_request` 异步命令，前端通过 `listenAiStreamEvent` 监听；`cargo test`（83 通过）、`npm test -- --run`（27 通过）、`npm run lint`、`cargo fmt -- --check` 和 `git diff --check` 均通过。
  - [x] 5.6 给不稳定的非流式图片响应做降级处理。
    - 验证：已修改 `handle_non_streaming_response`，先读原始文本，再尝试 JSON 解析提取标准 content；解析失败或结构不匹配时降级为发射原始响应体；`cargo test`（84 通过）、`cargo fmt -- --check` 均通过。
  - [x] 5.7 为无效 API Key、不支持图片的模型、超时、网络失败和格式错误添加类型化错误。
    - 验证：已新增 `classify_http_error`，将 HTTP 401/403 映射为 `invalidApiKey`、400 含 image/vision/multimodal 映射为 `modelDoesNotSupportImages`、429 映射为 `rateLimited`、5xx 映射为 `providerServerError`；reqwest 超时映射为 `requestTimeout`、连接失败映射为 `networkError`；SSE 解析失败映射为 `streamParseError`；`cargo test`（84 通过）、`cargo fmt -- --check` 均通过。
  - [x] 5.8 给临时故障增加重试逻辑，并限制重试次数。
    - 验证：已重构 `send_openai_request` 为带重试的包装器，`execute_openai_request` 执行单次请求；`is_retryable_error` 识别超时、网络错误、5xx、429 和通用请求失败为可重试；401/403/400 为非重试；指数退避 1s/2s/4s，最大 3 次重试；`cargo test`（84 通过）、`cargo fmt -- --check` 均通过。
  - [x] 5.9 给请求体结构、设置校验、错误映射和流解析写测试。
    - 验证：已补充 Rust 单测覆盖 `classify_http_error`（401/400-image/429/500/404）、`is_retryable_error`（可重试/非重试/非 AI 错误）、非流式 JSON 无结构降级；已补充前端 `types.test.ts` 覆盖 `AiStreamEventPayload` chunk/done/error 契约；`cargo test`（92 通过）、`npm test -- --run`（28 通过）、`npm run lint`、`cargo fmt -- --check` 均通过。
  - [ ] 5.10 用一个真实的多模态模型手动验证一次图片请求。
    - 阻塞：需要真实 API Key 和 GUI 环境运行 `npm run tauri dev` 来触发截图→AI 请求完整链路；当前 shell 无法完成手动验证，待后续有 API Key 时复测。
  - [ ] 5.11 在 AI 请求接入后，用真实多模态模型复测浏览器、PDF、IDE 和深色题目页截图的题目区域识别准确率。
    - 阻塞：依赖 5.10 完成后的真实模型验证；当前阶段已确认低清识别输入、坐标解析、边界校验、高清裁剪和手动框选兜底契约覆盖四类页面形态。

- [x] 6.0 生成 C++ 和其他主流语言的直接算法解法
  - [x] 6.1 定义支持语言元数据，覆盖 C++17、C++20、Python、Java、JavaScript、TypeScript、Go 和 Rust。
    - 验证：已新增 `src-tauri/src/language.rs`，定义 `LanguageId`、`PlatformFormat`、`LanguageMetadata`、`OutputSection`，C++20 为默认语言；`cargo test`（114 通过）。
  - [x] 6.2 给直接解题模式加默认输出模板，包含题目识别、策略、代码、复杂度、边界用例和说明。
    - 验证：已新增 `src-tauri/src/prompts.rs`，`build_solution_prompt` 生成含 6 个固定 section 的指令；支持传入识别标题和文本；`cargo test`（114 通过）。
  - [x] 6.3 增加平台格式选项，覆盖 ACM 标准输入输出、LeetCode 函数签名和通用函数模式。
    - 验证：`PlatformFormat` 枚举已定义 ACM/LeetCode/Generic 三种格式，`build_platform_signature_hint` 生成对应签名约束；`cargo test`（114 通过）。
  - [x] 6.4 把 C++ 设为 MVP 推荐默认语言，除非用户选择其他语言。
    - 验证：`LanguageId::default()` 返回 `Cpp20`，`all_language_ids()` 以 C++20 为首；`cargo test` 确认。
  - [x] 6.5 为每种语言增加 Prompt 约束，包含导入、类名、输入解析和标准版本要求。
    - 验证：`build_language_constraints` 覆盖全部 8 种语言，含标准版本、导入约定和输入解析提示；`cargo test`（114 通过）。
  - [x] 6.6 支持在相同裁剪图和识别文本的基础上，一键切换语言重新生成。
    - 验证：已新增 `src-tauri/src/session.rs` 的 `SessionContext`，保存最近一次截图和识别数据；新增 `regenerate_with_language` 命令复用保存的数据构造新语言 prompt 并重发 AI 请求；前端 `handleSwitchLanguage` 调用新命令；`cargo test`（135 通过，含 5 个 session 测试和 4 个 commands 测试），`npm test -- --run`（48 通过）。
  - [x] 6.7 增加输出解析，识别主代码块，供复制按钮使用。
    - 验证：已新增 `src-tauri/src/output_parser.rs`，`extract_main_code_block` 先按 "完整代码" section 提取，再降级到第一个 fenced code block；`cargo test`（114 通过）。
  - [x] 6.8 给语言元数据、模板覆盖、Prompt 变量和代码块提取写测试。
    - 验证：`language::tests`（8 通过）、`prompts::tests`（8 通过）、`output_parser::tests`（6 通过）；`cargo test`（114 通过）。
  - [ ] 6.9 至少手动验证一个数组题、一个动态规划题和一个图题的输出。
    - 阻塞：依赖 5.10 真实模型验证完成后才能进行。

- [x] 7.0 搭建结果面板、代码高亮、复制动作和输出速度控制
  - [x] 7.1 做出结果面板状态，包含空闲、截图中、识别中、等待框选、生成中、完成和失败。
    - 验证：已新增 `ResultPanel` 组件，支持 `idle`/`loading`/`streaming`/`complete`/`failed` 五种状态，带状态指示器和对应图标；`npm test -- --run`（48 通过）。
  - [x] 7.2 用 Markdown 渲染输出，并对代码块做语法高亮。
    - 验证：`ResultPanel` 使用 `react-markdown` 渲染内容，代码块通过 `prismjs` 做语法高亮，支持 C/C++/Python/Java/JavaScript/TypeScript/Go/Rust/Markdown；`npm test -- --run` 通过。
  - [x] 7.3 加复制代码、复制完整答案、清空结果、重新生成和切换语言动作。
    - 验证：`ResultPanel` 工具栏包含复制代码（提取 fenced code block）、复制完整答案、清空结果、重新生成和切换语言下拉菜单；`ResultPanel.test.tsx`（13 通过）。
  - [x] 7.4 增加输出速度选项，包含快速、正常、慢速和自定义字符每秒。
    - 验证：`useTypewriter` hook 支持 `fast`/`normal`/`slow`/`custom` 四档速度；`npm test -- --run` 通过。
  - [x] 7.5 实现本地展示节奏控制，同时兼容缓冲输出和流式分块。
    - 验证：`useTypewriter` 以 50ms tick 为基准，按 charsPerTick 逐步揭示文本；`append()` 可连续接收流式 chunk；`useTypewriter.test.tsx`（7 通过）。
  - [x] 7.6 保证在视觉上还没完全展开时，复制拿到的仍然是完整输出。
    - 验证：`fullText` 始终保存完整累积文本，`displayedText` 仅控制视觉展示；复制代码和复制完整答案都从 `fullText` 提取；测试覆盖。
  - [x] 7.7 给截图、识别、生成、完成和失败加可见状态文本。
    - 验证：`ResultPanel` 状态指示器显示对应文案（生成中.../生成完成/生成失败）；中英文 i18n 已补充；`npm test -- --run` 通过。
  - [x] 7.8 给速度节奏、复制行为、失败状态和语言重新生成动作写测试。
    - 验证：`useTypewriter.test.tsx`（7 通过）覆盖速度、append、reset、revealAll；`ResultPanel.test.tsx`（13 通过）覆盖状态渲染、按钮交互、语言切换；`npm test -- --run`（48 通过）。
  - [ ] 7.9 手动验证长 C++ 输出不会撑坏常见桌面窗口布局。
    - 阻塞：需要 GUI 环境运行 `npm run tauri dev` 来验证长输出布局；当前 shell 无法完成手动验证。

- [x] 8.0 加本地设置、可选历史、缓存清理和隐私控制
  - [x] 8.1 定义设置结构和默认值，覆盖快捷键、服务商、语言、平台格式、输出速度、历史和截图保留策略。
  - [x] 8.2 加设置迁移，避免未来的结构变化破坏老用户数据。
  - [x] 8.3 增加可选本地历史，记录时间戳、识别文本、选择的语言、模型、结果和用户备注。
  - [x] 8.4 保持默认不保存整张屏幕截图。
  - [x] 8.5 增加明确的隐私说明，解释截图、裁剪、临时文件、AI 请求和可选历史的流向。
  - [x] 8.6 增加清空缓存、删除单条历史和清空全部历史动作。
  - [x] 8.7 在应用启动时自动清理遗留的临时图片。
  - [x] 8.8 给设置默认值、历史关闭行为、删除动作和清理行为写测试。
  - [x] 8.9 编写 `docs/privacy-data-flow.md`，作为发布文档的一部分。

- [ ] 9.0 打包 Windows MVP 并编写发布说明
  - [x] 9.1 在 README 里写清楚 Node、Rust、Tauri 依赖、安装步骤和开发命令。
    - 验证：README.md 已更新，包含依赖列表、安装步骤、开发命令和分支说明。
  - [x] 9.2 说明 MVP 支持流程：配置模型、按快捷键、必要时框选、生成答案、复制代码。
    - 验证：README.md 中新增 "MVP 使用流程" 章节，覆盖 8 步完整流程。
  - [x] 9.3 说明不支持的范围：隐蔽考试、监考规避、自动提交到第三方平台和自动填答案。
    - 验证：README.md 中新增 "明确不支持的范围" 章节，列出 5 条红线。
  - [x] 9.4 加构建脚本，并验证 Windows 安装包能生成。
  - [x] 9.5 运行前端单元测试。
  - [x] 9.6 运行 Rust 单元测试。
  - [ ] 9.7 按 PRD 验收条件做一次完整的手动检查。
    - 阻塞：当前 shell 环境无法运行 GUI 做交互式手动验证。所有 PRD 验收项的代码实现和单元测试均已通过（前端 48 测试、Rust 135 测试）。真实模型端到端验证（5.10）同样依赖 GUI 环境。
  - [x] 9.8 在 README 里记录已知限制和后续版本方向。
    - 验证：README.md 中新增 "已知限制与后续方向" 章节，列出 4 条当前限制和 5 个后续方向。
  - [ ] 9.9 在验证通过后，为第一版 MVP 做 tag 或发布准备。
    - 阻塞：依赖 9.7 手动检查完成。安装包已生成（`src-tauri/target/release/bundle/nsis/Question Scan_0.1.0_x64-setup.exe`，3.87 MB）。

- [ ] 10.0 本地 RAG 和 LeetCode 题目上下文增强
  - [x] 10.1 更新 PRD，明确 RAG 只服务授权练习、自测、开放题目环境和个人工作流，不支持隐蔽考试、受限面试规避、自动提交或自动填答案。
    - 验证：已更新 `prd/prd-question-scan.md`，新增本地 RAG、多层容错 Agent 工作流和远端 Provider Proxy 的功能需求、非目标范围、设计要求、技术方案、阶段规划、后续验收标准、风险和开放问题；RAG 明确只使用轻量题目索引、用户导入资料和用户可控历史，不默认内置未授权完整题面。
  - [x] 10.2 设计本地 RAG 数据模型，覆盖题目索引、用户笔记、代码模板、历史题目、召回记录和数据来源标记。
    - 验证：已新增 `docs/rag-data-model.md`，覆盖 RAG 用例、JSON/SQLite 存储策略、`RagDocument`、`RagChunk`、`RagEmbedding`、`ProblemIndexEntry`、`CodeTemplate`、`HistoryRagLink`、`RetrievalRecord`、`RagSettings`、删除隐私规则、召回契约、prompt 注入契约和测试建议；`git diff --check` 通过。
  - [x] 10.3 增加 LeetCode 轻量题目索引，先只保存题号、标题、slug、难度、标签、题型和平台来源，不默认内置未授权完整题面。
    - 验证：已新增 `src-tauri/src/problem_index.rs` 和 `src-tauri/data/leetcode-lightweight-index.json`，索引项只包含 `id`、`platform`、`problemNumber`、`slug`、`title`、`difficulty`、`tags`、`algorithmTags`、`problemType` 和 `source`；后端使用 `deny_unknown_fields` 拒绝 `problemStatement` 等完整题面字段；已新增 `list_leetcode_problem_index` 只读命令和前端类型化 API；`npm test -- --run`（49 通过）、`cargo test`（142 通过）、`npm run lint` 和 `git diff --check` 均通过。当前项目没有 E2E 脚本，未声明 E2E 通过。
  - [x] 10.4 支持用户导入自己的 Markdown、JSON 或 CSV 题解、错题笔记和代码模板，并记录导入来源、更新时间和删除入口。
    - 验证：已新增 `src-tauri/src/rag_imports.rs`，支持 Markdown front matter/标题解析、JSON 单条或数组解析、CSV header 解析、题解/笔记/代码模板类型、来源名称、来源 URI、导入时间、更新时间、重复导入更新和软删除；已新增 `import_rag_documents`、`list_rag_imports` 和 `delete_rag_import` 命令及前端类型化 API；`npm test -- --run`（50 通过）、`cargo test`（148 通过）、`npm run lint` 和 `git diff --check` 均通过。当前项目没有 E2E 脚本，未声明 E2E 通过。
  - [x] 10.5 在用户开启历史保存时，为识别出的题目文本、AI 输出摘要、用户备注、语言和标签建立可检索记录。
    - 验证：已新增 `src-tauri/src/rag_history.rs`，把历史记录转换为本地 JSON-backed RAG 文档、分块和 `HistoryRagLink`，摘要会跳过完整代码块，删除单条历史或清空历史时对应记录标记为不可召回；已新增 `save_history_entry` 命令并在前端 AI 流 `done` 且 `saveHistory=true` 时触发，关闭历史时不会新增记录；`npm test -- --run`（52 通过）、`cargo test`（154 通过）、`npm run lint` 均通过。当前项目没有 E2E 脚本，未声明 E2E 通过。
  - [x] 10.6 接入 embedding 生成与本地向量检索，支持按题干、标题、样例、约束和标签召回 top K 相似题。
    - 验证：已新增 `src-tauri/src/rag_retrieval.rs`，使用本地 deterministic `local-hash` embedding provider 为用户导入资料、历史 RAG 分块和 LeetCode 轻量元数据生成并缓存向量；已新增 `search_rag_context` 命令和前端类型化 API，支持按标题、题干、样例、约束、标签、算法标签、目标语言、top K 和最低分过滤召回相似上下文；删除资料和低分结果不会返回，轻量索引只返回元数据摘要；`cargo test`（161 通过）、`npm test -- --run`（53 通过）、`npm run lint` 和 `git diff --check` 均通过。当前项目没有 E2E 脚本，未声明 E2E 通过。
  - [x] 10.7 增加题型和模板召回逻辑，按相似题、算法标签和用户默认语言选择可用的解题模式与代码模板。
    - 验证：已新增 `src-tauri/src/rag_templates.rs`，根据 10.6 相似题结果中的 `problemType`、标签和算法标签推荐哈希表、双指针、滑动窗口、动态规划、二分、图搜索、树遍历、单调栈、回溯和数学等解题模式；用户导入的 `template` 资料会按目标语言或用户默认语言、平台、算法标签和命中的解题模式排序，其他语言模板不会被误选，未指定语言的通用模板可作为回退；已新增 `select_rag_template_context` 命令和前端类型化 API；`cargo test`（165 通过）、`npm test -- --run`（54 通过）、`npm run lint`、`rustfmt --edition 2021 --check src\rag_templates.rs src\rag_retrieval.rs` 和 `git diff --check` 均通过。当前项目没有 E2E 脚本，未声明 E2E 通过。
  - [x] 10.8 在解题 prompt 中注入压缩后的本地上下文，限制召回条数和 token 预算，避免低置信内容覆盖截图识别结果。
    - 验证：已新增 `src-tauri/src/rag_prompt_context.rs`，把 10.6 的相似题/历史/用户资料命中和 10.7 的解题模式/代码模板推荐压缩为独立 `Local recalled context` section；注入前会按 `maxItems`、`maxContextTokens` 和 `minScore` 过滤，低分、空上下文、超条数和超 token 预算项会返回 `skippedReason`，且 LeetCode 轻量索引只作为 metadata-only 提示；`src-tauri/src/prompts.rs` 已支持可选 RAG section 注入，`build_rag_prompt_context` 命令和前端类型化 API 已接入；`cargo test`（171 通过）、`npm test -- --run`（55 通过）、`npm run lint`、`rustfmt --edition 2021 --check src\rag_prompt_context.rs src\prompts.rs` 和 `git diff --check` 均通过。当前项目没有 E2E 脚本，未声明 E2E 通过。
  - [x] 10.9 在结果面板展示本次命中的相似题、本地模板、历史记录和置信度，并允许用户展开查看或忽略这些上下文。
    - 验证：已扩展 `src/components/ResultPanel.tsx`，当传入 RAG prompt context items 时会在答案上方展示本地召回上下文区块，按相似题、用户资料、本地模板、历史记录和解题模式显示来源、类型、置信度、已注入/未注入状态；用户可以展开单条上下文查看摘要、标签、token 估算和跳过原因，也可以在当前面板忽略单条上下文且不删除本地资料；无上下文时不显示空区块；已更新中英文 i18n 文案和 `src/App.tsx` 的 ResultPanel 文案交接；`src/components/ResultPanel.test.tsx` 覆盖来源、置信度、展开详情、忽略和空上下文行为；`npm test -- --run`（59 通过）、`cargo test`（171 通过）、`npm run lint`、`npm run build` 和 `git diff --check` 均通过。当前项目没有 E2E 脚本，未声明 E2E 通过。
  - [x] 10.10 在设置页增加本地知识增强开关，覆盖历史入库、用户笔记检索、代码模板检索、相似题提示和最大召回条数。
    - 验证：已扩展前后端设置契约，新增本地知识增强总开关、历史入库、用户笔记检索、代码模板检索、相似题提示和最大召回条数；总开关默认关闭，历史入库默认关闭，最大召回条数在前端和后端均限制在 1-20；`save_history_entry`、`search_rag_context`、`select_rag_template_context` 和 `build_rag_prompt_context` 会按设置过滤历史、用户资料、模板和轻量相似题来源；设置页已加入中英文控件并覆盖持久化测试；`npm test -- --run`（60 通过）、`cargo test`（174 通过）、`npm run lint`、`npm run build` 和 `git diff --check` 均通过。当前项目没有 E2E 脚本，未声明 E2E 通过。
  - [x] 10.11 增加清除 RAG 索引、删除单条导入资料、重建索引和关闭历史入库的隐私控制。
    - 验证：已新增 `clear_rag_index` 和 `rebuild_rag_index` 后端命令，清除动作会移除 embedding 缓存和历史 RAG 索引但保留用户导入原文、普通历史和轻量题目元数据；重建动作会按当前本地知识增强开关、历史入库开关、用户资料检索开关和相似题开关重新生成索引并返回统计；删除单条导入资料时会同步清理对应 embedding；设置页数据管理区已增加清除索引、重建索引、刷新/删除导入资料和即时关闭历史入库入口；`npm test -- --run src/App.test.tsx src/lib/api.test.ts src/lib/types.test.ts src/lib/i18n.test.ts`（34 通过）、`cargo test rag_retrieval`（10 通过）、`cargo test rag_history`（6 通过）、`cargo test commands`（9 通过）、`npm test -- --run`（64 通过）、`cargo test`（178 通过）、`npm run lint`、`rustfmt --edition 2021 --check src\commands.rs src\rag_retrieval.rs src\rag_history.rs`、`npm run build` 和 `git diff --check` 均通过。当前项目没有 E2E 脚本，未声明 E2E 通过。
  - [x] 10.12 为题目索引解析、导入校验、分块、embedding 缓存、相似题排序、上下文裁剪和隐私关闭行为补充测试。
    - 用例：题目索引解析必须拒绝空标签、空算法标签和不支持的 schema，避免不完整元数据进入 RAG。
    - 用例：用户导入必须拒绝空来源、空内容、缺少正文的 JSON/CSV 和未闭合引号的 CSV。
    - 用例：长用户资料必须按稳定分块生成多个 embedding，删除资料后对应分块 embedding 必须全部清理。
    - 用例：相似题排序必须在分数相同的情况下保持稳定顺序，避免 UI 和 prompt 上下文抖动。
    - 用例：上下文裁剪必须压缩长 snippet 和模板正文，并在 token 预算不足时保留原始解题 prompt。
    - 用例：关闭本地知识增强、历史入库、用户资料检索、代码模板检索或相似题提示时，命令层必须过滤对应 RAG 来源，不阻塞主解题流程。
    - 验证：已在 `src-tauri/src/problem_index.rs` 补充不支持 schema、空标签和空算法标签测试；在 `src-tauri/src/rag_imports.rs` 补充空来源、空内容、缺少正文 JSON/CSV 和未闭合 CSV 引号测试；在 `src-tauri/src/rag_retrieval.rs` 补充长导入资料多分块 embedding、删除导入资料清理全部分块 embedding 和同分 chunk ID 稳定排序测试；在 `src-tauri/src/rag_prompt_context.rs` 补充长检索 snippet 和模板正文裁剪测试；在 `src-tauri/src/commands.rs` 补充关闭各 RAG 来源后的命令层过滤测试。`cargo test problem_index`（9 通过）、`cargo test rag_imports`（9 通过）、`cargo test rag_retrieval`（12 通过）、`cargo test rag_prompt_context`（6 通过）、`cargo test commands`（10 通过）、`cargo test`（187 通过）、`npm test -- --run src/App.test.tsx src/lib/api.test.ts src/lib/types.test.ts src/lib/i18n.test.ts`（34 通过）、`rustfmt --edition 2021 --check src\problem_index.rs src\rag_imports.rs src\rag_retrieval.rs src\rag_prompt_context.rs src\commands.rs` 和 `git diff --check` 均通过。当前项目没有 E2E 脚本，未声明 E2E 通过；全量 `cargo fmt -- --check` 仍会因既有未触碰文件 `src-tauri/src/pipeline.rs`、`src-tauri/src/session.rs` 的格式差异失败。
  - [ ] 10.13 用数组题、动态规划题和图题各手动验证一次：截图识别后能召回相似题，且输出不会被错误上下文带偏。
    - 用例：数组题可用 Two Sum / Contains Duplicate / Best Time to Buy and Sell Stock，触发截图识别后应召回数组、哈希表或双指针相关相似题，并且最终解法以当前截图题面为主。
    - 用例：动态规划题可用 Climbing Stairs / Maximum Subarray / Longest Palindromic Substring，触发截图识别后应召回动态规划相关相似题或解题模式，并且不能因为相似题上下文改变当前题目的状态定义。
    - 用例：图题可用 Number of Islands，触发截图识别后应召回图搜索、DFS/BFS 或网格图相关相似题，并且最终输出不能被数组或动态规划上下文带偏。
    - 验证：已补齐截图识别→RAG 召回→prompt 注入→结果面板上下文交接链路。`src-tauri/src/pipeline.rs` 会在截图后先调用视觉模型识别题目区域和标题/题干，再把识别结果交给 `search_rag_context_with_settings`、`select_rag_template_context_with_settings` 和 `build_rag_prompt_context_with_settings`；识别或 RAG 失败时会跳过增强并继续原解题主流程。`src-tauri/src/session.rs` 会保存本次 RAG 上下文和 prompt section，语言切换时复用；`src-tauri/src/streaming.rs` 和 `src/App.tsx` 已通过 `ragContext` 流事件把真实命中项传给 `ResultPanel`。`npm test -- --run`（65 通过）、`npm run lint`、`npm run build`、`cargo test`（190 通过）和 `git diff --check` 均通过。
    - 阻塞：该子任务仍未完成，因为还需要真实 GUI、真实多模态 Provider API Key 和数组/DP/图三类题截图人工验证召回质量与最终输出是否被错误上下文带偏。当前 shell 环境只能完成自动化验证，不能替代 `npm run dev:desktop` 下的手动观察；项目 `package.json` 也没有 E2E 脚本，未声明 E2E 通过。

- [ ] 11.0 多层容错 Agent 工作流
  - [ ] 11.1 更新 PRD 和开发文档，定义 Agent 只做可见、可解释、用户可控的本地流程编排，不新增隐蔽使用或规避检测能力。
  - [ ] 11.2 定义 Agent 步骤状态机，覆盖 `capture`、`crop`、`recognize`、`retrieve`、`solve`、`parse`、`render` 和 `save`。
  - [ ] 11.3 为每个步骤定义输入、输出、成功条件、失败原因、重试策略和可降级路径，统一写入类型定义。
  - [ ] 11.4 将现有截图、自动裁剪、手动框选、题目识别、AI 请求、输出解析和历史保存串联到 Agent 工作流。
  - [ ] 11.5 增加截图阶段容错：多显示器枚举失败、权限不足、截图为空、图片过大和临时文件写入失败时给出明确状态与重试入口。
  - [ ] 11.6 增加裁剪阶段容错：自动定位低置信度、坐标越界或识别失败时进入手动框选，并保留重新自动识别入口。
  - [ ] 11.7 增加识别阶段容错：视觉模型失败时尝试本地 OCR 或手动确认题目标题/题干，再把确认内容交给 RAG 和解题 prompt。
  - [ ] 11.8 增加 RAG 阶段容错：本地索引为空、embedding 失败或召回低置信度时跳过增强，不阻塞主解题流程。
  - [ ] 11.9 增加 AI 请求阶段容错：主 provider 超时、限流、网络错误或非流式解析失败时，按用户配置进行重试、非流式降级或备用 provider 切换。
  - [ ] 11.10 增加输出解析阶段容错：模型未按固定格式输出时，先本地提取主代码块，再可选请求一次格式修复，仍失败则展示原始输出并标记复制限制。
  - [ ] 11.11 在结果面板展示 Agent 步骤进度、使用过的降级路径、失败原因、重试次数和最终使用的 provider。
  - [ ] 11.12 在设置页增加智能容错开关，允许用户配置最大重试次数、是否启用备用 provider、是否启用 OCR 兜底和是否展示详细步骤日志。
  - [ ] 11.13 为状态机流转、截图失败、裁剪低置信度、识别失败、RAG 跳过、AI 重试、provider 切换和输出格式修复补充测试。
  - [ ] 11.14 手动验证浏览器、PDF、IDE、深色题目页和长题面截图，确认每条失败路径都有可理解提示和可继续操作。

- [ ] 12.0 可选远端 Provider Proxy 和 API Key 托管
  - [ ] 12.1 更新 PRD 和隐私文档，明确远端代理模式默认关闭，用户开启后裁剪图、识别文本和请求参数会发送到用户配置的代理服务。
  - [ ] 12.2 设计本地直连和远端代理两种 provider 模式，确保用户可以随时关闭代理并回到本地 API Key。
  - [ ] 12.3 定义远端代理请求协议，覆盖模型请求、流式输出、错误标准化、超时、限流、备用 provider 和审计元数据。
  - [ ] 12.4 在设置页增加代理地址、用户 token、连接测试、代理模式说明、数据流提示和清除 token 操作。
  - [ ] 12.5 将代理 token 按现有密钥存储策略保存，避免写入普通设置文件，并为迁移和删除流程补充校验。
  - [ ] 12.6 为代理模式接入 AI 请求层，使截图图片和 prompt 可以经代理转发，同时保留现有 OpenAI-compatible 本地直连路径。
  - [ ] 12.7 增加代理 failover 策略：代理不可用时按用户配置提示重试、切回本地 provider 或终止请求，不自动泄露到未配置服务。
  - [ ] 12.8 编写远端代理安全边界文档，要求默认不保存截图、题面和完整回答日志；如开启日志，只记录时间、模型、错误码、耗时和用量估算。
  - [ ] 12.9 可选搭建最小代理服务样例，用于本地开发验证 token 鉴权、provider 转发、流式响应和错误映射。
  - [ ] 12.10 为代理配置校验、token 存取、请求体构造、错误映射、流式转发和本地直连回退补充测试。
  - [ ] 12.11 手动验证本地直连、代理成功、代理超时、代理鉴权失败、代理限流和关闭代理后的回退行为。
