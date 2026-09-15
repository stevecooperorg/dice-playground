#!/usr/bin/env python3
"""Validate the learning inventory and add manifest-driven static navigation.

Run after rendering and before enhance-static-site. No third-party dependencies.
"""
import argparse
import html
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parent.parent


def inventory():
    manifest = json.loads((ROOT / "docs/learning-content.json").read_text())
    if manifest["version"] != 1:
        raise ValueError("unsupported learning manifest version")
    pages = manifest["pages"]
    ids, paths = set(), set()
    for page in pages:
        path = page["path"]
        if page["id"] in ids or path in paths:
            raise ValueError(f"duplicate content identity: {page}")
        if not re.fullmatch(r"(tutorial|cookbook)/[a-z0-9-]+\.dice", path):
            raise ValueError(f"invalid content path: {path}")
        ids.add(page["id"])
        paths.add(path)
        source = (ROOT / "docs" / path).read_text()
        page["title"] = next(line[2:] for line in source.splitlines() if line.startswith("# "))
        # Interim placement contract: pilot pages have one executable block.
        # No static output-call counting can misattribute loop/helper outputs.
        if page["status"] == "pilot":
            fences = re.findall(r"^```dice\s*$", source, re.MULTILINE)
            if len(fences) != 1:
                raise ValueError(f"pilot must have exactly one executable fence: {path}")
            if "```text" in source:
                raise ValueError(f"pilot must not maintain display-only code: {path}")
    actual = {str(p.relative_to(ROOT / "docs")) for folder in ("tutorial", "cookbook")
              for p in (ROOT / "docs" / folder).glob("*.dice")}
    if paths != actual:
        raise ValueError(f"manifest/corpus mismatch: {paths ^ actual}")
    return pages


def link(page, prefix=""):
    slug = Path(page["path"]).with_suffix(".html").name
    return f'<a href="{prefix}{slug}">{html.escape(page["title"])}</a>'


def publish(dist):
    pages = inventory()
    for section in ("tutorial", "cookbook"):
        selected = [p for p in pages if p["path"].startswith(section + "/")]
        for i, page in enumerate(selected):
            path = dist / Path(page["path"]).with_suffix(".html")
            body = path.read_text()
            nav = ['<nav aria-label="Learning navigation">']
            if section == "tutorial":
                if i:
                    nav.append("Previous: " + link(selected[i - 1]) + " · ")
                if i + 1 < len(selected):
                    nav.append("Next: " + link(selected[i + 1]) + " · ")
            nav.append('<a href="index.html">Index</a></nav>')
            nav.append(f'<p>Content ID: {html.escape(page["id"])} · Review status: {html.escape(page["status"])}</p>')
            path.write_text(body.replace("</main>", "\n".join(nav) + "\n</main>"))
        groups = ("part",) if section == "tutorial" else ("game", "mechanic")
        content = [f"<h1>{section.title()}</h1>",
                   "<p>The opening four lessons and Cairn recipe are rewrite pilots. "
                   "Legacy entries are retained while review continues; source pending is not rules certification.</p>"]
        for group in groups:
            content.append(f"<h2>By {group}</h2>")
            for value in dict.fromkeys(p[group] for p in selected):
                content.append(f"<h3>{html.escape(value)}</h3><ul>")
                for page in selected:
                    if page[group] == value:
                        detail = page.get("objective", page.get("scope", ""))
                        content.append(f'<li>{link(page)} — {html.escape(detail)} ({html.escape(page["status"])})</li>')
                content.append("</ul>")
        css = "tutorial.css" if section == "tutorial" else "../tutorial/tutorial.css"
        shell = f'<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>{section.title()}</title><link rel="stylesheet" href="{css}"></head><body><header><a href="../docs/index.html">User guide</a> · <a href="../tutorial/index.html">Tutorial</a> · <a href="../cookbook/index.html">Cookbook</a> · <a href="/">Playground</a></header><main>'
        (dist / section / "index.html").write_text(shell + "\n".join(content) + "</main></body></html>\n")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("dist", nargs="?", type=Path)
    args = parser.parse_args()
    if args.dist:
        publish(args.dist)
    else:
        inventory()
