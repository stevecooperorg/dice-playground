#!/usr/bin/env python3
"""Validate the learning inventory and add manifest-driven static navigation.

Run after rendering and before enhance-static-site. No third-party dependencies.
"""
import argparse
import html
import json
import shutil
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
        if page["status"] not in {"pilot", "legacy", "source pending"}:
            raise ValueError(f"unknown review status: {page['status']}")
        prerequisites = page.get("prerequisites", [])
        if not isinstance(prerequisites, list) or any(p not in ids for p in prerequisites):
            raise ValueError(f"prerequisites must name earlier content IDs: {page['id']}")
        ids.add(page["id"])
        paths.add(path)
        source = (ROOT / "docs" / path).read_text()
        page["title"] = next(line[2:] for line in source.splitlines() if line.startswith("# "))
        # All published learning pages obey the interim placement contract.
        # Source-pending game attribution does not weaken execution checks.
        if page["status"] in {"pilot", "source pending"}:
            fences = re.findall(r"^```dice\s*$", source, re.MULTILINE)
            if len(fences) != 1:
                raise ValueError(f"pilot must have exactly one executable fence: {path}")
            if "```text" in source:
                raise ValueError(f"pilot must not maintain display-only code: {path}")
    actual = {str(p.relative_to(ROOT / "docs")) for folder in ("tutorial", "cookbook")
              for p in (ROOT / "docs" / folder).glob("*.dice")}
    if paths != actual:
        raise ValueError(f"manifest/corpus mismatch: {paths ^ actual}")
    for alias in manifest.get("aliases", []):
        if alias["id"] in ids or alias["path"] in paths:
            raise ValueError(f"alias collides with existing identity: {alias}")
        if not re.fullmatch(r"(tutorial|cookbook)/[a-z0-9-]+\.dice", alias["path"]):
            raise ValueError(f"invalid alias path: {alias['path']}")
        if alias["target"] not in {p["id"] for p in pages}:
            raise ValueError(f"alias must target canonical content: {alias}")
        ids.add(alias["id"])
        paths.add(alias["path"])
    return pages


def link(page, prefix=""):
    slug = Path(page["path"]).with_suffix(".html").name
    return f'<a href="{prefix}{slug}">{html.escape(page["title"])}</a>'


def publish(dist):
    pages = inventory()
    by_id = {page["id"]: page for page in pages}
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
            if page.get("prerequisites"):
                links = [f'<a href="../{Path(by_id[p]["path"]).with_suffix(".html")}">{html.escape(p)}</a>'
                         for p in page["prerequisites"]]
                nav.append('<p>Prerequisites: ' + ', '.join(links) + '</p>')
            if page.get("rules_review"):
                nav.append('<p>Rules review: ' + html.escape(page["rules_review"]) + '</p>')
            path.write_text(body.replace("</main>", "\n".join(nav) + "\n</main>"))
        groups = ("part",) if section == "tutorial" else ("game", "mechanic")
        content = [f"<h1>{section.title()}</h1>",
                   "<p>Rewritten lessons and recipes are labelled pilot until human and browser review. "
                   "Old URLs have explicit migration pages; source pending is not rules certification.</p>"]
        for group in groups:
            content.append(f"<h2>By {group}</h2>")
            for value in dict.fromkeys(p[group] for p in selected):
                content.append(f"<h3>{html.escape(value)}</h3><ul>")
                for page in selected:
                    if page[group] == value:
                        detail = page.get("objective", page.get("scope", ""))
                        if page.get("difficulty"):
                            detail += " · " + page["difficulty"]
                        content.append(f'<li>{link(page)} — {html.escape(detail)} ({html.escape(page["status"])})</li>')
                content.append("</ul>")
        css = "tutorial.css" if section == "tutorial" else "../tutorial/tutorial.css"
        shell = f'<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>{section.title()}</title><link rel="stylesheet" href="{css}"></head><body><header><a href="../docs/index.html">User guide</a> · <a href="../tutorial/index.html">Tutorial</a> · <a href="../cookbook/index.html">Cookbook</a> · <a href="/">Playground</a></header><main>'
        (dist / section / "index.html").write_text(shell + "\n".join(content) + "</main></body></html>\n")
    manifest = json.loads((ROOT / "docs/learning-content.json").read_text())
    for alias in manifest.get("aliases", []):
        target = by_id[alias["target"]]
        href = "../" + str(Path(target["path"]).with_suffix(".html"))
        # An explicit migration page, never a silently reused lesson number.
        page = dist / Path(alias["path"]).with_suffix(".html")
        page.write_text('<!doctype html><html lang="en"><head><meta charset="utf-8">'
                        '<title>Lesson moved</title><link rel="stylesheet" href="../tutorial/tutorial.css">'
                        '</head><body><main><h1>Lesson moved</h1><p>The old lesson '
                        + html.escape(alias["id"]) + ' has been rewritten. Continue to '
                        + f'<a href="{href}">{html.escape(target["title"])}</a>.'
                        + '</p></main></body></html>\n')
        shutil.copyfile(ROOT / "docs" / target["path"], dist / alias["path"])


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("dist", nargs="?", type=Path)
    args = parser.parse_args()
    if args.dist:
        publish(args.dist)
    else:
        inventory()
