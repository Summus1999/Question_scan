## Relevant Files

- `package.json` - Frontend scripts, dependencies, and Tauri command shortcuts.
- `vite.config.ts` - Vite configuration for the React frontend.
- `tsconfig.json` - TypeScript compiler settings.
- `src/main.tsx` - React app entry.
- `src/App.tsx` - Main shell, routing, and layout composition.
- `src/styles.css` - Global styles and Tailwind entry.
- `src/lib/types.ts` - Shared frontend types for settings, capture state, AI output, languages, and history.
- `src/lib/api.ts` - Typed wrapper around Tauri commands and events.
- `src/lib/outputSpeed.ts` - Frontend display-speed controller for streamed and buffered output.
- `src/lib/outputSpeed.test.ts` - Unit tests for display-speed behavior.
- `src/components/ResultPanel.tsx` - AI output panel, status display, code highlighting, copy actions, and language switching.
- `src/components/ResultPanel.test.tsx` - Component tests for output states and copy actions.
- `src/components/SettingsPanel.tsx` - Shortcut, provider, model, language, output speed, and privacy settings.
- `src/components/SettingsPanel.test.tsx` - Component tests for settings validation and save behavior.
- `src/components/CropOverlay.tsx` - Manual crop fallback UI when automatic problem detection is uncertain.
- `src/components/CropOverlay.test.tsx` - Component tests for selection, cancel, and confirm behavior.
- `src/components/TrayStatus.tsx` - Small status surface for idle, capturing, detecting, generating, complete, and failed states.
- `src-tauri/Cargo.toml` - Rust dependencies for Tauri, capture, image processing, HTTP, settings, and storage.
- `src-tauri/tauri.conf.json` - Tauri app configuration, permissions, windows, tray, and bundle settings.
- `src-tauri/capabilities/default.json` - Tauri command and plugin permissions.
- `src-tauri/src/main.rs` - Tauri backend entry, command registration, tray setup, app state, and event wiring.
- `src-tauri/src/commands.rs` - Tauri command handlers exposed to the frontend.
- `src-tauri/src/settings.rs` - Local settings load, save, validation, and migration.
- `src-tauri/src/settings_test.rs` - Rust tests for settings defaults, validation, and persistence.
- `src-tauri/src/screen_capture.rs` - Screen capture, multi-monitor handling, temporary image creation, crop, and cleanup.
- `src-tauri/src/screen_capture_test.rs` - Rust tests for image crop math, temp-file lifecycle, and cleanup logic.
- `src-tauri/src/problem_detection.rs` - Problem-region detection, coordinate parsing, confidence scoring, and manual fallback decision.
- `src-tauri/src/problem_detection_test.rs` - Rust tests for detection response parsing and crop-coordinate validation.
- `src-tauri/src/ai_client.rs` - OpenAI-compatible multimodal API client, streaming, retries, and typed errors.
- `src-tauri/src/ai_client_test.rs` - Rust tests for request building, provider config validation, and stream parsing.
- `src-tauri/src/prompt_templates.rs` - Prompt templates for problem recognition, direct solution generation, language choices, and platform formats.
- `src-tauri/src/prompt_templates_test.rs` - Rust tests for prompt variables and language/template coverage.
- `src-tauri/src/languages.rs` - Supported language definitions, file extensions, and output preferences.
- `src-tauri/src/history.rs` - Optional local history, cache cleanup, and privacy data controls.
- `src-tauri/src/history_test.rs` - Rust tests for history save, delete, clear, and privacy defaults.
- `src-tauri/src/errors.rs` - Backend error types mapped to frontend-safe messages.
- `docs/technical-spike.md` - Notes from validating shortcut, capture, temp image, and AI request feasibility.
- `docs/privacy-data-flow.md` - User-facing privacy and data-flow explanation.
- `README.md` - Project setup, development commands, supported platforms, and MVP usage.

### Notes

- Unit tests should sit near the code they verify when the project structure allows it. Rust tests can live in the same module or sibling test modules.
- Use `npm test -- --run` for frontend unit tests after the frontend test runner is configured.
- Use `cargo test` from `src-tauri` for Rust unit tests.
- Use `npm run tauri dev` for manual desktop verification.
- First release should target Windows. macOS and Linux support can be planned after the screenshot and permission model is stable.
- OpenAI-compatible multimodal API is the first Provider target. Other Providers should be added behind the same interface after MVP.
- Output speed controls the local reveal speed in the result panel. It cannot fully control model-side generation latency.
- The product scope is authorized practice, self-testing, open problem environments, and personal workflow use. Do not add hidden exam, proctoring evasion, automatic third-party submission, or automatic answer insertion features.

