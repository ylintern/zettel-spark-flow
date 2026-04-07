Autostart
GitHub
npm
crates.io
API Reference
Automatically launch your application at system startup.

Supported Platforms
This plugin requires a Rust version of at least 1.77.2

Platform	Level	Notes
windows	
linux	
macos	
android	
ios	
Setup
Install the autostart plugin to get started.

Automatic
Manual
Use your project’s package manager to add the dependency:

npm
yarn
pnpm
deno
bun
cargo
bun tauri add autostart

Usage
The autostart plugin is available in both JavaScript and Rust.

JavaScript
Rust
import { enable, isEnabled, disable } from '@tauri-apps/plugin-autostart';
// when using `"withGlobalTauri": true`, you may use
// const { enable, isEnabled, disable } = window.__TAURI__.autostart;

// Enable autostart
await enable();
// Check enable state
console.log(`registered for autostart? ${await isEnabled()}`);
// Disable autostart
disable();

Permissions
By default all potentially dangerous plugin commands and scopes are blocked and cannot be accessed. You must modify the permissions in your capabilities configuration to enable these.

See the Capabilities Overview for more information and the step by step guide to use plugin permissions.

src-tauri/capabilities/default.json
{
  "permissions": [
    ...,
    "autostart:allow-enable",
    "autostart:allow-disable",
    "autostart:allow-is-enabled"
  ]
}

Default Permission
This permission set configures if your application can enable or disable auto starting the application on boot.

Granted Permissions
It allows all to check, enable and disable the automatic start on boot.

This default permission set includes the following:
allow-enable
allow-disable
allow-is-enabled
Permission Table
Identifier	Description
autostart:allow-disable

Enables the disable command without any pre-configured scope.

autostart:deny-disable

Denies the disable command without any pre-configured scope.

autostart:allow-enable

Enables the enable command without any pre-configured scope.

autostart:deny-enable

Denies the enable command without any pre-configured scope.

autostart:allow-is-enabled

Enables the is_enabled command without any pre-configured scope.

autostart:deny-is-enabled

Denies the is_enabled command without any pre-configured scope.