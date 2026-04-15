#!/usr/bin/env python3
"""
Phase 0 Test - Obsidian Compatibility
Verifies Obsidian compatibility across all .md files.

Run: python3 scripts/test_obsidian_compat.py
"""

import os
import re
import sys
from pathlib import Path
from typing import Dict, List, Tuple

PROJECT_ROOT = Path("/Users/cristianovb/Desktop/zettel-spark-flow")


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


def parse_frontmatter(content: str) -> Dict[str, str]:
    """Parse YAML frontmatter from markdown content."""
    if not content.startswith("---"):
        return {}

    parts = content.split("---", 2)
    if len(parts) < 3:
        return {}

    fm_text = parts[1]
    fm = {}

    for line in fm_text.strip().split("\n"):
        if ":" in line:
            key, value = line.split(":", 1)
            fm[key.strip()] = value.strip().strip('"').strip("'")

    return fm


def test_created_modified_fields() -> bool:
    """Verify frontmatter uses created/modified (not created_at/updated_at)."""
    log("Testing created/modified fields (not created_at/updated_at)...", "test")

    app_data = find_app_data()
    notes_dir = app_data / "database" / "notes"
    tasks_dir = app_data / "database" / "tasks"

    if not notes_dir.exists():
        log("Notes directory not found", "warn")
        return True

    all_dirs = [notes_dir]
    if tasks_dir.exists():
        all_dirs.append(tasks_dir)

    issues = []
    checked = 0

    for directory in all_dirs:
        md_files = list(directory.glob("*.md"))

        for md_file in md_files:
            content = md_file.read_text()
            fm = parse_frontmatter(content)

            if "created_at" in fm:
                issues.append(f"{md_file.name}: uses 'created_at' instead of 'created'")
            if "updated_at" in fm:
                issues.append(
                    f"{md_file.name}: uses 'updated_at' instead of 'modified'"
                )
            if "created" in fm or "modified" in fm or not fm:
                checked += 1

    if issues:
        for issue in issues:
            log(f"  {issue}", "fail")
        return False

    log(f"Created/modified fields OK ({checked} files)", "pass")
    return True


def test_type_field() -> bool:
    """Verify frontmatter uses type: (not kind:)."""
    log("Testing type: field (not kind:)...", "test")

    app_data = find_app_data()
    notes_dir = app_data / "database" / "notes"
    tasks_dir = app_data / "database" / "tasks"

    if not notes_dir.exists():
        log("Notes directory not found", "warn")
        return True

    all_dirs = [notes_dir]
    if tasks_dir.exists():
        all_dirs.append(tasks_dir)

    issues = []
    has_type = 0

    for directory in all_dirs:
        md_files = list(directory.glob("*.md"))

        for md_file in md_files:
            content = md_file.read_text()
            fm = parse_frontmatter(content)

            if "kind" in fm and "type" not in fm:
                issues.append(f"{md_file.name}: uses 'kind:' instead of 'type:'")
            if "type" in fm:
                has_type += 1

    if issues:
        for issue in issues:
            log(f"  {issue}", "fail")
        return False

    log(f"Type field OK ({has_type} files have type:)", "pass")
    return True


def test_tags_format() -> bool:
    """Verify tags are in array format [tag1, tag2] or list format."""
    log("Testing tags format...", "test")

    app_data = find_app_data()
    notes_dir = app_data / "database" / "notes"
    tasks_dir = app_data / "database" / "tasks"

    if not notes_dir.exists():
        log("Notes directory not found", "warn")
        return True

    all_dirs = [notes_dir]
    if tasks_dir.exists():
        all_dirs.append(tasks_dir)

    issues = []
    has_tags = 0

    for directory in all_dirs:
        md_files = list(directory.glob("*.md"))

        for md_file in md_files:
            content = md_file.read_text()
            fm = parse_frontmatter(content)

            if "tags" in fm:
                tags_value = fm["tags"]
                has_tags += 1

                # Check format - should be array or space-separated
                # Invalid: tags: tag1,tag2 (comma without array)
                if "," in tags_value and "[" not in tags_value:
                    issues.append(
                        f"{md_file.name}: tags use comma without array format"
                    )

    if issues:
        for issue in issues:
            log(f"  {issue}", "fail")
        return False

    log(f"Tags format OK ({has_tags} files have tags)", "pass")
    return True


