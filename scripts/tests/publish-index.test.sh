#!/usr/bin/env bash
#
# Drives the real scripts/publish-index.sh against a real bare remote and two
# real clones, because the claim under test is about git and nothing else: a
# runner cold-starts from what someone else already indexed, and a writer that
# loses a race gives way without discarding the winner's work.
#
# No stand-ins here. The only thing simulated is the second machine.

set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$here/../.." && pwd)"
publish="$here/../publish-index.sh"
fetch="$here/../fetch-index.sh"

beholder="${BEHOLDER:-}"
if [ -z "$beholder" ]; then
  cargo build -q --manifest-path "$root/Cargo.toml" -p beholder-cli
  beholder="$root/target/debug/beholder"
fi

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

fail() {
  echo "FAIL: $1" >&2
  exit 1
}

commit() {
  local dir="$1" name="$2" body="$3"
  mkdir -p "$dir/src"
  printf 'pub fn %s(items: &[u32]) -> u32 {\n%s}\n' "$name" "$body" > "$dir/src/lib.rs"
  git -C "$dir" add -A
  git -C "$dir" commit -q -m "$name"
}

clone() {
  git clone -q "$work/origin.git" "$work/$1"
  git -C "$work/$1" config user.email beholder@example.invalid
  git -C "$work/$1" config user.name beholder
}

report() {
  local dir="$1"
  ( cd "$dir" && "$beholder" report "$(git rev-parse HEAD~1)" "$(git rev-parse HEAD)" \
      --use-store --format json > /dev/null )
}

run_publish() {
  local dir="$1"
  ( cd "$dir" && BEHOLDER="$beholder" \
      BASE="$(git rev-parse HEAD~1)" HEAD="$(git rev-parse HEAD)" \
      bash "$publish" )
}

run_fetch() {
  local dir="$1" remote="${2:-origin}"
  ( cd "$dir" && REMOTE="$remote" bash "$fetch" )
}

indexed_sources() {
  ( cd "$1" && "$beholder" store --limit 50 | sed -n 's/.*source \([0-9a-f]*\).*/\1/p' )
}

git init -q --bare -b main "$work/origin.git"
clone first
commit "$work/first" tally '    items.iter().sum()\n'
commit "$work/first" tally '    let mut t = 0;\n    for i in items {\n        if *i > 0 {\n            t += i;\n        }\n    }\n    t\n'
git -C "$work/first" push -q origin HEAD:main

# Nothing is indexed yet: fetching says so plainly, and there is nothing to
# publish. That is the cold start, and it is not a fault.
run_fetch "$work/first" > "$work/cold" 2>&1 || fail "a cold start must not fail"
grep -q 'cold start' "$work/cold" || fail "the cold start should say so: $(cat "$work/cold")"
if grep -q '::warning' "$work/cold"; then fail "an empty remote is not a fault"; fi

run_publish "$work/first" | grep -q 'no index to publish' \
  || fail "an unindexed repository should have nothing to publish"

# A remote that cannot be reached looks the same from a distance and is not the
# same thing at all: the run continues, because everything the index holds can
# be recomputed, but it says loudly that it is doing so.
git -C "$work/first" remote add broken "$work/nowhere.git"
run_fetch "$work/first" broken > "$work/broken" 2>&1 \
  || fail "an unreachable remote must not stop the run"
grep -q '::warning' "$work/broken" \
  || fail "an unreachable remote should warn: $(cat "$work/broken")"
if grep -q 'cold start' "$work/broken"; then fail "a fault must not read as a cold start"; fi

report "$work/first"
run_publish "$work/first" | grep -q 'published' || fail "the first publish should succeed"

# A second machine cold-starts from the remote and finds the work already done.
clone second
run_fetch "$work/second" > "$work/warm" 2>&1 || fail "fetching a stored index should work"
grep -q 'stored index: ' "$work/warm" || fail "the fetched index should be named: $(cat "$work/warm")"
first_head="$(git -C "$work/first" rev-parse HEAD)"
( cd "$work/second" && "$beholder" report "$first_head~1" "$first_head" --use-store --format json \
    > /dev/null ) 2> "$work/second-origins"
grep -q 'reused a stored analysis' "$work/second-origins" \
  || fail "a fresh clone should reuse the stored analysis: $(cat "$work/second-origins")"

# The second machine indexes something of its own and publishes it.
commit "$work/second" totalled '    let mut t = 0;\n    for i in items {\n        if *i > 1 {\n            t += i;\n        }\n    }\n    t\n'
report "$work/second"
run_publish "$work/second" | grep -q 'published' || fail "the second machine should publish"
second_head="$(git -C "$work/second" rev-parse HEAD)"

# Meanwhile the first machine, which never saw any of that, indexes a commit of
# its own and pushes into the race it has already lost.
commit "$work/first" counted '    items.iter().filter(|i| **i > 0).sum()\n'
report "$work/first"
run_publish "$work/first" > "$work/race" 2>&1 || fail "the loser should recover: $(cat "$work/race")"
grep -q 'another writer won' "$work/race" || fail "the push should have been rejected first"
grep -q 'published' "$work/race" || fail "the loser should publish after giving way"

# Both writers' work survives, which is the point of never forcing the push.
git -C "$work/second" fetch -q origin '+refs/beholder/*:refs/beholder/*'
sources="$(indexed_sources "$work/second")"
grep -q "$second_head" <<< "$sources" || fail "the winner's index was discarded"
grep -q "$(git -C "$work/first" rev-parse HEAD)" <<< "$sources" \
  || fail "the loser's index never landed"

echo "publish-index: ok"
