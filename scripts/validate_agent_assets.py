#!/usr/bin/env python3
from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RULEBOOK = ROOT / "docs/agent/AGENT_OS_RULEBOOK.md"
GATES = ROOT / "docs/agent/QUALITY_GATES.yaml"
CARD = ROOT / "docs/agent/LEARNING_CARD_TEMPLATE.json"

REQUIRED_RULEBOOK_HEADINGS = [
    "## 2. 强制输出协议（所有 Agent 必须遵守）",
    "## 3. 三态状态机",
    "## 4. 审批令牌机制（无令牌=禁止真实动作）",
    "## 5. 红线风险（必须拦截）",
    "## 9. 每轮任务结束必须生成 Learning Card",
]

REQUIRED_TOP_LEVEL_KEYS = {
    "output_protocol",
    "state_machine",
    "approval_tokens",
    "semantic_guards",
    "mobile_checks",
    "visual_scoring",
    "learning_card",
    "severity_policy",
}
REQUIRED_GATE_RULES = {
    "G001_online_misleading",
    "G002_real_login_misleading",
    "G003_finance_hallucination",
    "G004_role_overreach_daren",
    "G005_missing_copyable_instruction",
    "G006_missing_evidence",
}
REQUIRED_CARD_FIELDS = {
    "version", "card_type", "task_id", "timestamp", "agent", "state", "intent",
    "result", "quality_evidence", "errors", "fix_action", "reusable_prompt",
    "policy_update", "review"
}


def validate_rulebook() -> list[str]:
    errors = []
    text = RULEBOOK.read_text(encoding="utf-8")
    for heading in REQUIRED_RULEBOOK_HEADINGS:
        if heading not in text:
            errors.append(f"Missing heading in rulebook: {heading}")
    return errors


def validate_gates() -> list[str]:
    errors = []
    text = GATES.read_text(encoding="utf-8")

    found_top = set(re.findall(r"^([a-z_]+):", text, flags=re.M))
    missing_top = REQUIRED_TOP_LEVEL_KEYS - found_top
    if missing_top:
        errors.append(f"Missing top-level keys in quality gates: {sorted(missing_top)}")

    guard_ids = set(re.findall(r'id:\s*"([^"]+)"', text))
    missing_ids = REQUIRED_GATE_RULES - guard_ids
    if missing_ids:
        errors.append(f"Missing semantic guards: {sorted(missing_ids)}")
    return errors


def validate_card() -> list[str]:
    errors = []
    data = json.loads(CARD.read_text(encoding="utf-8"))
    missing = REQUIRED_CARD_FIELDS - set(data)
    if missing:
        errors.append(f"Missing learning card keys: {sorted(missing)}")

    allowed = {"PLAN", "SANDBOX", "REAL"}
    declared = set(data.get("state", {}).get("allowed_values", []))
    if declared != allowed:
        errors.append("Learning card state.allowed_values must be exactly PLAN/SANDBOX/REAL")
    return errors


def main() -> int:
    errors = validate_rulebook() + validate_gates() + validate_card()
    if errors:
        print("Validation failed:")
        for e in errors:
            print(f"- {e}")
        return 1
    print("All agent governance assets are valid.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
