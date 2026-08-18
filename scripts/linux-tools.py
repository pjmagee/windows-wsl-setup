#!/usr/bin/env python3
"""Render Homebrew files and prune lists from profiles/tools.json.

  linux-tools.py brewfile home|work [catalog.json] [overlay.json]
  linux-tools.py prune    home|work [catalog.json] [overlay.json]
"""
from __future__ import annotations

import json
import sys
from pathlib import Path


def load(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def merge(catalog: dict, overlay: dict | None) -> list[dict]:
    tools = [dict(t) for t in catalog.get("tools", [])]
    if not overlay:
        return tools
    by_id = {t["id"]: t for t in overlay.get("tools", []) if "id" in t}
    out = []
    for t in tools:
        o = by_id.get(t["id"])
        if o is None:
            out.append(t)
            continue
        m = dict(t)
        if "home" in o:
            m["home"] = bool(o["home"])
        if "work" in o:
            m["work"] = bool(o["work"])
        if "layer" in o:
            m["layer"] = o["layer"]
        out.append(m)
    return out


def selected(tool: dict, profile: str) -> bool:
    if tool.get("layer") == "base":
        return True
    return bool(tool.get(profile, False))


def brewfile(tools: list[dict], profile: str) -> str:
    lines = [
        f"# Generated for profile={profile} from profiles/tools.json. Do not edit.",
        "",
    ]
    for t in tools:
        if not selected(t, profile):
            continue
        pkg = t.get("pkg") or t["id"]
        kind = t.get("kind", "brew")
        if kind == "cask":
            lines.append(f'cask "{pkg}"')
        else:
            lines.append(f'brew "{pkg}"')
    lines.append("")
    return "\n".join(lines)


def prune_pkgs(tools: list[dict], profile: str) -> list[tuple[str, str]]:
    """Packages to uninstall for this profile: extras not ticked for it."""
    out = []
    for t in tools:
        if t.get("layer") == "base":
            continue
        if selected(t, profile):
            continue
        out.append((t.get("kind", "brew"), t.get("pkg") or t["id"]))
    return out


def main() -> int:
    if len(sys.argv) < 3:
        print(__doc__.strip(), file=sys.stderr)
        return 2
    cmd, profile = sys.argv[1], sys.argv[2]
    if profile not in ("home", "work"):
        print(f"profile must be home or work, not {profile}", file=sys.stderr)
        return 2
    root = Path(__file__).resolve().parent.parent
    catalog_path = Path(sys.argv[3]) if len(sys.argv) > 3 else root / "profiles" / "tools.json"
    overlay_path = Path(sys.argv[4]) if len(sys.argv) > 4 else None
    catalog = load(catalog_path)
    overlay = load(overlay_path) if overlay_path and overlay_path.is_file() else None
    tools = merge(catalog, overlay)
    if cmd == "brewfile":
        sys.stdout.write(brewfile(tools, profile))
        return 0
    if cmd == "prune":
        for kind, pkg in prune_pkgs(tools, profile):
            print(f"{kind}\t{pkg}")
        return 0
    print(f"unknown command {cmd}", file=sys.stderr)
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
