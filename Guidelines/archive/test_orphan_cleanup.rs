#[tokio::main]
async fn main() {
    use std::{fs, path::Path};
    use uuid::Uuid;

    // Setup test directory
    let root = std::env::temp_dir().join(format!("vibo-orphan-test-{}", Uuid::new_v4()));
    let vault_dir = root.join("myspace");
    let db_path = root.join("test.db");

    fs::create_dir_all(&vault_dir).expect("create vault dir");

    // Initialize pool and db
    let pool = app::db::init_pool(&db_path)
        .await
        .expect("init pool");

    // Create initial note in "notes" folder
    let note1 = app::models::WorkspaceNote {
        id: "test-move-1".to_string(),
        title: "Move Test".to_string(),
        content: "Testing move".to_string(),
        tags: vec![],
        status: "inbox".to_string(),
        position: 0,
        is_kanban: false,
        created_at: "2026-04-18T00:00:00Z".to_string(),
        updated_at: "2026-04-18T00:00:00Z".to_string(),
        folder: Some("notes".to_string()),
        is_encrypted: Some(false),
    };

    let security = app::security::SecurityState::new(root.join("secure.hold"));
    app::db::save_note(&pool, &vault_dir, &note1, &security)
        .await
        .expect("save note");

    let original_path = vault_dir.join("notes/test-move-1.md");
    assert!(
        original_path.exists(),
        "Original file should exist at notes/test-move-1.md"
    );
    println!("✓ Note created at: {:?}", original_path);

    // Now move it to Research folder
    let mut note1_moved = note1.clone();
    note1_moved.folder = Some("Research".to_string());
    note1_moved.updated_at = "2026-04-18T00:01:00Z".to_string();

    app::db::save_note(&pool, &vault_dir, &note1_moved, &security)
        .await
        .expect("save note after move");

    // Check: old file should be GONE, new file should exist
    let new_path = vault_dir.join("Research/test-move-1.md");
    assert!(
        !original_path.exists(),
        "Old file should be deleted after move. Still exists at: {:?}",
        original_path
    );
    println!("✓ Old file cleaned up at: {:?}", original_path);

    assert!(
        new_path.exists(),
        "New file should exist at Research/test-move-1.md"
    );
    println!("✓ Note moved to: {:?}", new_path);

    // Verify content is intact
    let content = fs::read_to_string(&new_path).expect("read moved file");
    assert!(
        content.contains("folder: Research"),
        "Moved file should have updated folder metadata"
    );
    println!("✓ Moved file has correct metadata");

    // Cleanup
    drop(pool);
    fs::remove_dir_all(&root).expect("cleanup");
    println!("\n✅ All tests passed!");
}
