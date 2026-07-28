#!/usr/bin/env bash
#
# Fetch the stored index before analyzing anything.
#
# Two outcomes look alike from a distance and must not be conflated. A remote
# that nobody has indexed yet hands over nothing and that is the ordinary cold
# start — the wildcard refspec matches no refs and git is happy. A remote that
# cannot be reached, or refuses the credentials, is a fault: the run will still
# work, because everything the index holds can be recomputed, but it will silently
# do the expensive thing forever if nobody is told.
#
# So a fault is loud and does not stop the run. Beholder reports; it does not
# fail a build, least of all over its own cache.
#
# Environment:
#   REMOTE  remote to fetch from, default origin

set -euo pipefail

remote="${REMOTE:-origin}"

# Fresh and unpredictable: a fixed name in a shared temporary directory is a
# symlink someone else can plant, and two runs on one machine would overwrite
# each other's diagnosis.
errors="$(mktemp)"
trap 'rm -f "$errors"' EXIT

if ! git fetch --no-tags "$remote" '+refs/beholder/*:refs/beholder/*' 2> "$errors"; then
  echo "::warning title=beholder::could not fetch the stored index from $remote; \
analyzing from scratch. $(tr '\n' ' ' < "$errors")"
  exit 0
fi

if git show-ref --verify --quiet refs/beholder/index; then
  echo "stored index: $(git rev-parse refs/beholder/index)"
else
  echo "cold start: $remote has no stored index yet"
fi
