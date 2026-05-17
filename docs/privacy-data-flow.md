# Privacy and Data Flow

This document explains how Question Scan handles user data, screenshots, and AI requests.

## Overview

Question Scan is a local desktop application. By default, it does not save full
screenshots, does not upload data to any server other than the user-configured
AI provider, and keeps all history stored locally on the user's machine.

## Data Flow

```text
User presses shortcut
        |
        v
+---------------+
| Screen capture |  <-- Full screenshot of current display(s)
+---------------+
        |
        v
+---------------+
| Local crop     |  <-- Question region is cropped locally
+---------------+
        |
        v
+---------------+
| Temp image     |  <-- Temporary PNG/JPEG in %TEMP%/question-scan
+---------------+
        |
        v
+---------------+
| AI request     |  <-- Image + config sent to user-configured AI service
+---------------+
        |
        v
+---------------+
| Result display |  <-- Answer shown in the app
+---------------+
        |
        v
+---------------+
| Temp cleanup   |  <-- Temporary image is deleted (unless history is enabled)
+---------------+
```

## What Data Is Sent Where

### Sent to the AI Provider

- The cropped question image (temporary, user-triggered only)
- The user's API key (for authentication)
- The user's configuration (model, language preference, output format)

### Stored Locally

- Application settings (`%APPDATA%/com.questionscan.desktop/settings.json`)
- History entries (only if the user enables "Save history"):
  - Timestamp
  - Recognized question title and text
  - Selected programming language
  - Platform format (ACM, LeetCode, Generic)
  - AI model name
  - AI-generated result
  - Optional user note
- Temporary screenshot images (deleted after the AI request completes)

### Never Stored or Sent

- Full screen screenshots are never persisted by default
- No data is sent to any server other than the user-configured AI provider
- No analytics, telemetry, or crash reporting is collected
- No browser cookies, credentials, or personal accounts are accessed

## User Controls

### History

- **Default**: Disabled. No history is saved unless the user explicitly enables it in settings.
- **Enable**: Toggle "Save history" in the settings panel.
- **Delete single entry**: Each history entry has a delete button.
- **Clear all**: Use the "Clear all history" button in the data management section.

### Cache

- **Automatic cleanup**: On every app startup, orphaned temporary images from previous sessions are automatically deleted.
- **Manual cleanup**: Use the "Clear cache" button in the data management section to delete all temporary images immediately.

### Settings Reset

- Use "Reset defaults" to restore all settings to their initial values, including disabling history.

## Security Notes

1. **API Key**: The API key is stored in the Windows Credential Locker (secure storage) when possible. If migration fails, it remains in the settings file until the next successful save.
2. **Screenshots**: Screenshots are only taken when the user actively presses the configured global shortcut. There is no automatic or background capture.
3. **AI Provider**: Users must configure their own AI provider. The app does not ship with a built-in provider or pre-shared keys.
4. **Trusted Environment**: Only use Question Scan in authorized environments where screen capture and AI assistance are permitted.

## File Locations

| File | Location | Purpose |
| --- | --- | --- |
| Settings | `%APPDATA%/com.questionscan.desktop/settings.json` | User preferences |
| History | `%APPDATA%/com.questionscan.desktop/history.json` | Saved history entries (if enabled) |
| Temp images | `%TEMP%/question-scan/` | Temporary screenshots (auto-deleted) |

## Compliance with Product Scope

Question Scan is designed for:

- Authorized practice and self-test environments
- Open problem platforms
- Personal algorithm workflows

It explicitly does not support:

- Covert use in proctored exams
- Detection evasion
- Automatic submission to third-party platforms
- Automatic answer injection into web forms
