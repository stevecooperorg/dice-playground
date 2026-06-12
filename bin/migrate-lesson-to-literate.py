#!/usr/bin/env python3
"""One-off helper: merge docs/tutorial/*.md prose with examples/tutorial/*.dice executable fence."""

from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOCS = ROOT / "docs" / "tutorial"
EXAMPLES = ROOT / "examples" / "tutorial"

# docs slug (from .md basename) -> examples filename
EXAMPLE_FOR_SLUG: dict[str, str] = {
    "02-two-dice": "02-two-d6.dice",
    "03-modifiers": "03-modifier-shift.dice",
    "04-success": "04-success-chance.dice",
    "05-dice-notation": "05-dice-notation.dice",
    "06-dice-pools": "06-dice-pools.dice",
    "07-mixed-dice-pools": "07-mixed-dice-pools.dice",
    "08-restrict-faces": "08-restrict-faces.dice",
    "09-pool-success-counts": "09-pool-success-counts.dice",
    "10-tables": "10-table-2d10.dice",
    "11-ordered-outcomes": "11-ordered-outcomes.dice",
    "12-dnd5e-d20-check": "12-dnd5e-d20-check.dice",
    "13-pbta-2d6-move": "13-pbta-2d6-move.dice",
}


def strip_frontmatter(text: str) -> str:
    if text.startswith("---"):
        end = text.find("\n---", 3)
        if end != -1:
            return text[end + 4 :].lstrip("\n")
    return text


def rewrite_links(body: str) -> str:
    body = re.sub(r"\]\(\./([0-9]{2}-[^)]+)\.md\)", r"](\1.html)", body)
    body = re.sub(r"\]\(([0-9]{2}-[^)]+)\.md\)", r"](\1.html)", body)
    body = body.replace(
        "In the playground, copy the script below into the **editor**, then click **Run**",
        "Open this lesson in the playground and click **Run**",
    )
    body = body.replace(
        "Under **Output**, the **text** tab",
        "In the **report**,",
    )
    body = body.replace("Under **Output**, **mean**", "The report **mean**")
    body = body.replace(
        "Under **Output**, ",
        "In the report, ",
    )
    return body


def example_snippet_for_text_fence(script: str) -> str:
    lines = [ln for ln in script.strip().splitlines() if ln.strip() and not ln.strip().startswith("#")]
    if not lines:
        return script.strip()
    if len(lines) <= 3:
        return "\n".join(lines)
    return lines[-1] if lines[-1].startswith("output(") else "\n".join(lines[:2])


def build_literate(md_path: Path, example_path: Path) -> str:
    body = rewrite_links(strip_frontmatter(md_path.read_text(encoding="utf-8")))
    script = example_path.read_text(encoding="utf-8").rstrip() + "\n"
    snippet = example_snippet_for_text_fence(script)

    # Drop trailing "Next" section — index provides navigation
    body = re.sub(r"\n## Next\n[\s\S]*\Z", "", body).rstrip() + "\n"

    if "```text" not in body and "## The script" in body:
        body += f"\n## Runnable script\n\n```text\n{snippet}\n```\n"

    body += f"\n```dice\n{script}```\n"
    return body


def main() -> None:
    for slug, ex_name in sorted(EXAMPLE_FOR_SLUG.items()):
        md = DOCS / f"{slug}.md"
        ex = EXAMPLES / ex_name
        out = DOCS / f"{slug}.dice"
        if not md.is_file() or not ex.is_file():
            raise SystemExit(f"missing inputs for {slug}")
        out.write_text(build_literate(md, ex), encoding="utf-8")
        print(f"wrote {out.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
