# Question Scan

Question Scan 是一个基于 Tauri 的桌面算法解题助手。用户按下全局快捷键后，软件截取当前屏幕，自动识别屏幕中的算法题区域，生成临时图片，并把图片和用户配置发送给多模态 AI。AI 返回题目分析、解题思路、主流语言代码、复杂度说明和边界条件。

**默认使用场景**：授权练习、自测、开放题目环境和个人效率工作流。

## 快速开始

### 依赖

- Node.js 22 LTS 或更新
- Rust stable MSVC toolchain
- Microsoft C++ Build Tools
- Microsoft Edge WebView2 Runtime
- PowerShell 5.1 或 PowerShell 7

### 安装和启动

```powershell
git clone git@github.com:Summus1999/Question_scan.git
cd Question_scan
powershell -ExecutionPolicy Bypass -File .\scripts\setup.ps1
powershell -ExecutionPolicy Bypass -File .\scripts\dev.ps1
```

首次设置后，日常启动只需：

```powershell
.\scripts\dev.ps1
```

更多环境细节见 [docs/developer-setup.md](docs/developer-setup.md)。

## MVP 使用流程

1. **配置 AI 服务**：打开设置面板，填写 Provider 名称、Base URL、API Key 和模型名称。支持所有 OpenAI-compatible API，包括 DeepSeek、OpenAI、Gemini 等。
2. **配置偏好**：选择默认语言（C++20 / C++17 / Python / Java / JavaScript / TypeScript / Go / Rust）、平台格式（ACM / LeetCode / Generic）和输出速度。
3. **按下快捷键**：默认 `Ctrl+Shift+Q`，软件截取当前屏幕。
4. **自动识别**：软件识别屏幕中的算法题区域，生成裁剪后的临时图片。
5. **手动框选兜底**：如果自动识别失败或置信度不足，用户可以在屏幕上拖拽框选题目区域。
6. **生成答案**：软件把裁剪图片发送给 AI，AI 返回结构化答案（题目识别、解题策略、完整代码、复杂度、边界用例、注意事项）。
7. **复制代码**：用户可以在结果面板复制完整代码或完整答案。
8. **切换语言重新生成**：用户可以在结果面板一键切换语言，用同一张截图重新生成其他语言的解法。

## 开发命令

```powershell
# 启动开发环境
npm run dev:desktop

# 代码检查
npm run lint

# 前端单元测试
npm test -- --run

# Rust 单元测试
cd src-tauri && cargo test

# 构建生产包
npm run build
```

## 隐私与数据说明

- **只在用户主动触发时截图**：软件不会自动或隐蔽截图。
- **默认不保存完整截图**：临时图片在请求完成后自动删除，除非用户显式启用保存历史。
- **数据流向透明**：截图 → 本地裁剪 → 发送给用户配置的 AI 服务 → 结果展示在本地面板。
- **可选本地历史**：用户可以选择保存历史记录到本地 JSON 文件，包含时间戳、识别文本、语言、模型和结果。
- **缓存清理**：用户可以随时清空临时图片缓存和历史记录。

完整隐私数据流说明见 [docs/privacy-data-flow.md](docs/privacy-data-flow.md)。

## 明确不支持的范围

本产品**不**支持、不鼓励、不添加以下能力：

- 隐蔽考试使用或监考规避
- 屏幕共享规避或反监控行为
- 自动提交答案到第三方平台
- 自动把答案填入第三方网页
- 读取浏览器 Cookie、账号或凭据

## 已知限制与后续方向

**当前限制**：

- MVP 仅支持 Windows 桌面环境。
- 截图 → AI 请求端到端链路需要手动验证（依赖真实模型 API）。
- 自动题目识别准确率依赖 AI 视觉模型，复杂布局可能需手动框选兜底。
- 代码正确性由 AI 模型决定，软件不保证通过所有测试用例。

**后续版本方向**：

- 支持 macOS 和 Linux。
- 接入更多 AI Provider（Claude、Gemini、本地 Ollama 等）。
- 优化自动题目识别准确率。
- 增加本地样例运行和验证。
- 支持导出代码为文件。

## 开发分支

- `main`：稳定分支，供其他开发者拉取。
- `feature/question-scan-mvp`：MVP 开发分支，用于增量功能开发。

当一个父任务完成并验证通过后，合并或 fast-forward 到 `main`，然后推送分支。
