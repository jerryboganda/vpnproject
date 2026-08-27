# 11 — Test Strategy

## Testing layers

### Rust unit tests

- SOCKS5 parsing/encoding.
- state machines.
- authentication logic.
- UDP association lifecycle.
- config validation.
- network-generation cancellation.
- bounded queues/timeouts.

### Property/fuzz tests

Target every parser and externally supplied length/state field. Network input must never panic.

### Android tests

- permission flow.
- Local-Only Hotspot lifecycle.
- VPN detection.
- network callback ordering.
- VPN-network socket binding.
- foreground-service lifecycle.
- screen off/background.
- process/service recreation.

### Windows tests

- Wintun lifecycle abstraction.
- route transaction/rollback.
- recovery journal.
- kill-switch rules.
- DNS configuration restoration.
- elevation/helper IPC.

### End-to-end tests

- HTTP/HTTPS.
- WebSocket.
- large download/upload.
- UDP echo.
- DNS.
- QUIC/HTTP3 where tunnel supports UDP.
- IPv6 or explicit block.
- simultaneous streams.

## Continuous vs exhaustive testing

For speed, run focused deterministic tests after each change. Run the expensive device/network matrix at phase gates and especially before release. Do not defer basic correctness tests until the end.

## Failure-injection tests

Kill/restart:

- Android UI,
- Android foreground service,
- Windows UI,
- Windows privileged helper,
- TUN engine.

Toggle:

- Android VPN,
- phone airplane mode,
- cellular/Wi-Fi,
- Local-Only Hotspot,
- Windows Wi-Fi,
- sleep/resume.

## Test evidence

Every release gate records:

- git commit,
- device/OS,
- VPN provider/protocol,
- commands/test IDs,
- pass/fail,
- relevant sanitized logs,
- performance measurements.
