#!/usr/bin/env python3
import json
from pathlib import Path

CASES = Path("docs/agent/examples/regression_cases/regression_cases_v1.json")

def main():
    data = json.loads(CASES.read_text(encoding="utf-8"))
    print(f"Loaded {len(data['cases'])} regression cases.")
    required = {"id", "input", "expected", "must_not"}
    for c in data["cases"]:
        miss = required - set(c.keys())
        if miss:
            raise SystemExit(f"[FAIL] case {c.get('id')} missing keys: {sorted(miss)}")
    print("[PASS] regression case schema valid.")

if __name__ == "__main__":
    main()
