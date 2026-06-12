#!/usr/bin/env python3
"""Merge docs/cookbook/*.md prose with examples/cookbook/*.dice executable fences."""

from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOCS = ROOT / "docs" / "cookbook"
EXAMPLES = ROOT / "examples" / "cookbook"

# slug matches between md and examples (1:1 names)
RECIPE_SLUGS = [
    "ability-scores-4d6dl1",
    "blades-in-the-dark",
    "brindlewood-bay-theorize",
    "cairn-blood-elk",
    "count-high-faces",
    "exploding-dice",
    "fireball-half-damage",
    "fudge-4df",
    "rolemaster-open-ended",
    "the-pool",
]


def strip_frontmatter(text: str) -> str:
    if text.startswith("---"):
        end = text.find("\n---", 3)
        if end != -1:
            return text[end + 4 :].lstrip("\n")
    return text


def rewrite_links(body: str) -> str:
    body = re.sub(r"\]\(([a-z0-9-]+)\.md\)", r"](\1.html)", body)
    body = body.replace("](README.md)", "](index.html)")
    body = body.replace(
        "copy the script below into the **editor** and click **Run**",
        "open this recipe in the playground and click **Run**",
    )
    body = body.replace("Under **Output**", "In the **report**")
    return body


def example_snippet_for_text_fence(script: str) -> str:
    lines = [ln for ln in script.strip().splitlines() if ln.strip() and not ln.strip().startswith("#")]
    if len(lines) <= 4:
        return "\n".join(lines)
    return "\n".join(lines[:3])


def build_literate(md_path: Path, example_path: Path) -> str:
    body = rewrite_links(strip_frontmatter(md_path.read_text(encoding="utf-8")))
    script = example_path.read_text(encoding="utf-8").rstrip() + "\n"
    snippet = example_snippet_for_text_fence(script)
    body = re.sub(r"\n## Cookbook\n[\s\S]*\Z", "", body).rstrip() + "\n"
    if "```text" not in body and "## The script" in body:
        body += f"\n## Runnable script\n\n```text\n{snippet}\n```\n"
    body += f"\n```dice\n{script}```\n"
    return body


def main() -> None:
    for slug in RECIPE_SLUGS:
        md = DOCS / f"{slug}.md"
        ex = EXAMPLES / f"{slug}.dice"
        out = DOCS / f"{slug}.dice"
        if not md.is_file() or not ex.is_file():
            raise SystemExit(f"missing inputs for {slug}")
        out.write_text(build_literate(md, ex), encoding="utf-8")
        print(f"wrote {out.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