## Instructions for Completing Tasks

**IMPORTANT:** As you complete each task, you must check it off in this markdown file by changing `- [ ]` to `- [x]`. This helps track progress and ensures you don't skip any steps.

Example:
- `- [ ] 1.1 Read file` -> `- [x] 1.1 Read file` after completing.

Update the file after completing each sub-task, not just after completing an entire parent task.

## Tasks

- [ ] 0.0 Create feature branch
  - [ ] 0.1 Confirm the current git status and note any unrelated existing changes.
  - [ ] 0.2 Create and checkout a new branch, for example `feature/question-scan-mvp`.
  - [ ] 0.3 Confirm the branch name and clean baseline before project scaffolding.

- [ ] 1.0 Build Tauri app foundation
  - [ ] 1.1 Initialize a Tauri 2.x project with React, TypeScript, and Vite.
  - [ ] 1.2 Add Tailwind CSS and the base styling entry.
  - [ ] 1.3 Add a basic app shell with a compact control surface, settings area, and result panel placeholder.
  - [ ] 1.4 Configure linting, formatting, and test scripts.
  - [ ] 1.5 Configure Tauri permissions and app metadata in `tauri.conf.json` and capabilities files.
  - [ ] 1.6 Add typed frontend-to-backend command wrappers in `src/lib/api.ts`.
  - [ ] 1.7 Add shared type definitions for app state, settings, languages, capture status, and AI result status.
  - [ ] 1.8 Verify the app starts with `npm run tauri dev`.

- [ ] 2.0 Add global shortcut and tray workflow
  - [ ] 2.1 Add the Tauri global shortcut plugin or the chosen shortcut integration.
  - [ ] 2.2 Define default shortcut `Ctrl+Shift+Q` and make it configurable.
  - [ ] 2.3 Register the shortcut on app startup and unregister it on app shutdown.
  - [ ] 2.4 Detect registration failure and show a user-facing conflict message.
  - [ ] 2.5 Add a tray icon with states for idle, capturing, detecting, generating, complete, and failed.
  - [ ] 2.6 Add tray actions for show window, open settings, enable or disable shortcut, and quit.
  - [ ] 2.7 Write settings tests for shortcut defaults, custom shortcut persistence, and disabled shortcut state.
  - [ ] 2.8 Manually verify shortcut trigger while the app is in the background.

- [ ] 3.0 Build screen capture and temporary image pipeline
  - [ ] 3.1 Select and add a Rust screenshot library that works on Windows.
  - [ ] 3.2 Implement capture of the active display or all displays, depending on configuration.
  - [ ] 3.3 Normalize display coordinates for multi-monitor layouts.
  - [ ] 3.4 Save a temporary PNG or JPEG in the system temp directory.
  - [ ] 3.5 Add image compression settings to control AI request size.
  - [ ] 3.6 Return a capture metadata object with image path, dimensions, display id, and timestamp.
  - [ ] 3.7 Ensure temporary images are deleted after the AI request or after failure.
  - [ ] 3.8 Add tests for crop math, temp-file cleanup, and invalid path handling.
  - [ ] 3.9 Manually verify screenshot capture on a normal Windows desktop and a multi-monitor setup if available.

- [ ] 4.0 Add problem-region detection and manual crop fallback
  - [ ] 4.1 Define the detection response schema: bounding box, confidence, extracted title, extracted problem text, and reason.
  - [ ] 4.2 Build a low-resolution image request for region detection to reduce cost and latency.
  - [ ] 4.3 Ask the vision model to return the most likely algorithm problem area coordinates.
  - [ ] 4.4 Validate AI-returned coordinates against the screenshot bounds.
  - [ ] 4.5 Crop the original high-resolution screenshot using the validated region.
  - [ ] 4.6 Add confidence thresholds for automatic accept, confirm-before-use, and manual fallback.
  - [ ] 4.7 Build `CropOverlay` so the user can drag-select the problem area when detection fails.
  - [ ] 4.8 Add cancel, retry auto-detect, and confirm crop actions.
  - [ ] 4.9 Add tests for coordinate parsing, out-of-bounds rejection, confidence routing, and overlay selection behavior.
  - [ ] 4.10 Manually verify detection with browser, PDF, IDE, and dark-mode problem pages.

