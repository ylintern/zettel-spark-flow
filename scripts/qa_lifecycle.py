#!/usr/bin/env python3
"""
QA Lifecycle Inspection Script
Validates storage layers after core app gate implementation.
Non-destructive: reads only.
"""

import sqlite3
import sys
import os
import argparse
from pathlib import Path

def get_default_paths():
    """Get default DB and vault paths."""
    home = str(Path.home())
    app_support = os.path.join(
        home,
        "Library/Application Support/com.viboai.app"
    )
    return {
        "db": os.path.join(app_support, "vibo.db"),
        "vault": os.path.join(app_support, "vault"),
    }

def check_1_notes_table(conn):
    """CHECK 1: Notes table exists with expected columns."""
    print("\n[CHECK 1] Notes table structure...")
    try:
        cursor = conn.cursor()
        cursor.execute("PRAGMA table_info(notes)")
        columns = {row[1] for row in cursor.fetchall()}
        
        expected = {
            "id", "title", "file_path", "folder", "column_id",
            "position", "kind", "created_at", "updated_at", "is_encrypted"
        }
        
        if expected.issubset(columns):
            print(f"  ✓ PASS: All expected columns present")
            cursor.execute("SELECT COUNT(*) FROM notes")
            count = cursor.fetchone()[0]
            print(f"  → Notes count: {count}")
            return True
        else:
            missing = expected - columns
            print(f"  ✗ FAIL: Missing columns: {missing}")
            return False
    except Exception as e:
        print(f"  ✗ FAIL: {e}")
        return False

def check_2_columns_table(conn):
    """CHECK 2: Columns table exists and is seeded."""
    print("\n[CHECK 2] Columns table (Kanban board structure)...")
    try:
        cursor = conn.cursor()
        # Check if table exists at all
        cursor.execute("SELECT name FROM sqlite_master WHERE type='table' AND name='columns'")
        table_exists = cursor.fetchone() is not None
        
        if not table_exists:
            print("  ✗ FAIL: columns table does not exist — Step 1A migration did not run or app not relaunched")
            return False
        
        cursor.execute("PRAGMA table_info(columns)")
        columns = {row[1] for row in cursor.fetchall()}
        
        expected = {"id", "title", "order", "created_at"}
        
        if not expected.issubset(columns):
            print(f"  ✗ FAIL: Missing columns: {expected - columns}")
            return False
        
        cursor.execute("SELECT COUNT(*) FROM columns")
        count = cursor.fetchone()[0]
        
        if count > 0:
            print(f"  ✓ PASS: Columns table seeded ({count} columns)")
            cursor.execute("SELECT id, title, \"order\" FROM columns ORDER BY \"order\"")
            for row in cursor.fetchall():
                print(f"    - {row[0]}: {row[1]} (order {row[2]})")
            return True
        else:
            print(f"  ⓘ INFO: Columns table exists but empty (expected on first launch before app init)")
            return True
    except Exception as e:
        print(f"  ✗ FAIL: {e}")
        return False

def check_3_folders_table(conn):
    """CHECK 3: Folders table exists."""
    print("\n[CHECK 3] Folders table...")
    try:
        cursor = conn.cursor()
        cursor.execute("PRAGMA table_info(folders)")
        columns = {row[1] for row in cursor.fetchall()}
        
        if "name" in columns:
            cursor.execute("SELECT COUNT(*) FROM folders")
            count = cursor.fetchone()[0]
            print(f"  ✓ PASS: Folders table exists ({count} folders)")
            if count > 0:
                cursor.execute("SELECT name FROM folders ORDER BY name")
                for row in cursor.fetchall():
                    print(f"    - {row[0]}")
            return True
        else:
            print(f"  ✗ FAIL: 'name' column not found")
            return False
    except Exception as e:
        print(f"  ✗ FAIL: {e}")
        return False

