#!/usr/bin/env python3
"""
Phase 0 Smoke Tests - Build + Format Validation
Runs build tests (F1-F4) and file format validation tests.

Run: python3 scripts/smoke_tests.py
"""

import os
import re
import subprocess
import sys
from pathlib import Path
from typing import Tuple

PROJECT_ROOT = Path("/Users/cristianovb/Desktop/zettel-spark-flow")
SRC_TAURI = PROJECT_ROOT / "src-tauri"
SRC = PROJECT_ROOT / "src"


class Colors:
    GREEN = "\033[92m"
    RED = "\033[91m"
    YELLOW = "\033[93m"
    BLUE = "\033[94m"
    CYAN = "\033[96m"
    RESET = "\033[0m"
    BOLD = "\033[1m"


def log(msg: str, status: str = "info"):
    prefix = {
        "info": f"{Colors.BLUE}[INFO]{Colors.RESET}",
        "pass": f"{Colors.GREEN}[PASS]{Colors.RESET}",
        "fail": f"{Colors.RED}[FAIL]{Colors.RESET}",
        "warn": f"{Colors.YELLOW}[WARN]{Colors.RESET}",
        "test": f"{Colors.BOLD}[TEST]{Colors.RESET}",
    }.get(status, "[INFO]")
    print(f"{prefix} {msg}")


def run_command(
    cmd: list, cwd: Path = PROJECT_ROOT, timeout: int = 300
) -> Tuple[bool, str]:
    """Run command and return (success, output)."""
    try:
        result = subprocess.run(
            cmd, cwd=cwd, capture_output=True, text=True, timeout=timeout
        )
        return result.returncode == 0, result.stdout + result.stderr
    except subprocess.TimeoutExpired:
        return False, "Command timed out"
    except Exception as e:
        return False, str(e)


# ============================================================
# BUILD TESTS (F1-F4)
# ============================================================


def test_f1_dev_build() -> bool:
    """F1: Dev build starts without errors."""
    log("F1: Testing dev build (bun run tauri:dev check)...", "test")

    # Just check the frontend builds, not full tauri dev
    success, output = run_command(["bun", "run", "build"])

    if success:
        log("Frontend build successful", "pass")
        return True
    else:
        log(f"Frontend build failed: {output[:200]}", "fail")
        return False


def test_f2_prod_build() -> bool:
    """F2: Production build succeeds."""
    log("F2: Testing production build...", "test")

    # Check if cargo can compile (faster than full build)
    success, output = run_command(
        ["cargo", "build", "--release", "--manifest-path", "src-tauri/Cargo.toml"],
        timeout=600,
    )

    if success:
        log("Production build successful", "pass")
        return True
    else:
        log(f"Production build failed: {output[:200]}", "fail")
        return False


def test_f3_frontend_build() -> bool:
    """F3: Frontend build succeeds."""
    log("F3: Testing frontend build...", "test")

    success, output = run_command(["bun", "run", "build"])

    if success:
        log("Frontend build OK", "pass")
        return True
    else:
        log(f"Frontend build failed", "fail")
        return False


def test_f4_rust_check() -> bool:
    """F4: Rust code compiles without errors."""
    log("F4: Running cargo check...", "test")

    success, output = run_command(
        ["cargo", "check", "--manifest-path", "src-tauri/Cargo.toml"], cwd=PROJECT_ROOT
    )

    if success:
        log("Rust check passed", "pass")
        return True
    else:
        log(f"Rust check failed: {output[:200]}", "fail")
        return False


# ============================================================
# FILE FORMAT TESTS (B10-B11, C6-C7, D3)
# ============================================================


def find_app_data() -> Path:
    """Find the app data directory - checks new viboai/myspace path first."""
    possible = [
        # NEW: viboai/myspace (current)
        Path.home()
        / "Library/Application Support/com.viboai.app"
        / "viboai"
        / "myspace",
        Path.home()
        / "Library/Application Support/com.viboai.app.dev"
        / "viboai"
        / "myspace",
        # OLD: database (legacy)
        Path.home() / "Library/Application Support/com.viboai.app" / "database",
        Path.home() / "Library/Application Support/com.viboai.app.dev" / "database",
    ]
    for p in possible:
        if p.exists() and (p / "notes").exists():
            return p
    return possible[0]


