# 19 — Risk Register

| Risk | Impact | Mitigation / Experiment |
|---|---|---|
| VPN app excludes VPNBridge via split tunnel | Critical leak risk | Require active default `TRANSPORT_VPN`; bind upstream sockets to VPN Network; fail closed |
| VPN disappears between validation and socket use | Critical | VPN generation + bind-before-connect + network callbacks + flow cancellation + leak tests |
| Local-Only Hotspot cannot coexist with Wi-Fi upstream on device | Medium | cellular baseline; capability/device matrix; clear UX |
| VPN blocks local LAN communication | High usability | detect gateway reachability; user guidance for VPN LAN/local sharing setting; compatibility matrix |
| OEM kills foreground service | High | correct FGS type, notification, lifecycle tests, targeted OEM guidance |
| UDP/QUIC instability | Medium | bounded UDP associations, timeouts, soak/perf tests |
| DNS leak | Critical | VPN DNS snapshot + VPN-bound DNS + Windows DNS routing + leak harness |
| IPv6 leak | Critical | dual-stack implementation or explicit block |
| Wintun integration/license misuse | High | use official signed distribution, preserve license, verify current upstream instructions |
| Windows route changes strand user offline | High | transactional snapshot/recovery journal/rollback/uninstall tests |
| Kill switch blocks phone gateway | High | explicit local bypass rule before enforcement; recovery safe mode |
| TUN-to-SOCKS dependency regression | Medium | abstraction, pin version, integration tests, ability to replace |
| Protocol/parser vulnerability | High | standard SOCKS semantics, strict bounds, fuzzing, secure channel |
| Antigravity assumes unverified API | High | mandatory primary-source research + micro-experiments |
| Remote agent cannot hardware-test | High | explicit human/hardware gate; never fake evidence |