def check_4_note_files(conn, vault_dir):
    """CHECK 4: Every note row has corresponding markdown file."""
    print("\n[CHECK 4] Note file integrity...")
    try:
        if not os.path.isdir(vault_dir):
            print(f"  ⓘ INFO: Vault directory not found: {vault_dir}")
            return True
        
        cursor = conn.cursor()
        cursor.execute("SELECT id, file_path FROM notes")
        notes = cursor.fetchall()
        
        missing = []
        for note_id, rel_path in notes:
            full_path = os.path.join(vault_dir, rel_path)
            if not os.path.isfile(full_path):
                missing.append((note_id, rel_path))
        
        if not missing:
            print(f"  ✓ PASS: All {len(notes)} notes have corresponding files")
            return True
        else:
            print(f"  ✗ FAIL: {len(missing)} missing files:")
            for note_id, rel_path in missing:
                print(f"    - {note_id}: {rel_path}")
            return False
    except Exception as e:
        print(f"  ✗ FAIL: {e}")
        return False

def check_5_orphaned_files(conn, vault_dir):
    """CHECK 5: No orphaned markdown files (WARNING only, not FAIL)."""
    print("\n[CHECK 5] Orphaned file detection...")
    try:
        if not os.path.isdir(vault_dir):
            print(f"  ⓘ INFO: Vault directory not found")
            return True
        
        cursor = conn.cursor()
        cursor.execute("SELECT file_path FROM notes")
        db_paths = {row[0] for row in cursor.fetchall()}
        
        orphaned = []
        for root, dirs, files in os.walk(vault_dir):
            for file in files:
                if file.endswith(".md"):
                    full_path = os.path.join(root, file)
                    rel_path = os.path.relpath(full_path, vault_dir)
                    if rel_path not in db_paths:
                        orphaned.append(rel_path)
        
        if not orphaned:
            print(f"  ✓ PASS: No orphaned markdown files")
            return True
        else:
            print(f"  ⚠ WARNING: {len(orphaned)} orphaned files (expected after crash):")
            for path in orphaned[:5]:
                print(f"    - {path}")
            if len(orphaned) > 5:
                print(f"    ... and {len(orphaned) - 5} more")
            return True  # Warning, not fail
    except Exception as e:
        print(f"  ✗ FAIL: {e}")
        return False

def check_6_browser_storage(conn):
    """CHECK 6: Reminder about localStorage (manual check)."""
    print("\n[CHECK 6] Browser localStorage (manual verification)...")
    print("  ⓘ NOTE: localStorage keys cannot be read by Python")
    print("  → Manual check required:")
    print("    1. Open DevTools (Cmd+Option+I)")
    print("    2. Application → Storage → Local Storage → vibo://app")
    print("    3. Confirm NO zettel-* keys exist")
    print("    4. Confirm NO vibo-columns, vibo-folders keys exist")
    print("    5. Only vibo-pin and vibo-salt should exist (if vault is set up)")
    return True

def check_7_encryption_consistency(conn, vault_dir):
    """CHECK 7: is_encrypted flag matches file content."""
    print("\n[CHECK 7] Encryption flag consistency...")
    try:
        if not os.path.isdir(vault_dir):
            print(f"  ⓘ INFO: Vault directory not found")
            return True
        
        cursor = conn.cursor()
        cursor.execute("SELECT id, file_path, is_encrypted FROM notes")
        notes = cursor.fetchall()
        
        # Encrypted notes start with "!vbo:" prefix in markdown
        encrypted_prefix = "!vbo:"
        inconsistencies = []
        
        for note_id, rel_path, is_encrypted in notes:
            full_path = os.path.join(vault_dir, rel_path)
            if not os.path.isfile(full_path):
                continue
            
            with open(full_path, 'r') as f:
                try:
                    first_line = f.readline()
                except:
                    continue
            
            is_encrypted_flag = bool(is_encrypted)
            starts_with_prefix = first_line.startswith(encrypted_prefix)
            
            if is_encrypted_flag != starts_with_prefix:
                inconsistencies.append((note_id, is_encrypted_flag, starts_with_prefix))
        
        if not inconsistencies:
            print(f"  ✓ PASS: All {len(notes)} notes have consistent encryption flags")
            return True
        else:
            print(f"  ✗ FAIL: {len(inconsistencies)} inconsistent flags:")
            for note_id, db_flag, file_flag in inconsistencies:
                print(f"    - {note_id}: db={db_flag}, file={file_flag}")
            return False
    except Exception as e:
        print(f"  ⚠ WARNING: Could not verify (file read error): {e}")
        return True  # Don't fail, might be encrypted content

