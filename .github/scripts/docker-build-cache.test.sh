#!/usr/bin/env bash
# Tests that the Docker image build reuses compiled Rust dependencies from an exported layer cache,
# as the release workflow does with its registry cache, and that the built image runs as README
# "Docker" describes.
# Run: bash .github/scripts/docker-build-cache.test.sh
#
# It's slow: it compiles the backend in Docker more than once. CI doesn't run it.
#
# Each build runs on a new docker-container builder, like each release runs on a new runner. It
# imports the layer cache the previous build exported, and nothing else: cache mounts don't carry
# over. The test edits a copy of the working tree, never the checkout.
set -uo pipefail

root="$(cd "$(dirname "$0")/../.." && pwd)"
work="$(mktemp -d)"
context="$work/context"
cache="$work/cache"
builder="hypecheck-build-cache-test-$$"
image="hypecheck-build-cache-test:$$"
container="hypecheck-build-cache-test-$$"
volume="hypecheck-build-cache-test-$$"
failures=0

cleanup() {
  docker rm -fv "$container" >/dev/null 2>&1
  docker image rm "$image" >/dev/null 2>&1
  docker volume rm "$volume" >/dev/null 2>&1
  docker buildx rm "$builder" >/dev/null 2>&1
  rm -rf "$work"
}
trap cleanup EXIT

# Tracked and untracked files, leaving out what .gitignore ignores.
mkdir "$context"
git -C "$root" ls-files -z --cached --others --exclude-standard \
  | tar -C "$root" --null -T - -cf - \
  | tar -C "$context" -xf -

# build NAME [BUILD_ARGS...]: builds the copy on a new builder, importing and replacing the layer
# cache, and leaves the plain build log in $work/NAME.log.
build() {
  local name="$1" status
  shift
  local from=()
  if [[ -d "$cache" ]]; then from=(--cache-from "type=local,src=$cache"); fi

  docker buildx create --name "$builder" --driver docker-container >/dev/null || return 1
  docker buildx build --builder "$builder" --progress=plain \
    ${from[@]+"${from[@]}"} --cache-to "type=local,dest=$cache.new,mode=max" \
    "$@" "$context" >"$work/$name.log" 2>&1
  status=$?
  docker buildx rm "$builder" >/dev/null 2>&1

  if ((status != 0)); then
    echo "docker build exited $status. Last lines of the $name log:"
    tail -n 40 "$work/$name.log"
    return 1
  fi
  rm -rf "$cache"
  mv "$cache.new" "$cache"
}

# fail NAME MESSAGE
fail() {
  echo "FAIL $1: $2"
  failures=$((failures + 1))
}

# expect_rebuild_compiles_only_backend NAME: builds the copy as it now stands, and checks that cargo
# compiled the backend crate and none of its dependencies.
expect_rebuild_compiles_only_backend() {
  local crates
  if ! build "$1"; then
    fail "$1" "docker build failed"
    return
  fi
  crates="$(sed -nE 's/.* Compiling ([^ ]+) v.*/\1/p' "$work/$1.log" | sort -u)"
  if [[ "$crates" != hypecheck_api ]]; then
    fail "$1" "expected only hypecheck_api to compile, compiled:"$'\n'"${crates:-(nothing)}"
  else
    echo "ok   $1"
  fi
}

# expect_runs_as_documented NAME: runs $image as README "Docker" describes it, without IGDB
# credentials, and checks it serves on 8080, keeps its database in the /data volume, runs as the
# non-root hypecheck user and reports healthy.
expect_runs_as_documented() {
  local name="$1" health="" address page i
  if ! docker run -d --name "$container" -p 127.0.0.1::8080 -v "$volume:/data" \
    -e SESSION_SECRET=test "$image" >/dev/null; then
    fail "$name" "docker run failed"
    return
  fi

  # The first health check runs one interval (30s) after the start.
  for ((i = 0; i < 60; i++)); do
    health="$(docker inspect -f '{{.State.Health.Status}}' "$container")"
    [[ "$health" == starting ]] || break
    sleep 2
  done
  if [[ "$health" != healthy ]]; then
    fail "$name" "expected health healthy, got ${health:-nothing}. Container log:"
    docker logs "$container" 2>&1 | tail -n 20
    return
  fi

  address="$(docker port "$container" 8080/tcp | head -n 1)"
  if [[ "$(curl -fsS "http://$address/api/health")" != ok ]]; then
    fail "$name" "GET /api/health on port 8080 didn't answer ok"
  elif ! page="$(curl -fsS "http://$address/")" || ! grep -qi '<html' <<<"$page"; then
    fail "$name" "GET / on port 8080 didn't serve the frontend"
  elif ! docker exec "$container" test -s /data/hypecheck.db; then
    fail "$name" "no database at /data/hypecheck.db"
  elif [[ "$(docker exec "$container" id -un)" != hypecheck ]]; then
    fail "$name" "expected to run as hypecheck, runs as $(docker exec "$container" id -un)"
  else
    echo "ok   $name"
  fi
  docker rm -fv "$container" >/dev/null 2>&1
}

echo "Building from scratch to fill the cache. This compiles every dependency."
build initial --load --tag "$image" || exit 1
expect_runs_as_documented runs-as-documented

echo '// changed by docker-build-cache.test.sh' >>"$context/backend/src/main.rs"
expect_rebuild_compiles_only_backend source-change

# A release bumps the version in Cargo.toml, and with it the backend's own entry in Cargo.lock,
# while its dependencies stay the same. The Dockerfile's backend-manifests stage undoes this edit.
sed -i.bak -E '1,/^version = /s/^version = .*/version = "99.0.0"/' "$context/backend/Cargo.toml"
sed -i.bak -E '/^name = "hypecheck_api"$/{n;s/^version = .*/version = "99.0.0"/;}' "$context/backend/Cargo.lock"
rm "$context"/backend/*.bak
expect_rebuild_compiles_only_backend version-bump

if ((failures > 0)); then
  echo "$failures test(s) failed"
  exit 1
fi