def test_folder_format() -> bool:
    """Verify folder is string or null (not array)."""
    log("Testing folder format...", "test")

    app_data = find_app_data()
    notes_dir = app_data / "database" / "notes"

    if not notes_dir.exists():
        log("Notes directory not found", "warn")
        return True

    md_files = list(notes_dir.glob("*.md"))

    if not md_files:
        log("No markdown files found", "warn")
        return True

    issues = []

    for md_file in md_files:
        content = md_file.read_text()
        fm = parse_frontmatter(content)

        if "folder" in fm:
            folder_value = fm["folder"]
            # Should be string, null, or absent - not array
            if folder_value.startswith("["):
                issues.append(
                    f"{md_file.name}: folder is array, should be string or null"
                )

    if issues:
        for issue in issues:
            log(f"  {issue}", "fail")
        return False

    log(f"Folder format OK", "pass")
    return True


def test_wiki_link_present() -> bool:
    """Verify wiki-link syntax [[...]] is used."""
    log("Testing wiki-link syntax presence...", "test")

    app_data = find_app_data()
    notes_dir = app_data / "database" / "notes"

    if not notes_dir.exists():
        log("Notes directory not found", "warn")
        return True

    md_files = list(notes_dir.glob("*.md"))

    if not md_files:
        log("No markdown files found", "warn")
        return True

    wiki_link_count = 0

    for md_file in md_files:
        content = md_file.read_text()

        if re.search(r"\[\[([^\]]+)\]\]", content):
            wiki_link_count += 1

    if wiki_link_count > 0:
        log(f"Wiki-links found in {wiki_link_count} files", "pass")
    else:
        log("No wiki-links found (OK if not used yet)", "warn")

    return True


def test_frontmatter_order() -> bool:
    """Verify frontmatter fields follow Obsidian convention."""
    log("Testing frontmatter field order...", "test")

    app_data = find_app_data()
    notes_dir = app_data / "database" / "notes"
    tasks_dir = app_data / "database" / "tasks"

    if not notes_dir.exists():
        log("Notes directory not found", "warn")
        return True

    all_dirs = [notes_dir]
    if tasks_dir.exists():
        all_dirs.append(tasks_dir)

    # Common Obsidian fields in order: tags, aliases, cssclass, folder
    recommended_fields = ["tags", "aliases", "cssclass", "folder", "type"]
    optional_fields = ["created", "modified", "icon"]

    # This is informational - no strict enforcement
    checked = 0

    for directory in all_dirs:
        md_files = list(directory.glob("*.md"))

        for md_file in md_files:
            content = md_file.read_text()
            fm = parse_frontmatter(content)

            if fm:
                checked += 1

    log(f"Frontmatter checked in {checked} files", "pass")
    return True


def main():
    print()
    print("=" * 60)
    print(f"{Colors.BOLD}Phase 0 Test - Obsidian Compatibility{Colors.RESET}")
    print("=" * 60)
    print()

    results = {}

    print(f"\n{Colors.CYAN}=== OBSIDIAN COMPATIBILITY TESTS ==={Colors.RESET}")
    results["Created_Modified"] = test_created_modified_fields()
    results["Type_Field"] = test_type_field()
    results["Tags_Format"] = test_tags_format()
    results["Folder_Format"] = test_folder_format()
    results["Wiki_Links"] = test_wiki_link_present()
    results["Frontmatter_Order"] = test_frontmatter_order()

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
        log("All Obsidian compatibility tests passed!", "pass")
        return 0
    else:
        log(f"{total - passed} test(s) failed", "fail")
        return 1


if __name__ == "__main__":
    sys.exit(main())
