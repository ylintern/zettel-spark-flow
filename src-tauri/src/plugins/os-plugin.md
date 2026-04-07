OS Information
GitHub
npm
crates.io
API Reference
Read information about the operating system using the OS Information plugin.

Supported Platforms
This plugin requires a Rust version of at least 1.77.2

Platform	Level	Notes
windows	
linux	
macos	
android	
ios	
Setup
Install the OS Information plugin to get started.

Automatic
Manual
Use your project’s package manager to add the dependency:

npm
yarn
pnpm
deno
bun
cargo
bun tauri add os

Usage
With this plugin you can query multiple information from current operational system. See all available functions in the JavaScript API or Rust API references.

Example: OS Platform
platform returns a string describing the specific operating system in use. The value is set at compile time. Possible values are linux, macos, ios, freebsd, dragonfly, netbsd, openbsd, solaris, android, windows.

JavaScript
Rust
import { platform } from '@tauri-apps/plugin-os';
// when using `"withGlobalTauri": true`, you may use
// const { platform } = window.__TAURI__.os;

const currentPlatform = platform();
console.log(currentPlatform);
// Prints "windows" to the console

Permissions
By default all potentially dangerous plugin commands and scopes are blocked and cannot be accessed. You must modify the permissions in your capabilities configuration to enable these.

See the Capabilities Overview for more information and the step by step guide to use plugin permissions.

src-tauri/capabilities/default.json
{
  "permissions": [
    ...,
    "os:default"
  ]
}

Default Permission
This permission set configures which operating system information are available to gather from the frontend.

Granted Permissions
All information except the host name are available.

This default permission set includes the following:
allow-arch
allow-exe-extension
allow-family
allow-locale
allow-os-type
allow-platform
allow-version
Permission Table
Identifier	Description
os:allow-arch

Enables the arch command without any pre-configured scope.

os:deny-arch

Denies the arch command without any pre-configured scope.

os:allow-exe-extension

Enables the exe_extension command without any pre-configured scope.

os:deny-exe-extension

Denies the exe_extension command without any pre-configured scope.

os:allow-family

Enables the family command without any pre-configured scope.

os:deny-family

Denies the family command without any pre-configured scope.

os:allow-hostname

Enables the hostname command without any pre-configured scope.

os:deny-hostname

Denies the hostname command without any pre-configured scope.

os:allow-locale

Enables the locale command without any pre-configured scope.

os:deny-locale

Denies the locale command without any pre-configured scope.

os:allow-os-type

Enables the os_type command without any pre-configured scope.

os:deny-os-type

Denies the os_type command without any pre-configured scope.

os:allow-platform

Enables the platform command without any pre-configured scope.

os:deny-platform

Denies the platform command without any pre-configured scope.

os:allow-version

Enables the version command without any pre-configured scope.

os:deny-version

Denies the version command without any pre-configured scope.

Edit page
