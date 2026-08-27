# 15 — Build and Development Environment

## Toolchain

Antigravity must verify the latest compatible versions before scaffolding and then pin/document the selected versions.

Expected tools:

- Rust stable + rustfmt + clippy,
- Node.js LTS,
- pnpm or another single standardized package manager,
- Tauri 2 CLI,
- Android Studio/JBR,
- Android SDK Platform/Build Tools/Platform Tools,
- Android NDK required by Tauri/Rust mobile build,
- Windows SDK/Visual Studio Build Tools for Windows builds,
- Wintun distribution for Windows integration.

## Android target policy

The primary runtime requirement is Android 15/API 35. `minSdk`, `compileSdk`, and `targetSdk` must be selected from current Tauri/Android/Play requirements rather than copied blindly from this document.

If targeting newer SDKs, account for newer local-network permission behavior. Keep Android 15 functional.

## Environment checks

Create a developer preflight command/script later that verifies:

- rustc/cargo,
- Node/package manager,
- Java,
- Android SDK/NDK paths,
- adb,
- Tauri CLI,
- Windows SDK when on Windows,
- required native libraries.

## Antigravity environment limitation handling

If Antigravity is running in a remote Linux sandbox, it may be able to compile/test much of the shared and Android code but cannot truthfully execute physical Android/Windows integration tests. Mark those as `HUMAN_GATE` or hardware-lab work while continuing all software-only tasks.

If Antigravity is running locally on the development Windows machine with Android attached through ADB, automate device builds/log capture/tests directly.
