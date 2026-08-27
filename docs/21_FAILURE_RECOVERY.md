# 21 — Failure and Recovery Specification

## Android VPN loss

Expected behavior:

- VPN callback invalidates current generation.
- forwarding gate closes.
- old upstream sockets close.
- UDP associations clear.
- Windows receives unsafe state if control path remains.
- no new egress is permitted.
- safe revalidation begins when VPN returns.

## Android hotspot loss

- stop local listener/session.
- Windows stays fail closed.
- attempt hotspot recreation only from permitted lifecycle context.
- communicate new SSID/subnet/pairing metadata as needed.

## Android process/service crash

No raw Internet route exists through Local-Only Hotspot. Windows kill switch remains fail closed. On restart, a new authenticated session is required/resumed according to protocol.

## Windows UI crash

Privileged tunnel/kill-switch component should have a defined ownership model. Do not leave raw routing exposed. A service may keep protection or fail closed until explicit recovery.

## Windows privileged helper/TUN crash

- kill switch remains on,
- route transaction is not silently reverted to raw Internet,
- watchdog attempts safe restart,
- recovery UI explains state.

## Windows reboot with stale journal

At startup, helper detects unfinished transaction and either:

- restores original network state if no protected session exists, or
- reconstructs protected state only after phone/VPN validation.

Never leave ambiguous partial routes.

## Intentional disconnect

Order:

1. stop new TUN flows,
2. remove protected routes,
3. remove/relax kill switch at the correct safe point,
4. restore DNS/interface state,
5. verify normal connectivity restoration,
6. clear recovery journal.
