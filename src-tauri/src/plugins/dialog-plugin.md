Dialog
GitHub
npm
crates.io
API Reference
Native system dialogs for opening and saving files along with message dialogs.

Supported Platforms
This plugin requires a Rust version of at least 1.77.2

Platform	Level	Notes
windows	
linux	
macos	
android	
Does not support folder picker

ios	
Does not support folder picker

Setup
Install the dialog plugin to get started.

Automatic
Manual
Use your project’s package manager to add the dependency:

npm
yarn
pnpm
deno
bun
cargo
bun tauri add dialog

Usage
The dialog plugin is available in both JavaScript and Rust. Here’s how you can use it:

in JavaScript:

Create Yes/No Dialog
Create Ok/Cancel Dialog
Create Message Dialog
Open a File Selector Dialog
Save to File Dialog
in Rust:

Build an Ask Dialog
Build a Message Dialog
Build a File Selector Dialog
Note

The file dialog APIs returns file system paths on Linux, Windows and macOS.

On iOS, a file://<path> URIs are returned.

On Android, content URIs are returned.

The filesystem plugin works with any path format out of the box.

JavaScript
See all Dialog Options at the JavaScript API reference.

Create Yes/No Dialog
Shows a question dialog with Yes and No buttons.

import { ask } from '@tauri-apps/plugin-dialog';
// when using `"withGlobalTauri": true`, you may use
// const { ask } = window.__TAURI__.dialog;

// Create a Yes/No dialog
const answer = await ask('This action cannot be reverted. Are you sure?', {
  title: 'Tauri',
  kind: 'warning',
});

console.log(answer);
// Prints boolean to the console

Create Ok/Cancel Dialog
Shows a question dialog with Ok and Cancel buttons.

import { confirm } from '@tauri-apps/plugin-dialog';
// when using `"withGlobalTauri": true`, you may use
// const { confirm } = window.__TAURI__.dialog;

// Creates a confirmation Ok/Cancel dialog
const confirmation = await confirm(
  'This action cannot be reverted. Are you sure?',
  { title: 'Tauri', kind: 'warning' }
);

console.log(confirmation);
// Prints boolean to the console

Create Message Dialog
Shows a message dialog with an Ok button. Keep in mind that if the user closes the dialog it will return false.

import { message } from '@tauri-apps/plugin-dialog';
// when using `"withGlobalTauri": true`, you may use
// const { message } = window.__TAURI__.dialog;

// Shows message
await message('File not found', { title: 'Tauri', kind: 'error' });

Open a File Selector Dialog
Open a file/directory selection dialog.

The multiple option controls whether the dialog allows multiple selection or not, while the directory, whether is a directory selection or not.

import { open } from '@tauri-apps/plugin-dialog';
// when using `"withGlobalTauri": true`, you may use
// const { open } = window.__TAURI__.dialog;

// Open a dialog
const file = await open({
  multiple: false,
  directory: false,
});
console.log(file);
// Prints file path or URI

Save to File Dialog
Open a file/directory save dialog.

import { save } from '@tauri-apps/plugin-dialog';
// when using `"withGlobalTauri": true`, you may use
// const { save } = window.__TAURI__.dialog;

// Prompt to save a 'My Filter' with extension .png or .jpeg
const path = await save({
  filters: [
    {
      name: 'My Filter',
      extensions: ['png', 'jpeg'],
    },
  ],
});
console.log(path);
// Prints the chosen path

Rust
Refer to the Rust API reference to see all available options.

Build an Ask Dialog
Shows a question dialog with Absolutely and Totally buttons.

use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};

let answer = app.dialog()
        .message("Tauri is Awesome")
        .title("Tauri is Awesome")
        .buttons(MessageDialogButtons::OkCancelCustom("Absolutely", "Totally"))
        .blocking_show();

If you need a non blocking operation you can use show() instead:

use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};

app.dialog()
    .message("Tauri is Awesome")
    .title("Tauri is Awesome")
   .buttons(MessageDialogButtons::OkCancelCustom("Absolutely", "Totally"))
    .show(|result| match result {
        true => // do something,
        false =>// do something,
    });

Build a Message Dialog
Shows a message dialog with an Ok button. Keep in mind that if the user closes the dialog it will return false.

use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

let ans = app.dialog()
    .message("File not found")
    .kind(MessageDialogKind::Error)
    .title("Warning")
    .blocking_show();

If you need a non blocking operation you can use show() instead:

use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

app.dialog()
    .message("Tauri is Awesome")
    .kind(MessageDialogKind::Info)
    .title("Information")
    .buttons(MessageDialogButtons::OkCustom("Absolutely"))
    .show(|result| match result {
        true => // do something,
        false => // do something,
    });

Build a File Selector Dialog
Pick Files
use tauri_plugin_dialog::DialogExt;

let file_path = app.dialog().file().blocking_pick_file();
// return a file_path `Option`, or `None` if the user closes the dialog

If you need a non blocking operation you can use pick_file() instead:

use tauri_plugin_dialog::DialogExt;

app.dialog().file().pick_file(|file_path| {
    // return a file_path `Option`, or `None` if the user closes the dialog
    })

Save Files
use tauri_plugin_dialog::DialogExt;

let file_path = app
    .dialog()
    .file()
    .add_filter("My Filter", &["png", "jpeg"])
    .blocking_save_file();
    // do something with the optional file path here
    // the file path is `None` if the user closed the dialog

or, alternatively:

use tauri_plugin_dialog::DialogExt;

app.dialog()
    .file()
    .add_filter("My Filter", &["png", "jpeg"])
    .pick_file(|file_path| {
        // return a file_path `Option`, or `None` if the user closes the dialog
    });

Default Permission
This permission set configures the types of dialogs available from the dialog plugin.

Granted Permissions
All dialog types are enabled.

This default permission set includes the following:
allow-message
allow-save
allow-open
Permission Table
Identifier	Description
dialog:allow-ask

Enables the ask command without any pre-configured scope. (DEPRECATED: This is now an alias to allow-message and will be removed in v3)

dialog:deny-ask

Denies the ask command without any pre-configured scope. (DEPRECATED: This is now an alias to deny-message and will be removed in v3)

dialog:allow-message

Enables the message command without any pre-configured scope.

dialog:deny-message

Denies the message command without any pre-configured scope.

dialog:allow-open

Enables the open command without any pre-configured scope.

dialog:deny-open

Denies the open command without any pre-configured scope.

dialog:allow-save

Enables the save command without any pre-configured scope.

dialog:deny-save

Denies the save command without any pre-configured scope.

dialog:allow-confirm

Enables the confirm command without any pre-configured scope. (DEPRECATED: This is now an alias to allow-message and will be removed in v3)

dialog:deny-confirm

Denies the confirm command without any pre-configured scope. (DEPRECATED: This is now an alias to deny-message and will be removed in v3)