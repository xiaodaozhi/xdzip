#!/usr/bin/env python3
"""Preview or remove stale, unregistered temporary directories under build/."""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import os
from pathlib import Path
import shutil
import subprocess
import sys


ROOT = Path(__file__).resolve().parents[1]
BUILD = ROOT / "build"
PRESERVED_TOP_LEVEL = {
    "desktop",
    "package",
    "review",
    "test-tmp",
    "toolchain",
}
TEMPORARY_PREFIXES = (
    "archive-",
    "association-",
    "bridge-",
    "build-",
    "cab-",
    "ci-",
    "closeout-",
    "desktop-",
    "docs-",
    "fixture-",
    "html-",
    "install",
    "installer-",
    "issue",
    "issues",
    "isolated-",
    "linux-",
    "macos-",
    "main-",
    "native-",
    "optical-",
    "production-",
    "qt-",
    "rar-",
    "release-",
    "settings-",
    "signing",
    "source-",
    "sevenzip-",
    "test-",
    "tmp-",
    "ui-",
    "update-",
    "windows-",
    "work-",
    "worktree",
    "worktrees",
)


def display(path: Path) -> str:
    try:
        return path.resolve().relative_to(ROOT).as_posix()
    except ValueError:
        return str(path)


def worktree_paths() -> set[Path]:
    result = subprocess.run(
        ["git", "worktree", "list", "--porcelain"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=True,
    )
    repo_root = ROOT.resolve()
    paths = set()
    for line in result.stdout.splitlines():
        if line.startswith("worktree "):
            path = Path(line[9:]).resolve()
            if path != repo_root:
                paths.add(path)
    return paths


def is_active_worktree(path: Path, worktrees: set[Path]) -> bool:
    resolved = path.resolve()
    repo_root = ROOT.resolve()
    return any(item != repo_root and
               (resolved == item or resolved.is_relative_to(item) or item.is_relative_to(resolved))
               for item in worktrees)


def latest_mtime(path: Path) -> float | None:
    latest = path.stat().st_mtime
    if not path.is_dir() or path.is_symlink():
        return latest
    for directory, _, files in os.walk(path, followlinks=False):
        current = Path(directory)
        try:
            latest = max(latest, current.stat().st_mtime)
            for name in files:
                entry = current / name
                if not entry.is_symlink():
                    latest = max(latest, entry.stat().st_mtime)
        except OSError:
            return None
    return latest


def is_temporary_name(name: str) -> bool:
    lowered = name.lower()
    return lowered.startswith(TEMPORARY_PREFIXES)


def validate_target(value: Path) -> Path:
    target = value if value.is_absolute() else ROOT / value
    target = target.resolve()
    if target == BUILD or not target.is_relative_to(BUILD):
        raise ValueError("cleanup targets must be below build/")
    top_level = target.relative_to(BUILD).parts[0]
    if top_level in PRESERVED_TOP_LEVEL:
        raise ValueError(f"refusing to clean protected build/{top_level}/")
    return target


def explicit_candidates(paths: list[Path], worktrees: set[Path]) -> list[Path]:
    candidates = []
    for value in paths:
        target = validate_target(value)
        if not target.exists() and not target.is_symlink():
            print(f"missing: {display(target)}")
            continue
        if target.is_symlink() or is_active_worktree(target, worktrees):
            print(f"skipped active or linked path: {display(target)}")
            continue
        candidates.append(target)
    return candidates


def stale_candidates(days: float, worktrees: set[Path]) -> list[tuple[Path, float]]:
    if not BUILD.is_dir():
        return []
    now = datetime.now(timezone.utc).timestamp()
    candidates = []
    for path in BUILD.iterdir():
        if not path.is_dir() or path.is_symlink() or path.name in PRESERVED_TOP_LEVEL:
            continue
        if not is_temporary_name(path.name) or is_active_worktree(path, worktrees):
            continue
        mtime = latest_mtime(path)
        if mtime is None:
            continue
        age = max(0.0, (now - mtime) / 86400.0)
        if age >= days:
            candidates.append((path, age))
    return sorted(candidates, key=lambda item: item[0].name.lower())


def remove(path: Path) -> None:
    if path.is_dir():
        shutil.rmtree(path)
    else:
        path.unlink()


def parser() -> argparse.ArgumentParser:
    value = argparse.ArgumentParser(description=__doc__)
    value.add_argument(
        "--path",
        action="append",
        type=Path,
        default=[],
        help="explicit temporary file or directory below build/; may be repeated",
    )
    value.add_argument(
        "--stale-days",
        type=float,
        help="also select recognized temporary directories older than this many days",
    )
    value.add_argument("--apply", action="store_true", help="remove the listed paths")
    value.add_argument("--yes", action="store_true", help="confirm --apply; required for deletion")
    return value


def main(argv: list[str] | None = None) -> int:
    args = parser().parse_args(argv)
    if not args.path and args.stale_days is None:
        parser().error("provide --path or --stale-days")
    if args.stale_days is not None and args.stale_days < 0:
        parser().error("--stale-days must be non-negative")
    if args.apply and not args.yes:
        parser().error("--apply requires --yes")

    try:
        worktrees = worktree_paths()
        selected = explicit_candidates(args.path, worktrees)
        stale_items = stale_candidates(args.stale_days, worktrees) if args.stale_days is not None else []
        selected.extend(path for path, _ in stale_items if path not in selected)
    except (OSError, subprocess.SubprocessError, ValueError) as error:
        print(str(error), file=sys.stderr)
        return 1

    if not selected:
        print("No eligible build paths.")
        return 0

    stale = dict(stale_items)
    for path in selected:
        suffix = f" ({stale[path]:.1f} days old)" if path in stale else ""
        print(("remove: " if args.apply else "candidate: ") + display(path) + suffix)

    if not args.apply:
        print("Dry run only. Re-run with --apply --yes to remove these paths.")
        return 0
    for path in selected:
        try:
            remove(path)
        except OSError as error:
            print(f"failed to remove {display(path)}: {error}", file=sys.stderr)
            return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
