# Agent Governance Assets

This folder contains the core governance assets for the multi-agent platform.

## Files
- `AGENT_OS_RULEBOOK.md`: Human-readable governance baseline.
- `QUALITY_GATES.yaml`: Machine-readable quality and safety checks.
- `LEARNING_CARD_TEMPLATE.json`: Canonical post-task learning record.
- `examples/learning_cards/`: Place real generated learning cards here.

## Quick start
```bash
python3 scripts/validate_agent_assets.py
```

If validation passes, these files are structurally aligned for onboarding and CI checks.

## Regression checks
```bash
python3 scripts/run_regression_checks.py
./ci/check_agent_governance.sh
```

## Zero-touch auto mode (recommended)
Run once:
```bash
./scripts/setup_auto_governance.sh
```

After setup:
- `pre-commit` will auto-run governance checks when governance-related files are staged.
- `pre-push` will always run onebox governance checks before push.
