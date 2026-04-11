# Project CTO Decisions — Cloud & Tor Scope

**Status**: FINAL
**Last Updated**: 2026-04-11

---

## Final Decisions

1. **Stronghold remains the canonical key store**
   - All provider/API secrets remain in `tauri-plugin-stronghold`.
   - No frontend state ownership for secrets.

2. **`.env` rejected permanently**
   - `.env` is not an acceptable path for provider keys in this app.
   - No development, staging, or production exception is allowed for canonical key storage.

3. **Tor integration deferred until Block 1 completion**
   - Tor work is explicitly out of scope until Block 1 (Event Bus) is complete and stable.
   - Tor tasks must not be pulled forward into active execution while Block 1 gating is unmet.

4. **Initial Tor scope is desktop-only with strict runtime constraints**
   - Desktop-only initial target (macOS/Windows/Linux).
   - Lazy bootstrap only (start Tor path only when explicitly required by Tor-routed provider action).
   - **Strict no-silent-clearnet fallback**: if Tor route cannot be established, request must fail with explicit error state; no transparent retry over clearnet.

---

## Phase Placement Constraint

- Tor execution begins only after:
  1) Phase 1 Block 1 is complete, and
  2) clear-net cloud provider path is stable in execution board gates.
- Any Tor implementation issue must remain queued in deferred list until both conditions are true.
