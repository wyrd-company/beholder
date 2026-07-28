#!/usr/bin/env bash
#
# One comment per pull request, edited in place.
#
# The marker in the body is the only state this keeps. Finding it is what makes
# the comment sticky: no database, no run id, no label, nothing to migrate. A
# comment beholder cannot find is a comment beholder will duplicate, so the
# search runs over every page before it decides to post.
#
# Silence is a real outcome. An empty body means nothing crossed the threshold,
# and beholder does not open a conversation to say it has nothing to say. The
# one exception is a comment left by an earlier push: that comment is making a
# claim about code that has since changed, so it is corrected in place rather
# than left standing. Correcting is still one comment.
#
# Environment:
#   REPO       owner/name
#   PR         pull request number
#   BODY       file holding the comment body; empty means nothing to say
#   QUIET_BODY file holding the body for a comment that has gone quiet
#   GH_TOKEN   token gh authenticates with

set -euo pipefail

: "${REPO:?REPO is required}"
: "${PR:?PR is required}"
: "${BODY:?BODY is required}"

marker='<!-- beholder:sticky:v1 -->'

# The lowest id wins, so anything that manages to create a second marked comment
# still cannot turn beholder into a repeat commenter.
existing="$(
  gh api --paginate "repos/$REPO/issues/$PR/comments" \
    --jq ".[] | select(.body != null) | select(.body | contains(\"$marker\")) | .id" |
    sort -n | head -1
)"

body="$BODY"
if [ ! -s "$BODY" ]; then
  if [ -z "$existing" ]; then
    echo "nothing crossed the threshold and no comment to correct; saying nothing"
    exit 0
  fi
  body="${QUIET_BODY:?QUIET_BODY is required to correct a comment}"
fi

if [ -n "$existing" ]; then
  echo "edited $(gh api --method PATCH "repos/$REPO/issues/comments/$existing" \
    -F "body=@$body" --jq '.html_url')"
else
  echo "posted $(gh api --method POST "repos/$REPO/issues/$PR/comments" \
    -F "body=@$body" --jq '.html_url')"
fi
