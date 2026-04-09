#!/usr/bin/env python3
"""
QA P0 Tail — Phase 0 Sign-Off Verification
Run after completing manual QA steps in userrequest.md.
Verifies: note/task persistence, kind separation, file integrity.
"""

import sqlite3
import os
import sys
import argparse
from pathlib import Path

def resolve_paths(dev: bool) -> dict:
    base = "com.viboai.app.dev" if dev else "com.viboai.app"
    root = Path.home() / "Library" / "Application Support" / base
    # Canonical target: database/ — code rename from vault/ pending Lead approval
    # Script checks both so QA works before and after the rename
    db_root = root / "database" if (root / "database").exists() else root / "vault"
    return {
        "root": root,
        "db": root / "vibo.db",
        "db_root": db_root,
        "notes_dir": db_root / "notes",
        "tasks_dir": db_root / "tasks",
    }

def check_vault_dirs(paths: dict) -> bool:
    print("\n[T1] Data directory structure...")
    print(f"  Base: {paths['db_root']}")
    ok = True
    for key in ["notes_dir", "tasks_dir"]:
        d = paths[key]
        if d.exists():
            count = len(list(d.glob("*.md")))
            print(f"  ✓ {d.name}/  ({count} .md files)")
        else:
            print(f"  ✗ MISSING: {d}")
            ok = False
    return ok

def check_notes_persist(conn, test_title: str = "Test Note P0-T4") -> bool:
    print(f"\n[T4-notes] Note persistence (looking for '{test_title}')...")
    cur = conn.cursor()
    cur.execute("SELECT id, title, kind FROM notes WHERE title = ? AND kind = 'note'", (test_title,))
    row = cur.fetchone()
    if row:
        print(f"  ✓ Found note: id={row[0]}, kind={row[2]}")
        return True
    cur.execute("SELECT COUNT(*) FROM notes WHERE kind = 'note'")
    total = cur.fetchone()[0]
    print(f"  ✗ '{test_title}' not found. Total notes: {total}")
    return False

def check_tasks_persist(conn, test_title: str = "Test Task P0-T4") -> bool:
    print(f"\n[T4-tasks] Task persistence (looking for '{test_title}')...")
    cur = conn.cursor()
    cur.execute("SELECT id, title, kind FROM notes WHERE title = ? AND kind = 'task'", (test_title,))
    row = cur.fetchone()
    if row:
        print(f"  ✓ Found task: id={row[0]}, kind={row[2]}")
        return True
    cur.execute("SELECT COUNT(*) FROM notes WHERE kind = 'task'")
    total = cur.fetchone()[0]
    print(f"  ✗ '{test_title}' not found. Total tasks: {total}")
    return False

def check_kind_separation(conn) -> bool:
    print("\n[T4-sep] Note/task kind separation...")
    cur = conn.cursor()
    cur.execute("SELECT kind, COUNT(*) FROM notes GROUP BY kind")
    rows = cur.fetchall()
    kinds = {r[0]: r[1] for r in rows}
    notes = kinds.get("note", 0)
    tasks = kinds.get("task", 0)
    unknown = {k: v for k, v in kinds.items() if k not in ("note", "task")}
    print(f"  notes={notes}, tasks={tasks}", end="")
    if unknown:
        print(f"  ⚠  unknown kinds: {unknown}", end="")
    print()
    if unknown:
        print("  ✗ FAIL: unexpected kind values in notes table")
        return False
    print("  ✓ PASS: only 'note' and 'task' kinds present")
    return True

def check_file_integrity(conn, paths: dict) -> bool:
    print("\n[T4-files] File integrity (every DB row has a .md file)...")
    cur = conn.cursor()
    cur.execute("SELECT id, file_path FROM notes")
    rows = cur.fetchall()
    missing = []
    for note_id, rel_path in rows:
        full = paths["db_root"] / rel_path
        if not full.exists():
            missing.append((note_id, rel_path))
    if missing:
        for note_id, p in missing[:5]:
            print(f"  ✗ Missing: {note_id} → {p}")
        print(f"  ✗ FAIL: {len(missing)} missing files")
        return False
    print(f"  ✓ PASS: all {len(rows)} rows have .md files")
    return True

def check_encryption_flags(conn, paths: dict) -> bool:
    print("\n[T5] Encryption flag consistency...")
    ENCRYPTED_PREFIX = "!vibo-encrypted:v1:"
    cur = conn.cursor()
    cur.execute("SELECT id, file_path, is_encrypted FROM notes")
    rows = cur.fetchall()
    bad = []
    for note_id, rel_path, is_enc in rows:
        full = paths["db_root"] / rel_path
        if not full.exists():
            continue
        try:
            with open(full, encoding="utf-8") as f:
                content = f.read()
            # Find body after YAML front matter
            body = content
            if content.startswith("---\n"):
                idx = content.find("\n---\n", 4)
                if idx != -1:
                    body = content[idx + 5:].lstrip("\n")
            file_encrypted = body.startswith(ENCRYPTED_PREFIX)
            if bool(is_enc) != file_encrypted:
                bad.append((note_id, bool(is_enc), file_encrypted))
        except Exception:
            pass
    if bad:
        for note_id, db_flag, file_flag in bad:
            print(f"  ✗ {note_id}: db={db_flag}, file={file_flag}")
        print(f"  ✗ FAIL: {len(bad)} inconsistencies")
        return False
    print(f"  ✓ PASS: {len(rows)} notes — encryption flags consistent")
    return True

def main():
    parser = argparse.ArgumentParser(description="Phase 0 Tail QA")
    parser.add_argument("--dev", action="store_true", help="Use com.viboai.app.dev data dir")
    parser.add_argument("--note", default="Testinfolder69", help="Note title to verify")
    parser.add_argument("--task", default="Countertest69", help="Task title to verify")
    args = parser.parse_args()

    paths = resolve_paths(args.dev)
    label = "DEV" if args.dev else "RELEASE"

    print("=" * 60)
    print(f"ViBo — Phase 0 Tail QA  [{label}]")
    print("=" * 60)
    print(f"Root:    {paths['root']}")
    print(f"DB:      {paths['db']}")
    print(f"DB dir:  {paths['db_root']} ({'EXISTS' if paths['db_root'].exists() else 'MISSING'})")

    if not paths["db"].exists():
        print(f"\n✗ DB not found: {paths['db']}")
        print("  → Launch the app first and complete onboarding, then re-run.")
        sys.exit(1)

    results = []

    # T1: dirs
    results.append(("T1: vault dirs", check_vault_dirs(paths)))

    try:
        conn = sqlite3.connect(str(paths["db"]), timeout=5)
        results.append(("T4: note persists", check_notes_persist(conn, args.note)))
        results.append(("T4: task persists", check_tasks_persist(conn, args.task)))
        results.append(("T4: kind separation", check_kind_separation(conn)))
        results.append(("T4: file integrity", check_file_integrity(conn, paths)))
        results.append(("T5: encryption flags", check_encryption_flags(conn, paths)))
        conn.close()
    except Exception as e:
        print(f"\n✗ DB error: {e}")
        sys.exit(1)

    print("\n" + "=" * 60)
    passed = sum(1 for _, r in results if r)
    total = len(results)
    for name, r in results:
        print(f"  {'✓' if r else '✗'} {name}")
    print(f"\n{passed}/{total} checks passed")
    if passed == total:
        print("✓ P0 TAIL QA PASSED — ready for Phase 0 sign-off")
        sys.exit(0)
    else:
        print("✗ P0 TAIL QA FAILED — fix issues before sign-off")
        sys.exit(1)

if __name__ == "__main__":
    main()
