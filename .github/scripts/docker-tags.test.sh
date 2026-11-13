#!/usr/bin/env bash
# Tests for docker-tags.sh. Run: bash .github/scripts/docker-tags.test.sh
set -uo pipefail

script="$(cd "$(dirname "$0")" && pwd)/docker-tags.sh"
failures=0

# expect_pass NAME IMAGE VERSION PRERELEASE EXPECTED_STDOUT
expect_pass() {
  local out
  if ! out="$(bash "$script" "$2" "$3" "$4" 2>&1)"; then
    echo "FAIL $1: exited non-zero: $out"
    failures=$((failures + 1))
  elif [[ "$out" != "$5" ]]; then
    printf 'FAIL %s:\n  expected: %q\n  actual:   %q\n' "$1" "$5" "$out"
    failures=$((failures + 1))
  else
    echo "ok   $1"
  fi
}

expect_pass "stable release gets its version, major.minor and latest" \
  ghcr.io/owner/app 1.2.3 false $'ghcr.io/owner/app:1.2.3\nghcr.io/owner/app:1.2\nghcr.io/owner/app:latest'
expect_pass "pre-release gets only its own version" \
  ghcr.io/owner/app 1.2.3-rc.1 true 'ghcr.io/owner/app:1.2.3-rc.1'

if ((failures > 0)); then
  echo "$failures test(s) failed"
  exit 1
fi