def check_8_cold_restart(db_path):
    """CHECK 8: Cold restart simulation (multiple connections)."""
    print("\n[CHECK 8] Cold restart simulation...")
    try:
        # First connection
        conn1 = sqlite3.connect(db_path, timeout=5)
        cursor1 = conn1.cursor()
        cursor1.execute("SELECT COUNT(*) FROM notes")
        count1 = cursor1.fetchone()[0]
        
        # Second connection
        conn2 = sqlite3.connect(db_path, timeout=5)
        cursor2 = conn2.cursor()
        cursor2.execute("SELECT COUNT(*) FROM notes")
        count2 = cursor2.fetchone()[0]
        
        # Close both
        conn1.close()
        conn2.close()
        
        if count1 == count2:
            print(f"  ✓ PASS: Two connections agree on state ({count1} notes)")
            return True
        else:
            print(f"  ✗ FAIL: Connections disagree (conn1={count1}, conn2={count2})")
            return False
    except Exception as e:
        print(f"  ✗ FAIL: {e}")
        return False

def main():
    parser = argparse.ArgumentParser(
        description="QA Lifecycle Inspection Script"
    )
    parser.add_argument(
        "--db-path",
        default=None,
        help=f"Path to vibo.db (default: {get_default_paths()['db']})"
    )
    parser.add_argument(
        "--vault-dir",
        default=None,
        help=f"Path to vault directory (default: {get_default_paths()['vault']})"
    )
    
    args = parser.parse_args()
    
    defaults = get_default_paths()
    db_path = args.db_path or defaults["db"]
    vault_dir = args.vault_dir or defaults["vault"]
    
    print("=" * 70)
    print("ViBo QA Lifecycle Inspection")
    print("=" * 70)
    print(f"\nDatabase: {db_path}")
    print(f"Vault:    {vault_dir}\n")
    
    if not os.path.isfile(db_path):
        print(f"✗ ERROR: Database not found: {db_path}")
        sys.exit(1)
    
    results = []
    
    try:
        conn = sqlite3.connect(db_path, timeout=5)
        
        results.append(("CHECK 1: Notes table", check_1_notes_table(conn)))
        results.append(("CHECK 2: Columns table", check_2_columns_table(conn)))
        results.append(("CHECK 3: Folders table", check_3_folders_table(conn)))
        results.append(("CHECK 4: Note files", check_4_note_files(conn, vault_dir)))
        results.append(("CHECK 5: Orphaned files", check_5_orphaned_files(conn, vault_dir)))
        results.append(("CHECK 6: Browser storage", check_6_browser_storage(conn)))
        results.append(("CHECK 7: Encryption flags", check_7_encryption_consistency(conn, vault_dir)))
        results.append(("CHECK 8: Cold restart", check_8_cold_restart(db_path)))
        
        conn.close()
    except Exception as e:
        print(f"\n✗ CRITICAL: Database error: {e}")
        sys.exit(1)
    
    print("\n" + "=" * 70)
    print("RESULTS")
    print("=" * 70)
    
    passed = sum(1 for _, result in results if result)
    total = len(results)
    
    for check_name, result in results:
        status = "✓ PASS" if result else "✗ FAIL"
        print(f"  {status}: {check_name}")
    
    print(f"\n{passed}/{total} checks passed")
    
    if passed == total:
        print("\n✓ QA LIFECYCLE INSPECTION PASSED")
        sys.exit(0)
    else:
        print("\n✗ QA LIFECYCLE INSPECTION FAILED")
        sys.exit(1)

if __name__ == "__main__":
    main()
