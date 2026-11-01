#!/usr/bin/env bash
# Build a small repository and store its index with the git2 (pre-migration) beholder.
set -euo pipefail
BEHOLDER="$1"; out="$2"
work="$(mktemp -d)"; trap 'rm -rf "$work"' EXIT
export GIT_AUTHOR_NAME=test GIT_AUTHOR_EMAIL=test@example.invalid GIT_AUTHOR_DATE='@0 +0000'
export GIT_COMMITTER_NAME=test GIT_COMMITTER_EMAIL=test@example.invalid GIT_COMMITTER_DATE='@0 +0000'
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null
cd "$work"; git init -q -b main .
mkdir -p src web cmd/tool
cat > src/lib.rs <<'R'
pub fn tally(items: &[u32]) -> u32 {
    let mut t = 0;
    for i in items {
        if *i > 0 {
            t += i;
        }
    }
    t
}
R
cat > web/shape.ts <<'T'
export function area(w: number, h: number): number {
  if (w < 0 || h < 0) {
    return 0;
  }
  return w * h;
}
T
cat > cmd/tool/main.go <<'G'
package main

func clamp(v, lo, hi int) int {
	if v < lo {
		return lo
	}
	if v > hi {
		return hi
	}
	return v
}

func main() { _ = clamp(1, 0, 2) }
G
git add -A; git commit -q -m first
"$BEHOLDER" index --store --out-dir "$(mktemp -d)" . >/dev/null
cat >> src/lib.rs <<'R'

pub fn largest(items: &[u32]) -> Option<u32> {
    items.iter().copied().max()
}
R
git add -A; git commit -q -m second
"$BEHOLDER" index --store --use-store --out-dir "$(mktemp -d)" . >/dev/null
git for-each-ref --format='%(refname) %(objectname)'
git log --format='%H %s' main
git log --format='%H %P %T' refs/beholder/index
git bundle create -q "$out" main refs/beholder/index
