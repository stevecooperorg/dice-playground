#!/usr/bin/env python3
"""Check local links, assets and anchors in a built learning site."""
from html.parser import HTMLParser
from pathlib import Path
import sys
from urllib.parse import unquote, urlsplit


class Links(HTMLParser):
    def __init__(self, text):
        super().__init__()
        self.links = []
        self.ids = set()
        self.feed(text)

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        if "id" in attrs:
            self.ids.add(attrs["id"])
        for key in ("href", "src"):
            if key in attrs:
                self.links.append(attrs[key])


def check(root):
    root = root.resolve()
    errors = []
    for section in ("tutorial", "cookbook"):
        for page in (root / section).glob("*.html"):
            for href in Links(page.read_text()).links:
                url = urlsplit(href)
                if url.scheme or url.netloc or url.path == "/":
                    continue
                target = ((root / url.path.lstrip("/")) if url.path.startswith("/")
                          else page.parent / unquote(url.path)) if url.path else page
                if target.is_dir():
                    target /= "index.html"
                if not target.is_file():
                    errors.append(f"{page.relative_to(root)}: missing {href}")
                elif url.fragment and target.suffix == ".html":
                    if unquote(url.fragment) not in Links(target.read_text()).ids:
                        errors.append(f"{page.relative_to(root)}: missing anchor {href}")
    if errors:
        raise SystemExit("\n".join(errors))


if __name__ == "__main__":
    check(Path(sys.argv[1]))