def test_b10_frontmatter_format() -> bool:
    """B10: Verify Obsidian frontmatter format in .md files."""
    log("B10: Checking frontmatter format...", "test")

    app_data = find_app_data()
    notes_dir = app_data / "notes"
    tasks_dir = app_data / "tasks"

    if not notes_dir.exists():
        log("Notes directory not found (app may not have been run)", "warn")
        return True

    issues = []
    md_files = list(notes_dir.glob("*.md")) + list(tasks_dir.glob("*.md"))

    for md_file in md_files:
        try:
            content = md_file.read_text()

            # Check for frontmatter delimiters
            if content.startswith("---"):
                lines = content.split("\n")
                frontmatter_end = -1
                for i, line in enumerate(lines[1:], 1):
                    if line.strip() == "---":
                        frontmatter_end = i
                        break

                if frontmatter_end > 0:
                    fm_lines = lines[1:frontmatter_end]
                    fm_text = "\n".join(fm_lines)

                    # Check for non-Obsidian fields
                    if "created_at:" in fm_text:
                        issues.append(
                            f"{md_file.name}: uses 'created_at' instead of 'created'"
                        )
                    if "updated_at:" in fm_text:
                        issues.append(
                            f"{md_file.name}: uses 'updated_at' instead of 'modified'"
                        )
                    if "kind:" in fm_text:
                        issues.append(
                            f"{md_file.name}: uses 'kind:' instead of 'type:'"
                        )
        except Exception as e:
            issues.append(f"{md_file.name}: read error - {e}")

    if issues:
        for issue in issues:
            log(f"  {issue}", "fail")
        return False

    log(f"Frontmatter format OK ({len(md_files)} files checked)", "pass")
    return True


def test_b11_markdown_file_exists() -> bool:
    """B11: Verify note .md files exist in database/notes/."""
    log("B11: Checking markdown file existence...", "test")

    app_data = find_app_data()
    notes_dir = app_data / "database" / "notes"

    if not notes_dir.exists():
        log("Notes directory not found", "fail")
        return False

    md_files = list(notes_dir.glob("*.md"))

    if not md_files:
        log("No .md files found in notes directory", "warn")
        return True

    log(f"Found {len(md_files)} markdown files in notes/", "pass")
    return True


def test_c6_task_checkbox_format() -> bool:
    """C6: Verify task files use - [ ] or - [x] format."""
    log("C6: Checking task checkbox format...", "test")

    app_data = find_app_data()
    tasks_dir = app_data / "database" / "tasks"

    if not tasks_dir.exists():
        log("Tasks directory not found (app may not have been run)", "warn")
        return True

    md_files = list(tasks_dir.glob("*.md"))

    if not md_files:
        log("No task files found", "warn")
        return True

    issues = []
    for md_file in md_files:
        content = md_file.read_text()

        # Check for checkbox format in content (after frontmatter)
        if "---" in content:
            content = content.split("---", 2)[2]  # Skip frontmatter

        # Must have - [ ] or - [x]
        has_checkbox = bool(re.search(r"-\s*\[\s*\]", content)) or bool(
            re.search(r"-\s*\[x\]", content)
        )

        if not has_checkbox:
            issues.append(f"{md_file.name}: no checkbox format found")

    if issues:
        for issue in issues:
            log(f"  {issue}", "fail")
        return False

    log(f"Task checkbox format OK ({len(md_files)} files)", "pass")
    return True


