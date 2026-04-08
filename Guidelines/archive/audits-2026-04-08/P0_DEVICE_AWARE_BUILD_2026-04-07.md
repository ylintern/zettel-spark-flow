# P0 Device-Aware Build

## Date
- 2026-04-07

## Scope
- production macOS bundle
- device-aware security selection

## Done
- Tauri identifier changed to `com.vibo.zettel-spark-flow`
- production `.app` bundle built successfully
- Rust now reports device capabilities to the frontend
- onboarding auth methods now filter by device class
- lock screen now chooses device-specific unlock path

## Verified
- `.app` bundle exists:
  - `src-tauri/target/release/bundle/macos/vibo.app`
- `Info.plist` contains:
  - `CFBundleIdentifier = com.vibo.zettel-spark-flow`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `bun run build`
- `bun tauri build --bundles app`

## Important Caveat
- full `bun tauri build` failed at the `.dmg` stage
- the `.app` bundle was still produced correctly
- interactive launch of the `.app` was not verified from this terminal session

## Security Read
- desktop builds no longer offer mobile biometric unlock
- mobile biometric unlock remains hidden unless the backend says it is actually offerable
- passphrase and PIN remain the trusted setup paths until hardware-backed biometric release exists
