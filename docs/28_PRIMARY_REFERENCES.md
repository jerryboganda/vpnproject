# 28 — Primary Technical References

These references were current when this pack was prepared. Antigravity must verify them again at implementation time because Android/Tauri/Antigravity APIs and policies can change.

## Google Antigravity / Gemini managed agents

- Antigravity Agent: https://ai.google.dev/gemini-api/docs/antigravity-agent
- Building managed agents / AGENTS.md / SKILL.md: https://ai.google.dev/gemini-api/docs/custom-agents
- Agents overview: https://ai.google.dev/gemini-api/docs/agents

## Android

- Local-Only Hotspot: https://developer.android.com/develop/connectivity/wifi/localonlyhotspot
- WifiManager API: https://developer.android.com/reference/android/net/wifi/WifiManager
- Android VPN guide: https://developer.android.com/develop/connectivity/vpn
- VpnService: https://developer.android.com/reference/android/net/VpnService
- Network: https://developer.android.com/reference/android/net/Network
- ConnectivityManager: https://developer.android.com/reference/android/net/ConnectivityManager
- Foreground service types: https://developer.android.com/develop/background-work/services/fgs/service-types
- Android 15 behavior changes: https://developer.android.com/about/versions/15/behavior-changes-15
- Android NDK networking / `android_setsocknetwork`: https://developer.android.com/ndk/reference/group/networking

## Tauri

- Tauri prerequisites/mobile setup: https://v2.tauri.app/start/prerequisites/
- Tauri plugin development: https://v2.tauri.app/develop/plugins/

## Windows

- Wintun: https://www.wintun.net/
- Wintun source mirror/readme: https://github.com/WireGuard/wintun
- Windows Filtering Platform: https://learn.microsoft.com/windows/win32/fwp/windows-filtering-platform-start-page

## TUN-to-proxy candidate

- tun2proxy: https://github.com/tun2proxy/tun2proxy
- docs.rs: https://docs.rs/crate/tun2proxy/latest

## Rule

For implementation decisions, prefer official platform documentation and upstream project documentation over blogs or copied snippets. When docs and observed device behavior conflict, record both and build to the tested public-API behavior without hidden/privileged hacks.
