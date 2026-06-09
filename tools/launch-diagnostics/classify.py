#!/usr/bin/env python3
"""
classify.py — parse a Wine launch log and emit a structured verdict.

Usage:
    python3 tools/launch-diagnostics/classify.py [logfile]

If logfile is omitted, reads the most recent file in data/launch/.
Emits JSON: {log, stage, winerosetta, signatures, hints, evidence}
"""

import json
import re
import sys
from pathlib import Path
from typing import Optional

REPO_ROOT = Path(__file__).resolve().parents[2]
LOG_DIR = REPO_ROOT / "data" / "launch"

# Ordered from earliest to latest milestone — last match sets `stage`.
MILESTONES = [
    {
        "name": "dll_not_found",
        "pattern": re.compile(r"err:.*import_dll.*not found|err:.*loader_init.*c0000135", re.I),
        "stage": "dll_missing",
        "hint": "A required DLL is missing (c0000135). Check libDllLdr.dll, mods/winerosetta.dll. libSiliconPatch.dll is optional.",
    },
    {
        "name": "dxvk_init",
        "pattern": re.compile(r"DXVK|d3d9.*initialized|MoltenVK", re.I),
        "stage": "graphics_init",
        "hint": "DXVK/MoltenVK initialized — graphics layer is up.",
    },
    {
        "name": "x87_exception",
        "pattern": re.compile(r"unhandled exception|EXCEPTION_ILLEGAL_INSTRUCTION", re.I),
        "stage": "x87_crash",
        "hint": "Unhandled x87/illegal-instruction exception — winerosetta VEH may not cover this opcode.",
    },
    {
        "name": "wowerror",
        "pattern": re.compile(r"WowError\.exe|starting.*WowError", re.I),
        "stage": "crashed",
        "hint": "WoW crashed into WowError.exe.",
    },
    {
        "name": "login_screen",
        "pattern": re.compile(r"character.select|login.screen|GlueXML|RealmList", re.I),
        "stage": "login_screen",
        "hint": "Reached login/character-select screen — launch successful.",
    },
]


def find_latest_log() -> Optional[Path]:
    if not LOG_DIR.exists():
        return None
    logs = sorted(LOG_DIR.glob("*.log"), reverse=True)
    return logs[0] if logs else None


def classify(log_path: Path) -> dict:
    text = log_path.read_text(errors="replace")
    lines = text.splitlines()

    matched_names = []
    stage = "unknown"
    hints = []
    evidence = {}

    for sig in MILESTONES:
        if sig["pattern"].search(text):
            matched_names.append(sig["name"])
            stage = sig["stage"]
            hints.append(sig["hint"])
            for i, line in enumerate(lines):
                if sig["pattern"].search(line):
                    evidence[sig["name"]] = {"line": i + 1, "text": line.strip()}
                    break

    # winerosetta injection is checked separately — it is not a milestone but a
    # prerequisite whose absence explains crashes even at advanced stages.
    winerosetta_loaded = bool(re.search(r"winerosetta", text, re.I))
    if winerosetta_loaded:
        injection_hint = "winerosetta VEH is active."
    else:
        injection_hint = (
            "winerosetta did not appear in log — DivxDecoder bootstrap may have failed. "
            "Delete DivxDecoder.dll.bak and re-run the launcher to re-patch."
        )

    return {
        "log": str(log_path),
        "stage": stage,
        "winerosetta": "loaded" if winerosetta_loaded else "absent",
        "signatures": matched_names,
        "hints": hints + [injection_hint],
        "evidence": evidence,
    }


def main() -> None:
    if len(sys.argv) > 1:
        log_path = Path(sys.argv[1])
    else:
        log_path = find_latest_log()

    if log_path is None or not log_path.exists():
        print(json.dumps({"error": "No log file found", "log_dir": str(LOG_DIR)}))
        sys.exit(1)

    print(json.dumps(classify(log_path), indent=2))


if __name__ == "__main__":
    main()
