#!/usr/bin/env bash
set -euo pipefail
python3 scripts/validate_agent_assets.py
python3 scripts/run_regression_checks.py
echo "[PASS] Agent governance checks all green."
