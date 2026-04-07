# README Map

## Purpose
- quick digest of `/README.md`
- not a duplicate

## Current Signals
- architecture is opinionated
- inference stack is already decided at high level
- several older options are explicitly rejected
- Swiftide is bounded, not the owner of everything
- app has 2 flows:
  - user
  - agent
- no sidecars remains a hard rule

## Operational Read
- use README for technical direction
- do not let current UI drift against it
- delay inference implementation until app core is stable
- keep passphrase/secrets in Rust-owned secure storage, not browser crypto

## Watchouts
- model/package UI currently diverges from README decisions
- current chat flow is still placeholder-oriented
- biometrics and safe vault reset still need implementation
