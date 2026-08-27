# 22 — Privacy, Permissions, and Distribution

## Privacy posture

VPNBridge should work locally without an account or cloud relay. Avoid telemetry by default unless the product owner later explicitly requests it.

## Data minimization

Persist only what is needed:

- paired device identity/public key,
- user settings,
- sanitized diagnostics metadata,
- recovery journal on Windows.

Do not persist browsing history.

## Android permissions

Every permission must map to a documented feature/API requirement. Remove unused permissions before release.

## Android/Play policy

Before Play distribution, Antigravity must verify the current Google Play policies for foreground services, device/network behavior, VPN-related claims, target SDK, privacy declarations, and data safety. Do not rely on stale policy text in this repository.

## Windows distribution

- signed executable/installer for public release,
- minimal elevation,
- transparent Wintun/native component licensing,
- clean uninstall and network-state restoration.

## Product claims

Do not market the product as guaranteeing anonymity or universal censorship bypass. It forwards traffic through the user's selected VPN; actual reachability/privacy depends on that VPN, destination, network, and platform.