- [ ] 5.0 Add AI provider settings and multimodal request flow
  - [ ] 5.1 Add settings fields for Provider name, API Base URL, API Key, model, timeout, and streaming enabled.
  - [ ] 5.2 Store secrets locally with the safest practical mechanism available for MVP.
  - [ ] 5.3 Validate Provider settings before sending any request.
  - [ ] 5.4 Implement OpenAI-compatible multimodal request construction for image input and text instructions.
  - [ ] 5.5 Implement streaming response handling and event emission to the frontend.
  - [ ] 5.6 Implement non-streaming fallback for Providers that do not stream image responses reliably.
  - [ ] 5.7 Add typed errors for invalid API Key, unsupported image model, timeout, network failure, and malformed response.
  - [ ] 5.8 Add retry behavior for transient failures with a clear retry limit.
  - [ ] 5.9 Add tests for request payload shape, settings validation, error mapping, and stream parsing.
  - [ ] 5.10 Manually verify one real image request against a configured multimodal model.

- [ ] 6.0 Generate direct algorithm solutions in C++ and other main languages
  - [ ] 6.1 Define supported language metadata for C++17, C++20, Python, Java, JavaScript, TypeScript, Go, and Rust.
  - [ ] 6.2 Add default output template for direct solution mode: problem recognition, strategy, code, complexity, edge cases, and notes.
  - [ ] 6.3 Add platform format options for ACM stdin/stdout, LeetCode function signature, and generic function.
  - [ ] 6.4 Make C++ the recommended default language for MVP unless the user chooses another language.
  - [ ] 6.5 Add per-language prompt constraints, including imports, class naming, input parsing, and standard version.
  - [ ] 6.6 Support one-click regenerate in a different language using the same cropped image and detected text.
  - [ ] 6.7 Add output parsing that identifies the primary code block for copy-code actions.
  - [ ] 6.8 Add tests for language metadata, template coverage, prompt variables, and code-block extraction.
  - [ ] 6.9 Manually verify generated answers for at least one array, one dynamic programming, and one graph problem.

- [ ] 7.0 Build result panel, code highlighting, copy actions, and output-speed control
  - [ ] 7.1 Build result panel states for idle, capturing, detecting, waiting for crop, generating, complete, and failed.
  - [ ] 7.2 Render Markdown output with syntax-highlighted code blocks.
  - [ ] 7.3 Add copy code, copy full answer, clear result, regenerate, and change-language actions.
  - [ ] 7.4 Add output speed options: fast, normal, slow, and custom characters per second.
  - [ ] 7.5 Implement local reveal pacing for buffered output and streamed chunks.
  - [ ] 7.6 Ensure copying uses the complete generated output, even if the visual reveal is still catching up.
  - [ ] 7.7 Add visible status text for screenshot, detection, generation, completion, and failure.
  - [ ] 7.8 Add tests for speed pacing, copy behavior, failed state, and language regeneration action.
  - [ ] 7.9 Manually verify long C++ output does not break layout on common desktop window sizes.

- [ ] 8.0 Add local settings, optional history, cache cleanup, and privacy controls
  - [ ] 8.1 Define settings schema and defaults for shortcut, Provider, language, platform format, output speed, history, and screenshot retention.
  - [ ] 8.2 Add settings migrations so future schema changes do not break existing users.
  - [ ] 8.3 Add optional local history with timestamp, detected text, chosen language, model, result, and user note.
  - [ ] 8.4 Keep full-screen screenshot saving off by default.
  - [ ] 8.5 Add explicit privacy copy explaining screenshot, crop, temporary file, AI request, and optional history storage.
  - [ ] 8.6 Add clear cache, delete history item, and clear all history actions.
  - [ ] 8.7 Add automatic cleanup for orphaned temporary images on app startup.
  - [ ] 8.8 Add tests for settings defaults, history disabled behavior, delete actions, and cleanup behavior.
  - [ ] 8.9 Write `docs/privacy-data-flow.md` for release documentation.

- [ ] 9.0 Package Windows MVP and write release documentation
  - [ ] 9.1 Add README setup instructions for Node, Rust, Tauri prerequisites, and development commands.
  - [ ] 9.2 Document supported MVP workflow: configure model, press shortcut, crop if needed, generate answer, copy code.
  - [ ] 9.3 Document unsupported scope: hidden exam use, proctoring evasion, automatic third-party submission, and automatic answer insertion.
  - [ ] 9.4 Add build script and verify Windows bundle generation.
  - [ ] 9.5 Run frontend unit tests.
  - [ ] 9.6 Run Rust unit tests.
  - [ ] 9.7 Run a manual MVP acceptance pass against the PRD acceptance criteria.
  - [ ] 9.8 Record known limitations and next-version candidates in the README.
  - [ ] 9.9 Tag or prepare the first MVP release after verification passes.
