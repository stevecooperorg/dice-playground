#!/usr/bin/env python3
"""Measure native CLI defaults and documented expensive parameter bounds.

Build the debug CLI first: cargo build --bin dice.
Timings include process startup; these are not browser responsiveness promises.
"""
import json
from pathlib import Path
import statistics
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parent.parent
BINARY = ROOT / "target/debug/dice"


def measure(path):
    samples = []
    for _ in range(3):
        start = time.perf_counter()
        subprocess.run([str(BINARY), "eval", str(path)], check=True,
                       stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
        samples.append((time.perf_counter() - start) * 1000)
    return round(statistics.median(samples), 2)


if __name__ == "__main__":
    manifest = json.loads((ROOT / "docs/learning-content.json").read_text())
    defaults = [dict(path=page["path"], median_ms=measure(ROOT / "docs" / page["path"]))
                for page in manifest["pages"]]
    bounds = []
    with tempfile.TemporaryDirectory(prefix="dice-benchmark-") as directory:
        for name, relative, before, after in [
            ("callback-five-dice", "tutorial/23-shared-pool-rule.dice", "dice = 2", "dice = 5"),
            ("decomposition-twenty-dice", "cookbook/blades-in-the-dark.dice", "dice = 2", "dice = 20"),
            ("open-ended-three-continuations", "tutorial/29-open-ended-procedures.dice", "max_chain=1", "max_chain=3"),
            ("explosion-six-extra-rolls", "tutorial/28-explosion-caps.dice", "cap = 1", "cap = 6"),
        ]:
            source = (ROOT / "docs" / relative).read_text()
            if before not in source:
                raise ValueError(f"missing benchmark input: {before}")
            target = Path(directory) / (name + ".dice")
            target.write_text(source.replace(before, after))
            bounds.append(dict(name=name, median_ms=measure(target)))
    print(json.dumps(dict(mode="debug native CLI, median of three including process startup",
                          defaults=defaults, supported_bounds=bounds), indent=2))
