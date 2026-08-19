#!/usr/bin/env python3
"""Render Homebrew files and prune lists from the Linux catalog + a profile.

  linux-tools.py brewfile <profile> [catalog.json] [profile.json] [overlay.json]
  linux-tools.py prune    <profile> [catalog.json] [profile.json] [overlay.json]

Profile files are ID lists (profiles/linux/<id>.json). Overlay, if present:
  { "tools": ["id", ...] }  replaces the profile ID list
"""
from __future__ import annotations

import json
import sys
from pathlib import Path


def load(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def profile_ids_from_doc(doc: dict) -> list[str]:
    tools = doc.get("tools", [])
    out: list[str] = []
    for t in tools:
        if isinstance(t, str):
            if t and t not in out:
                out.append(t)
        elif isinstance(t, dict) and t.get("id"):
            i = str(t["id"])
            if i not in out:
                out.append(i)
    return out


def resolve_ids(profile: str, catalog: dict, profile_doc: dict | None, overlay: dict | None) -> list[str]:
    if overlay and isinstance(overlay.get("tools"), list):
        tools = overlay["tools"]
        if tools and all(isinstance(t, str) for t in tools):
            return profile_ids_from_doc(overlay)
    if profile_doc:
        ids = profile_ids_from_doc(profile_doc)
        if ids:
            return ids
    # Last resort: old catalog with home/work booleans + layer=base.
    out: list[str] = []
    for t in catalog.get("tools", []):
        tid = t.get("id")
        if not tid:
            continue
        if t.get("layer") == "base" or bool(t.get(profile, False)):
            out.append(tid)
    return out


def brewfile(catalog: dict, ids: list[str], profile: str) -> str:
    wanted = set(ids)
    lines = [
        f"# Generated for profile={profile} from profiles/linux.json. Do not edit.",
        "",
    ]
    for t in catalog.get("tools", []):
        tid = t.get("id")
        if tid not in wanted:
            continue
        pkg = t.get("pkg") or tid
        kind = t.get("kind", "brew")
        if kind == "cask":
            lines.append(f'cask "{pkg}"')
        else:
            lines.append(f'brew "{pkg}"')
    lines.append("")
    return "\n".join(lines)


def prune_pkgs(catalog: dict, ids: list[str]) -> list[tuple[str, str]]:
    wanted = set(ids)
    out = []
    for t in catalog.get("tools", []):
        tid = t.get("id")
        if not tid or tid in wanted:
            continue
        out.append((t.get("kind", "brew"), t.get("pkg") or tid))
    return out


def main() -> int:
    if len(sys.argv) < 3:
        print(__doc__.strip(), file=sys.stderr)
        return 2
    cmd, profile = sys.argv[1], sys.argv[2]
    if not profile or profile.startswith("-"):
        print(f"profile must be a name, not {profile!r}", file=sys.stderr)
        return 2
    root = Path(__file__).resolve().parent.parent
    catalog_path = Path(sys.argv[3]) if len(sys.argv) > 3 else root / "profiles" / "linux.json"
    profile_path = Path(sys.argv[4]) if len(sys.argv) > 4 else root / "profiles" / "linux" / f"{profile}.json"
    overlay_path = Path(sys.argv[5]) if len(sys.argv) > 5 else None
    catalog = load(catalog_path)
    profile_doc = load(profile_path) if profile_path.is_file() else None
    overlay = load(overlay_path) if overlay_path and overlay_path.is_file() else None
    ids = resolve_ids(profile, catalog, profile_doc, overlay)
    if not ids and cmd in ("brewfile", "prune"):
        print(f"no tools for profile {profile} (missing {profile_path})", file=sys.stderr)
        return 1
    if cmd == "brewfile":
        sys.stdout.write(brewfile(catalog, ids, profile))
        return 0
    if cmd == "prune":
        for kind, pkg in prune_pkgs(catalog, ids):
            print(f"{kind}\t{pkg}")
        return 0
    print(f"unknown command {cmd}", file=sys.stderr)
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
