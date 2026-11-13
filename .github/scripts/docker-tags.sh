#!/usr/bin/env bash
# Lists the tags a release pushes for the container image, one per line.
#
# Usage: docker-tags.sh <image> <version> <prerelease>
#
# <version> and <prerelease> are the outputs of check-release-version.sh. A stable release
# X.Y.Z gets X.Y.Z, X.Y and latest. A pre-release gets only its own version, so it never moves
# the tags people pull for a stable release.
set -euo pipefail

image="$1"
version="$2"
prerelease="$3"

echo "$image:$version"
if [[ "$prerelease" == true ]]; then
  exit 0
fi

IFS=. read -r major minor _ <<<"$version"
echo "$image:$major.$minor"
echo "$image:latest"
