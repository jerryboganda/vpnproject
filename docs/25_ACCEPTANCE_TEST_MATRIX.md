# 25 — Acceptance Test Matrix

Every row applicable to the release must have recorded evidence.

| ID | Test | Expected result |
|---|---|---|
| A01 | Android VPN absent, user starts sharing | Forwarding disabled; clear error |
| A02 | VPN active and covers VPNBridge | Protected state becomes available |
| A03 | VPN excludes VPNBridge via split tunnel | Sharing blocked |
| A04 | Start/stop Local-Only Hotspot repeatedly | No leaked reservation/resource; reconnect works |
| A05 | TCP HTTPS through manual SOCKS | Public egress is VPN path |
| A06 | Large TCP download/upload | Correct data; stable memory |
| A07 | UDP echo | Works through VPN-bound UDP |
| A08 | DNS A/AAAA | Resolves through protected path |
| A09 | VPN disconnect during established TCP | Flow fails; no raw continuation |
| A10 | VPN disconnect during rapid new connects | No successful raw egress |
| A11 | VPN disconnect during UDP traffic | No raw UDP egress |
| A12 | VPN server switch/network replacement | Old generation closed; safe reconnect |
| A13 | Android UI killed | Foreground service behavior matches design |
| A14 | Android service killed | Windows fails closed |
| A15 | Screen off 30+ min | Expected stable sharing or documented OEM limitation |
| W01 | Windows pairing | Only approved client connects |
| W02 | Wrong pairing/auth secret | Rejected and rate limited |
| W03 | Proxy Mode browser | Works through phone VPN |
| W04 | Wintun Full Tunnel | Non-proxy-aware application works |
| W05 | Full Tunnel UDP/QUIC | Works or explicit supported limitation recorded |
| W06 | DNS full tunnel | No physical-adapter DNS leak |
| W07 | IPv6 | Tunneled or blocked; no bypass |
| W08 | Android VPN drops | Windows kill switch prevents fallback |
| W09 | TUN process crashes | Raw Internet remains blocked |
| W10 | Windows UI crashes | Protection state remains safe |
| W11 | Windows sleep/resume | Recovers without raw leak |
| W12 | Hotspot subnet changes | rediscovery/recovery works |
| W13 | Intentional disconnect | Original network restored |
| W14 | Windows reboot after interrupted session | recovery journal restores safe state |
| W15 | Uninstall | no broken routes/DNS/firewall residue |
| S01 | Fuzz SOCKS/control parser | no panic/memory safety issue |
| S02 | Connection flood | limits/backpressure work |
| S03 | UDP mapping flood | bounded resource usage |
| S04 | Invalid protocol states/replay | rejected safely |
| P01 | TCP throughput benchmark | meets recorded target/baseline |
| P02 | latency benchmark | meets recorded target/baseline |
| P03 | 8-hour soak | no unbounded growth or fatal error |
| P04 | 10k connection churn | no leak/crash; bounded resources |
