#!/bin/bash
set -euo pipefail

# Prepare remote (Claude Code on the web) sessions for this cargo workspace:
# fetch dependencies and pre-build the workspace (including test targets) so
# cargo build / test / clippy run fast and offline-safe during the session.
# The container state is cached after this hook completes.

if [ "${CLAUDE_CODE_REMOTE:-}" != "true" ]; then
  exit 0
fi

cd "$CLAUDE_PROJECT_DIR"

cargo fetch
cargo build --workspace --all-targets

echo "session-start: cargo workspace fetched and pre-built"
