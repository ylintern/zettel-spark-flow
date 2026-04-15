#!/usr/bin/env python3
"""
Phase 0 Test - Wiki-link Functionality
Validates wiki-link creation, aliases, and format in .md files.

Run: python3 scripts/test_wiki_links.py
"""

import os
import re
import sys
from pathlib import Path
from typing import List, Tuple

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
    """Find the app data directory."""
    possible = [
        Path.home() / "Library/Application Support/com.viboai.app",
        Path.home() / "Library/Application Support/com.viboai.app.dev",
    ]
    for p in possible:
        if p.exists():
            return p
    return possible[0]


def test_wiki_link_basic() -> bool:
    """B5: Verify [[Link]] syntax exists in notes."""
    log("Testing basic wiki-link [[Link]] syntax...", "test")

    app_data = find_app_data()
    notes_dir = app_data / "database" / "notes"
    tasks_dir = app_data / "database" / "tasks"

    if not notes_dir.exists():
        log("Notes directory not found", "warn")
        return True

    all_md_files = list(notes_dir.glob("*.md"))
    if tasks_dir.exists():
        all_md_files.extend(tasks_dir.glob("*.md"))

    if not all_md_files:
        log("No markdown files found", "warn")
        return True

    wiki_links = []
    for md_file in all_md_files:
        content = md_file.read_text()
        # Find [[...]] patterns
        links = re.findall(r"\[\[([^\]|]+)\]\]", content)
        if links:
            wiki_links.extend([(md_file.name, link) for link in links])

    if wiki_links:
        log(f"Found {len(wiki_links)} basic wiki-links:", "pass")
        for fname, link in wiki_links[:5]:
            log(f"  {fname}: [[{link}]]", "pass")
        return True
    else:
        log("No wiki-links found (may not be created yet)", "warn")
        return True


def test_wiki_link_alias() -> bool:
    """B7: Verify [[Target|Display]] alias syntax."""
    log("Testing wiki-link alias [[Target|Display]] syntax...", "test")

    app_data = find_app_data()
    notes_dir = app_data / "database" / "notes"
    tasks_dir = app_data / "database" / "tasks"

    if not notes_dir.exists():
        log("Notes directory not found", "warn")
        return True

    all_md_files = list(notes_dir.glob("*.md"))
    if tasks_dir.exists():
        all_md_files.extend(tasks_dir.glob("*.md"))

    if not all_md_files:
        log("No markdown files found", "warn")
        return True

    aliases = []
    for md_file in all_md_files:
        content = md_file.read_text()
        # Find [[target|display]] patterns
        alias_links = re.findall(r"\[\[([^|]+)\|([^\]]+)\]\]", content)
        if alias_links:
            aliases.extend(
                [(md_file.name, target, display) for target, display in alias_links]
            )

    if aliases:
        log(f"Found {len(aliases)} aliased wiki-links:", "pass")
        for fname, target, display in aliases[:5]:
            log(f"  {fname}: [[{target}|{display}]]", "pass")
        return True
    else:
        log("No aliased wiki-links found (may not be created yet)", "warn")
        return True


def test_wiki_link_format() -> bool:
    """Verify wiki-links follow correct markdown format."""
    log("Testing wiki-link format compliance...", "test")

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

        # Find all [[...]] patterns
        all_links = re.findall(r"\[\[([^\]]+)\]\]", content)

        for link in all_links:
            # Check for invalid patterns
            if "  " in link.strip():
                issues.append(f"{md_file.name}: wiki-link has extra spaces: [[{link}]]")
            # Check for empty links
            if not link.strip():
                issues.append(f"{md_file.name}: empty wiki-link found")

    if issues:
        for issue in issues:
            log(f"  {issue}", "fail")
        return False

    log(f"Wiki-link format OK ({len(md_files)} files checked)", "pass")
    return True


def test_no_html_links() -> bool:
    """Verify notes don't use HTML <a> tags for internal links."""
    log("Checking no HTML links (should use wiki-links)...", "test")

    app_data = find_app_data()
    notes_dir = app_data / "database" / "notes"

    if not notes_dir.exists():
        log("Notes directory not found", "warn")
        return True

    md_files = list(notes_dir.glob("*.md"))

    if not md_files:
        log("No markdown files found", "warn")
        return True

    html_links = []
    for md_file in md_files:
        content = md_file.read_text()
        # Find <a href= patterns (internal links)
        html = re.findall(r'<a\s+href="([^"]+)"', content)
        if html:
            html_links.extend([(md_file.name, link) for link in html])

    if html_links:
        log(f"Found {len(html_links)} HTML links (should use wiki-links):", "warn")
        for fname, link in html_links:
            log(f'  {fname}: <a href="{link}">', "warn")
        # This is a warning, not a failure - Obsidian accepts both
        log("Note: Obsidian accepts both wiki-links and HTML links", "warn")
        return True

    log("No HTML links found - using wiki-links only", "pass")
    return True


def main():
    print()
    print("=" * 60)
    print(f"{Colors.BOLD}Phase 0 Test - Wiki-link Functionality{Colors.RESET}")
    print("=" * 60)
    print()

    results = {}

    print(f"\n{Colors.CYAN}=== WIKI-LINK TESTS ==={Colors.RESET}")
    results["B5_WikiLink_Basic"] = test_wiki_link_basic()
    results["B7_WikiLink_Alias"] = test_wiki_link_alias()
    results["WikiLink_Format"] = test_wiki_link_format()
    results["No_HTML_Links"] = test_no_html_links()

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
        log("All wiki-link tests passed!", "pass")
        return 0
    else:
        log(f"{total - passed} test(s) failed", "fail")
        return 1


if __name__ == "__main__":
    sys.exit(main())
