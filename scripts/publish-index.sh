#!/usr/bin/env bash
#
# Publish generated results back to the beholder ref.
#
# git moves the bytes and beholder reconciles. The push is ordinary git, so it
# goes over whatever protocol and credentials the checkout already has, and
# nothing here knows it is running on GitHub. Beholder starts no git process
# and does not push.
#
# The push is never forced. A rejection means another writer got there first.
# The answer is to fetch the tip that beat us and have beholder rebuild the
# local index commits onto it, which keeps the winner's work and puts ours on
# top as a fast-forward. A force push would make that impossible to notice.
#
# Only a rejected ref is a lost race. Anything else — an unreachable remote,
# refused credentials, a protected ref — is a fault, and retrying it would only
# repeat it.
#
# Environment:
#   REMOTE    remote to publish to, default origin
#   BEHOLDER  path to the beholder binary
#   ATTEMPTS  how many times to give way to another writer, default 3

set -euo pipefail

remote="${REMOTE:-origin}"
attempts="${ATTEMPTS:-3}"
beholder="${BEHOLDER:?BEHOLDER is required to rebuild against another writer}"
ref="refs/beholder/index"
tab=$'\t'

if ! git show-ref --verify --quiet "$ref"; then
  echo "no index to publish"
  exit 0
fi

errors="$(mktemp)"
trap 'rm -f "$errors"' EXIT

for attempt in $(seq 1 "$attempts"); do
  # No leading '+'. --porcelain gives one stable status line per ref, so a lost
  # race is read from the ref's own line and not guessed from stderr.
  status=0
  result="$(LC_ALL=C git push --porcelain "$remote" "$ref:$ref" 2> "$errors")" || status=$?

  if [ "$status" -eq 0 ]; then
    echo "published $ref to $remote"
    exit 0
  fi

  # "[rejected]" is a non-fast-forward. "[remote rejected]" is the remote
  # refusing for its own reasons, which no rebuild can fix.
  if ! grep -q "^!${tab}${ref}:${ref}${tab}\[rejected\]" <<< "$result"; then
    echo "pushing $ref to $remote failed:" >&2
    printf '%s\n' "$result" >&2
    cat "$errors" >&2
    exit 1
  fi

  echo "another writer won attempt $attempt; rebuilding on their tip"
  # Into FETCH_HEAD, so the local ref only moves through beholder's
  # compare-and-swap.
  git fetch --quiet --no-tags "$remote" "$ref"
  "$beholder" store --rebuild-onto "$(git rev-parse FETCH_HEAD)" --limit 0 > /dev/null
done

echo "gave up publishing $ref to $remote after $attempts attempts" >&2
exit 1