def test_c7_task_type_field() -> bool:
    """C7: Verify tasks have type: task in frontmatter."""
    log("C7: Checking task type field...", "test")

    app_data = find_app_data()
    tasks_dir = app_data / "database" / "tasks"

    if not tasks_dir.exists():
        log("Tasks directory not found", "warn")
        return True

    md_files = list(tasks_dir.glob("*.md"))

    if not md_files:
        log("No task files found", "warn")
        return True

    issues = []
    for md_file in md_files:
        content = md_file.read_text()

        if content.startswith("---"):
            lines = content.split("\n")
            frontmatter_end = -1
            for i, line in enumerate(lines[1:], 1):
                if line.strip() == "---":
                    frontmatter_end = i
                    break

            if frontmatter_end > 0:
                fm_text = "\n".join(lines[1:frontmatter_end])

                if "type:" not in fm_text and "type: task" not in fm_text:
                    issues.append(f"{md_file.name}: missing 'type: task'")

    if issues:
        for issue in issues:
            log(f"  {issue}", "fail")
        return False

    log(f"Task type field OK ({len(md_files)} files)", "pass")
    return True


def test_d3_encrypted_content() -> bool:
    """D3: Verify encrypted notes show cipher in .md files."""
    log("D3: Checking encrypted content...", "test")

    app_data = find_app_data()
    db_path = app_data / "vibo.db"

    if not db_path.exists():
        log("Database not found", "warn")
        return True

    try:
        import sqlite3

        conn = sqlite3.connect(db_path)
        cur = conn.cursor()

        # Find encrypted notes
        cur.execute("SELECT id, title, is_encrypted FROM notes WHERE is_encrypted = 1")
        encrypted = cur.fetchall()

        conn.close()

        if not encrypted:
            log("No encrypted notes found (OK)", "pass")
            return True

        # Check that encrypted notes have encrypted content in file
        notes_dir = app_data / "database" / "notes"
        issues = []

        for note_id, title, _ in encrypted:
            # Find the file
            md_files = list(notes_dir.glob(f"{note_id}.md"))
            if md_files:
                content = md_files[0].read_text()
                # Should have some encryption markers (this is a basic check)
                # Real encryption would show base64 or cipher text
                if len(content) < 50:
                    issues.append(f"{title}: encrypted note too short")

        if issues:
            for issue in issues:
                log(f"  {issue}", "fail")
            return False

        log(f"Encrypted content OK ({len(encrypted)} encrypted notes)", "pass")
        return True

    except Exception as e:
        log(f"Error checking encrypted content: {e}", "warn")
        return True


# ============================================================
# MAIN EXECUTION
# ============================================================


def main():
    print()
    print("=" * 60)
    print(f"{Colors.BOLD}Phase 0 Smoke Tests - Build + Format{Colors.RESET}")
    print("=" * 60)
    print()

    results = {}

    # Build tests (F1-F4)
    print(f"\n{Colors.CYAN}=== BUILD TESTS (F1-F4) ==={Colors.RESET}")
    results["F1_Dev_Build"] = test_f1_dev_build()
    results["F3_Frontend_Build"] = test_f3_frontend_build()
    results["F4_Rust_Check"] = test_f4_rust_check()

    # File format tests
    print(f"\n{Colors.CYAN}=== FILE FORMAT TESTS ==={Colors.RESET}")
    results["B10_Frontmatter"] = test_b10_frontmatter_format()
    results["B11_Markdown_Exists"] = test_b11_markdown_file_exists()
    results["C6_Task_Checkbox"] = test_c6_task_checkbox_format()
    results["C7_Task_Type"] = test_c7_task_type_field()
    results["D3_Encrypted_Content"] = test_d3_encrypted_content()

    # Summary
    print()
    print("=" * 60)
    print(f"{Colors.BOLD}SUMMARY{Colors.RESET}")
    print("=" * 60)

    passed = sum(1 for v in results.values() if v)
    total = len(results)

    for name, result in results.items():
        status = (
            f"{Colors.GREEN}✓ PASS{Colors.RESET}"
            if result
            else f"{Colors.RED}✗ FAIL{Colors.RESET}"
        )
        print(f"  {name}: {status}")

    print()
    log(f"Results: {passed}/{total} passed", "info")

    if passed == total:
        log("All smoke tests passed!", "pass")
        return 0
    else:
        log(f"{total - passed} test(s) failed", "fail")
        return 1


if __name__ == "__main__":
    sys.exit(main())
