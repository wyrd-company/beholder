#!/usr/bin/env bash
#
# Drives the real scripts/sticky-comment.sh against a stand-in for `gh` that
# keeps the comments in a file, so the sequence a pull request actually goes
# through — post, edit, edit, fall quiet — is exercised rather than described.
#
# The stand-in is only the transport. Every decision under test belongs to the
# script.

set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
script="$here/../sticky-comment.sh"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

export PATH="$work/bin:$PATH"
export REPO=example/fixture PR=7
export BODY="$work/body.md" QUIET_BODY="$work/quiet.md"

# A `gh` that stores comments as one JSON array on disk. Only the three calls
# the script makes are implemented; anything else is a failure worth seeing.
mkdir -p "$work/bin"
cat >"$work/bin/gh" <<'GH'
#!/usr/bin/env bash
set -euo pipefail
store="$COMMENTS"
[ -f "$store" ] || echo '[]' > "$store"

method=GET
args=()
jq_filter='.'
while [ $# -gt 0 ]; do
  case "$1" in
    api|--paginate) ;;
    --method) method="$2"; shift ;;
    --jq) jq_filter="$2"; shift ;;
    -F) field="$2"; shift ;;
    *) args+=("$1") ;;
  esac
  shift
done
path="${args[0]}"

case "$method:$path" in
  GET:*)
    jq -r "$jq_filter" < "$store"
    ;;
  POST:*)
    body="$(cat "${field#body=@}")"
    id=$(( $(jq 'length' < "$store") + 100 ))
    jq --arg b "$body" --argjson id "$id" \
      '. + [{id: $id, body: $b, html_url: ("https://example.invalid/c/" + ($id|tostring))}]' \
      < "$store" > "$store.tmp" && mv "$store.tmp" "$store"
    jq -r "$jq_filter" <<< "$(jq --argjson id "$id" '.[] | select(.id == $id)' < "$store")"
    ;;
  PATCH:*)
    id="${path##*/}"
    body="$(cat "${field#body=@}")"
    jq --arg b "$body" --argjson id "$id" \
      'map(if .id == $id then .body = $b else . end)' \
      < "$store" > "$store.tmp" && mv "$store.tmp" "$store"
    jq -r "$jq_filter" <<< "$(jq --argjson id "$id" '.[] | select(.id == $id)' < "$store")"
    ;;
  *)
    echo "unexpected gh call: $method $path" >&2
    exit 1
    ;;
esac
GH
chmod +x "$work/bin/gh"

export COMMENTS="$work/comments.json"
marker='<!-- beholder:sticky:v1 -->'
printf '%s\n### beholder\n\nfirst push\n' "$marker" > "$BODY"
printf '%s\n### beholder\n\nnothing crossed the threshold\n' "$marker" > "$QUIET_BODY"

fail() {
  echo "FAIL: $1" >&2
  jq -c '.' < "$COMMENTS" >&2
  exit 1
}

count() { jq 'length' < "$COMMENTS"; }
bodies() { jq -r '.[].body' < "$COMMENTS"; }

# A pull request nobody has commented on gets one comment.
"$script" > "$work/out1"
grep -q '^posted ' "$work/out1" || fail "the first run should post"
[ "$(count)" = 1 ] || fail "the first run should leave exactly one comment"

# Two more pushes. Each rewrites the same comment; none adds another.
printf '%s\n### beholder\n\nsecond push\n' "$marker" > "$BODY"
"$script" > "$work/out2"
grep -q '^edited ' "$work/out2" || fail "the second run should edit"

printf '%s\n### beholder\n\nthird push\n' "$marker" > "$BODY"
"$script" > "$work/out3"
grep -q '^edited ' "$work/out3" || fail "the third run should edit"

[ "$(count)" = 1 ] || fail "three pushes should still be one comment"
bodies | grep -q 'third push' || fail "the comment should hold the latest body"
if bodies | grep -q 'second push'; then fail "an edit should replace, not append"; fi

# A push that falls below the threshold: the body is empty, and the comment
# already standing is corrected rather than left making a stale claim.
: > "$BODY"
"$script" > "$work/out4"
grep -q '^edited ' "$work/out4" || fail "a quiet run should correct the comment it left"
[ "$(count)" = 1 ] || fail "a quiet run must not add a comment"
bodies | grep -q 'nothing crossed the threshold' || fail "the quiet body should be in place"

# And on a pull request where beholder never had anything to say, silence means
# no comment at all.
echo '[]' > "$COMMENTS"
: > "$BODY"
"$script" > "$work/out5"
[ "$(count)" = 0 ] || fail "silence must not create a comment"
grep -q 'saying nothing' "$work/out5" || fail "a silent run should say why it did nothing"

echo "sticky-comment: ok"
