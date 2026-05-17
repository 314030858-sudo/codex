#!/usr/bin/env bash
# 一次安装：让治理检查自动执行（无需每次手动跑）
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

mkdir -p .githooks

cat > .githooks/pre-commit <<'HOOK'
#!/usr/bin/env bash
set -euo pipefail

# 只要提交中涉及 agent 治理相关文件，就自动执行检查
if git diff --cached --name-only | grep -E '^(docs/agent/|scripts/(onebox_agent_governance|validate_agent_assets|run_regression_checks)\.py|ci/check_agent_governance\.sh)' >/dev/null; then
  echo "[pre-commit] Running agent governance checks..."
  ./ci/check_agent_governance.sh
fi
HOOK

cat > .githooks/pre-push <<'HOOK'
#!/usr/bin/env bash
set -euo pipefail

echo "[pre-push] Running full onebox governance check..."
./scripts/onebox_agent_governance.sh
HOOK

chmod +x .githooks/pre-commit .githooks/pre-push

git config core.hooksPath .githooks

echo "✅ Auto governance enabled."
echo "- pre-commit: governance file changes trigger ci/check_agent_governance.sh"
echo "- pre-push: always runs scripts/onebox_agent_governance.sh"
echo "- hooks path: $(git config --get core.hooksPath)"
