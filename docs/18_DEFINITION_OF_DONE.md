# 18 — Definition of Done

The project is not done until every applicable item below has objective evidence.

## Architecture

- [ ] No-root design implemented.
- [ ] Local-Only Hotspot is primary local link.
- [ ] Android upstream sockets are explicitly bound to a validated VPN `Network`.
- [ ] No production path binds upstream traffic to raw Wi-Fi/cellular.
- [ ] No external-VPN gateway path calls a VPN bypass API.

## Android

- [ ] VPN detection and split-tunnel exclusion detection work.
- [ ] Hotspot lifecycle works.
- [ ] foreground service survives expected UI lifecycle.
- [ ] TCP works.
- [ ] UDP works.
- [ ] DNS policy works.
- [ ] VPN loss cancels old-generation flows.
- [ ] VPN return safely resumes.
- [ ] permissions are least-privilege and current.

## Windows

- [ ] pairing works.
- [ ] Proxy Mode works.
- [ ] Full Tunnel/Wintun works.
- [ ] TCP/UDP/DNS full tunnel works.
- [ ] IPv6 is tunneled or explicitly blocked.
- [ ] kill switch prevents raw fallback.
- [ ] route/DNS state is restored on disconnect.
- [ ] crash/reboot recovery works.
- [ ] uninstall leaves networking healthy.

## Security

- [ ] production gateway requires app-layer authentication.
- [ ] long-term secrets use secure storage.
- [ ] parser fuzz/property tests pass.
- [ ] rate/resource limits exist.
- [ ] dependency vulnerabilities reviewed.
- [ ] licenses/provenance reviewed.
- [ ] logs contain no secrets.

## Leak tests

- [ ] repeated VPN-drop TCP test passes.
- [ ] UDP test passes.
- [ ] DNS leak test passes.
- [ ] IPv6 leak test passes.
- [ ] Windows TUN crash test passes.
- [ ] Android gateway crash test passes safely.
- [ ] VPN server-change test passes.

## Quality

- [ ] format/lint/type checks pass.
- [ ] unit/integration tests pass.
- [ ] no critical TODO/FIXME/stub path.
- [ ] no ignored critical errors.
- [ ] 8-hour soak has no unbounded resource growth.
- [ ] supported device/VPN matrix recorded.
- [ ] final independent gap audit completed.
- [ ] no unresolved critical/high issue.

## Release

- [ ] reproducible build inputs recorded.
- [ ] signed installers/APK as applicable.
- [ ] clean install works.
- [ ] upgrade works.
- [ ] rollback/recovery documented.
- [ ] known limitations are explicit.
