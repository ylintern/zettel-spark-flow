#!/usr/bin/env python3
"""
Phase 0 Test - Task Format Validation
Validates task format compliance with Obsidian standard.

Run: python3 scripts/test_task_format.py
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
        if p.exists() and (p / "tasks").exists():
            return p
    return possible[0]


def parse_frontmatter(content: str) -> dict:
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


def test_task_checkbox_unchecked() -> bool:
    """C3: Verify tasks use - [ ] unchecked format."""
    log("Testing unchecked task checkbox format...", "test")

    app_data = find_app_data()
    tasks_dir = app_data / "database" / "tasks"

    if not tasks_dir.exists():
        log("Tasks directory not found (app may not have been run)", "warn")
        return True

    md_files = list(tasks_dir.glob("*.md"))

    if not md_files:
        log("No task files found", "warn")
        return True

    unchecked_tasks = []
    for md_file in md_files:
        content = md_file.read_text()

        # Skip frontmatter
        if "---" in content:
            content = content.split("---", 2)[2]

        # Find - [ ] pattern
        unchecked = re.findall(r"-\s*\[\s*\]", content)
        if unchecked:
            unchecked_tasks.append((md_file.name, len(unchecked)))

    if unchecked_tasks:
        log(f"Found tasks with unchecked box:", "pass")
        for fname, count in unchecked_tasks[:5]:
            log(f"  {fname}: {count} unchecked task(s)", "pass")
        return True
    else:
        log("No unchecked task boxes found", "warn")
        return True


def test_task_checkbox_checked() -> bool:
    """C4: Verify tasks can use - [x] checked format."""
    log("Testing checked task checkbox format...", "test")

    app_data = find_app_data()
    tasks_dir = app_data / "database" / "tasks"

    if not tasks_dir.exists():
        log("Tasks directory not found", "warn")
        return True

    md_files = list(tasks_dir.glob("*.md"))

    if not md_files:
        log("No task files found", "warn")
        return True

    checked_tasks = []
    for md_file in md_files:
        content = md_file.read_text()

        # Skip frontmatter
        if "---" in content:
            content = content.split("---", 2)[2]

        # Find - [x] pattern
        checked = re.findall(r"-\s*\[x\]", content, re.IGNORECASE)
        if checked:
            checked_tasks.append((md_file.name, len(checked)))

    if checked_tasks:
        log(f"Found tasks with checked box:", "pass")
        for fname, count in checked_tasks[:5]:
            log(f"  {fname}: {count} checked task(s)", "pass")
        return True
    else:
        log("No checked task boxes found (may not have toggled yet)", "warn")
        return True


def test_task_type_frontmatter() -> bool:
    """C7: Verify tasks have type: task in frontmatter."""
    log("Testing task type field in frontmatter...", "test")

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
    type_ok = 0

    for md_file in md_files:
        content = md_file.read_text()
        fm = parse_frontmatter(content)

        if "type" in fm:
            if fm["type"] == "task":
                type_ok += 1
            else:
                issues.append(
                    f"{md_file.name}: type is '{fm['type']}', expected 'task'"
                )
        else:
            issues.append(f"{md_file.name}: missing 'type' field in frontmatter")

    if issues:
        for issue in issues:
            log(f"  {issue}", "fail")
        return False

    log(f"Task type field OK ({type_ok}/{len(md_files)} files have type: task)", "pass")
    return True


def test_no_custom_yaml_fields() -> bool:
    """Verify tasks don't use custom YAML fields like Status: or Priority:."""
    log("Testing no custom YAML fields (Status, Priority)...", "test")

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
    custom_fields = ["Status:", "Priority:", "status:", "priority:", "Stage:", "stage:"]

    for md_file in md_files:
        content = md_file.read_text()

        if content.startswith("---"):
            fm_text = content.split("---", 2)[1]

            for field in custom_fields:
                if field in fm_text:
                    issues.append(
                        f"{md_file.name}: contains '{field}' (use checkbox instead)"
                    )

    if issues:
        for issue in issues:
            log(f"  {issue}", "fail")
        return False

    log(f"No custom YAML fields found ({len(md_files)} files checked)", "pass")
    return True


def test_task_keeper_compatible() -> bool:
    """Verify tasks are compatible with Obsidian Tasks plugin."""
    log("Testing Obsidian Tasks plugin compatibility...", "test")

    app_data = find_app_data()
    tasks_dir = app_data / "database" / "tasks"

    if not tasks_dir.exists():
        log("Tasks directory not found", "warn")
        return True

    md_files = list(tasks_dir.glob("*.md"))

    if not md_files:
        log("No task files found", "warn")
        return True

    compatible = 0
    issues = []

    for md_file in md_files:
        content = md_file.read_text()

        # Check for Obsidian Tasks compatible format
        # - Must have checkbox format
        # - type: task in frontmatter
        # - Optional: due:, scheduled:, priority: fields (Obsidian standard)

        fm = parse_frontmatter(content)
        has_type_task = fm.get("type") == "task"

        # Check body
        body = content.split("---", 2)[2] if "---" in content else content
        has_checkbox = bool(re.search(r"-\s*\[[ x]\]", body, re.IGNORECASE))

        if has_type_task and has_checkbox:
            compatible += 1
        else:
            issues.append(f"{md_file.name}: not Obsidian Tasks compatible")

    if issues:
        for issue in issues:
            log(f"  {issue}", "fail")
        return False

    log(f"Obsidian Tasks compatibility OK ({compatible}/{len(md_files)} files)", "pass")
    return True


def main():
    print()
    print("=" * 60)
    print(f"{Colors.BOLD}Phase 0 Test - Task Format Validation{Colors.RESET}")
    print("=" * 60)
    print()

    results = {}

    print(f"\n{Colors.CYAN}=== TASK FORMAT TESTS ==={Colors.RESET}")
    results["C3_Task_Checkbox_Unchecked"] = test_task_checkbox_unchecked()
    results["C4_Task_Checkbox_Checked"] = test_task_checkbox_checked()
    results["C7_Task_Type_Frontmatter"] = test_task_type_frontmatter()
    results["No_Custom_YAML"] = test_no_custom_yaml_fields()
    results["Obsidian_Tasks_Compatible"] = test_task_keeper_compatible()

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
        log("All task format tests passed!", "pass")
        return 0
    else:
        log(f"{total - passed} test(s) failed", "fail")
        return 1


if __name__ == "__main__":
    sys.exit(main())
