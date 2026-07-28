#!/usr/bin/env bash
#
# Publish generated results back to the beholder ref.
#
# Ordinary git, on purpose. The stored index has to travel through fetch and
# push or the storage model is a claim rather than a fact, so nothing here knows
# it is running on GitHub.
#
# The push is never forced. A rejection means another writer got there first,
# and the answer is to fetch what beat us and re-run the report: results already
# stored are reused, and only what is still missing is appended to the winning
# tip. A force push would make that impossible to notice.
#
# Environment:
#   REMOTE    remote to publish to, default origin
#   BEHOLDER  path to the beholder binary
#   BASE HEAD the revisions the report compared
#   ATTEMPTS  how many times to give way to another writer, default 3

set -euo pipefail

remote="${REMOTE:-origin}"
attempts="${ATTEMPTS:-3}"

if ! git show-ref --verify --quiet refs/beholder/index; then
  echo "no index to publish"
  exit 0
fi

for attempt in $(seq 1 "$attempts"); do
  if git push "$remote" refs/beholder/index:refs/beholder/index; then
    echo "published refs/beholder/index to $remote"
    exit 0
  fi

  echo "another writer won attempt $attempt; rebuilding on their tip"
  git fetch "$remote" '+refs/beholder/*:refs/beholder/*'
  "${BEHOLDER:?BEHOLDER is required to rebuild against another writer}" \
    report "${BASE:?BASE is required}" "${HEAD:?HEAD is required}" \
    --use-store --format json >/dev/null
done

echo "gave up publishing refs/beholder/index to $remote after $attempts attempts" >&2
exit 1
