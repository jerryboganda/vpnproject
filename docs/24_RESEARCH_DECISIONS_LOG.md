# 24 — Initial Research and Decision Notes

These are starting points. Antigravity must re-check current primary documentation when implementing.

## Antigravity customization

Google's current Antigravity/managed-agent documentation supports project instructions via `.agents/AGENTS.md` and skills under `.agents/skills/<name>/SKILL.md`. This repository uses that mechanism deliberately.

## Android Local-Only Hotspot

Current Android documentation states that Local-Only Hotspot provides local communication without Internet access. For apps targeting Android 13+, `NEARBY_WIFI_DEVICES` is required for this API.

Decision: use Local-Only Hotspot rather than privileged system tethering.

## Android VPN routing

Android documentation states that an app's traffic continues through a VPN unless it deliberately binds/bypasses to another network, while `VpnService.protect()` specifically sends a socket outside the VPN.

Android `Network.bindSocket`/NDK network binding can constrain a socket to a particular network, and network-bound sockets fail when that network goes away.

Decision: explicitly bind each gateway upstream socket to the current validated `TRANSPORT_VPN` network for stronger fail-closed behavior.

## Android foreground service

Android 15 places a cumulative background timeout on `dataSync` foreground services. `connectedDevice` is documented for interaction with external devices over network connections and has its own permission/prerequisite rules.

Decision: evaluate/use the truthful connected-device service type rather than misclassifying the persistent gateway as dataSync.

## Windows

Wintun is a small Layer-3 TUN driver intended for userspace tunneling applications and has signed distributable binaries under accompanying terms.

Current `tun2proxy` is a Rust TUN-to-HTTP/SOCKS project supporting Windows/Wintun, IPv4/IPv6, and SOCKS5 UDP.

Decision: evaluate it first before implementing a new userspace network stack.
