# Antigravity Adversarial Bug-Hunt Prompt

Try to break VPNBridge rather than prove it works.

Prioritize scenarios that could leak raw Internet or strand the user's Windows networking:

- VPN disconnect/reconnect races,
- VPN replacement while sockets are opening,
- split-tunnel exclusion,
- stale VPN network handle,
- TCP/UDP traffic at the exact moment protection changes,
- DNS and IPv6 bypass,
- Android service/process death,
- hotspot recreation/subnet change,
- Windows TUN/helper/UI crash,
- sleep/resume,
- route transaction partial failure,
- kill-switch partial failure,
- malformed/hostile SOCKS/control clients,
- connection/UDP floods,
- low-memory/resource exhaustion.

For every discovered issue:

1. create a reproducible failing test where feasible,
2. identify root cause,
3. implement the real fix rather than masking symptoms,
4. run affected regression tests,
5. update `KNOWN_ISSUES.md` and `TEST_EVIDENCE.md`,
6. rerun the adversarial scenario.

Continue until a fresh bug-hunt pass finds no critical/high defect.
