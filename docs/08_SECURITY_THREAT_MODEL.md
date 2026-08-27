# 08 — Security Threat Model

## Assets

- protected Internet path,
- pairing/device identity keys,
- hotspot credentials,
- session secrets,
- Windows routing/firewall state,
- user traffic confidentiality/integrity,
- privacy-sensitive diagnostics.

## Trust boundaries

1. Windows application traffic -> privileged Windows tunnel component.
2. Windows tunnel -> local hotspot link.
3. Local client -> Android gateway parser/authentication.
4. Android Rust core -> Android network-binding bridge.
5. Android VPNBridge -> third-party VPN network.
6. Build system -> third-party dependencies/binaries.

## Threats and mitigations

### Unauthorized hotspot client uses gateway

Mitigate with app-layer authentication, limited listener interface, rate limits, paired device identity, and credential rotation.

### VPN disappears and raw Internet is used

Mitigate by explicit per-socket binding to the validated VPN `Network`, generation invalidation, immediate flow cancellation, Windows kill switch, and leak tests.

### VPN excludes VPNBridge by split tunneling

Require VPN transport for VPNBridge's active default network. Disable forwarding if absent.

### DNS/IPv6 bypass

Route/protect explicitly; block unsupported families.

### Malformed SOCKS/control input

Strict parser, bounded lengths, fuzzing, no unchecked indexing, no panic on network input.

### Resource exhaustion

Connection limits, UDP flow caps, memory limits, idle expiry, authentication throttling, bounded queues.

### Local MITM

Production authenticated secure channel; do not trust hotspot encryption alone.

### Windows local privilege attack

Minimal privileged helper, authenticated IPC, fixed command schema, no arbitrary command execution.

### Supply-chain compromise

Lockfiles, checksums/provenance for bundled binary assets, vulnerability scans, license review, minimal dependency count, reproducible release procedure.

### Sensitive logs

Redact credentials/keys and avoid visited-domain logging by default. Diagnostics should use event codes and aggregate counters.

## Security gates

- dependency audit,
- static analysis,
- fuzz/property tests,
- protocol abuse tests,
- privilege boundary review,
- route/killswitch bypass tests,
- manual threat-model re-review before release.
