SQL

Plugin providing an interface for the frontend to communicate with SQL databases through sqlx. It supports the SQLite, MySQL and PostgreSQL drivers, enabled by a Cargo feature.

Supported Platforms
This plugin requires a Rust version of at least 1.77.2

Platform	Level	Notes
windows	- yes 
linux - yes 
macos - yes 
android	- yes 
ios	- yes 
Setup - yes 
Install the SQL plugin to get started.


bun tauri add sql
## We are here.(to -do)
After installing the plugin, you must select the supported database engine. The available engines are Sqlite, MySQL and PostgreSQL. Run the following command in the src-tauri folder to enable your preferred engine:

SQLite
MySQL
PostgreSQL
cargo add tauri-plugin-sql --features sqlite

Usage
All the plugin’s APIs are available through the JavaScript guest bindings:

SQLite
MySQL
PostgreSQL
The path is relative to tauri::api::path::BaseDirectory::AppConfig.

import Database from '@tauri-apps/plugin-sql';
// when using `"withGlobalTauri": true`, you may use
// const Database = window.__TAURI__.sql;

const db = await Database.load('sqlite:test.db');
await db.execute('INSERT INTO ...');

Syntax
We use sqlx as the underlying library and adopt their query syntax.

SQLite
MySQL
PostgreSQL
Use the ”$#” syntax when substituting query data

const result = await db.execute(
  'INSERT into todos (id, title, status) VALUES ($1, $2, $3)',
  [todos.id, todos.title, todos.status]
);

const result = await db.execute(
  'UPDATE todos SET title = $1, status = $2 WHERE id = $3',
  [todos.title, todos.status, todos.id]
);

Migrations
This plugin supports database migrations, allowing you to manage database schema evolution over time.

Defining Migrations
Migrations are defined in Rust using the Migration struct. Each migration should include a unique version number, a description, the SQL to be executed, and the type of migration (Up or Down).

Example of a migration:

use tauri_plugin_sql::{Migration, MigrationKind};

let migration = Migration {
    version: 1,
    description: "create_initial_tables",
    sql: "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT);",
    kind: MigrationKind::Up,
};

Or if you want to use SQL from a file, you can include it by using include_str!:

use tauri_plugin_sql::{Migration, MigrationKind};

let migration = Migration {
    version: 1,
    description: "create_initial_tables",
    sql: include_str!("../drizzle/0000_graceful_boomer.sql"),
    kind: MigrationKind::Up,
};

Adding Migrations to the Plugin Builder
Migrations are registered with the Builder struct provided by the plugin. Use the add_migrations method to add your migrations to the plugin for a specific database connection.

Example of adding migrations:

src-tauri/src/main.rs
use tauri_plugin_sql::{Builder, Migration, MigrationKind};

fn main() {
    let migrations = vec![
        // Define your migrations here
        Migration {
            version: 1,
            description: "create_initial_tables",
            sql: "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT);",
            kind: MigrationKind::Up,
        }
    ];

    tauri::Builder::default()
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:mydatabase.db", migrations)
                .build(),
        )
        ...
}

Applying Migrations
To apply the migrations when the plugin is initialized, add the connection string to the tauri.conf.json file:

src-tauri/tauri.conf.json
{
  "plugins": {
    "sql": {
      "preload": ["sqlite:mydatabase.db"]
    }
  }
}

Alternatively, the client side load() also runs the migrations for a given connection string:

import Database from '@tauri-apps/plugin-sql';
const db = await Database.load('sqlite:mydatabase.db');

Ensure that the migrations are defined in the correct order and are safe to run multiple times.

Note

All migrations are executed within a transaction, ensuring atomicity. If any migration fails, the entire transaction is rolled back, leaving the database in a consistent state.

Migration Management
Version Control: Each migration must have a unique version number. This is crucial for ensuring the migrations are applied in the correct order.
Idempotency: Write migrations in a way that they can be safely re-run without causing errors or unintended consequences.
Testing: Thoroughly test migrations to ensure they work as expected and do not compromise the integrity of your database.
Permissions
By default all potentially dangerous plugin commands and scopes are blocked and cannot be accessed. You must modify the permissions in your capabilities configuration to enable these.

See the Capabilities Overview for more information and the step by step guide to use plugin permissions.

src-tauri/capabilities/default.json
{
  "permissions": [
    ...,
    "sql:default",
    "sql:allow-execute",
  ]
}

Default Permission
Default Permissions
This permission set configures what kind of database operations are available from the sql plugin.

Granted Permissions
All reading related operations are enabled. Also allows to load or close a connection.

This default permission set includes the following:
allow-close
allow-load
allow-select
Permission Table
Identifier	Description
sql:allow-close

Enables the close command without any pre-configured scope.

sql:deny-close

Denies the close command without any pre-configured scope.

sql:allow-execute

Enables the execute command without any pre-configured scope.

sql:deny-execute

Denies the execute command without any pre-configured scope.

sql:allow-load

Enables the load command without any pre-configured scope.

sql:deny-load

Denies the load command without any pre-configured scope.

sql:allow-select

Enables the select command without any pre-configured scope.

sql:deny-select

Denies the select command without any pre-configured scope.