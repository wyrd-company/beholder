# Exercising the action against a real pull request

The action's behaviour is pinned by `scripts/tests/*.test.sh`, which drive the
same scripts the action runs. What those tests cannot show is GitHub itself:
that a comment identified by a marker is the same comment GitHub edits, that
SARIF at note level becomes an inline annotation, and that a runner's checkout
can fetch and publish a ref outside `refs/heads/*`. That takes a pull request.

## What it needs

A throwaway repository the runner can write to, with Actions enabled and
`security-events: write` available for the SARIF upload. Code scanning upload is
a paid feature on private repositories, so a public throwaway is the simpler
path; a private one demonstrates everything except the annotations.

## The fixture

One small file per language, so the report has something to rank in each and the
comment has to merge percentiles across them.

```text
src/lib.rs        a Rust function others call
pkg/tally/tally.go a Go function others call
web/src/index.ts  a TypeScript function others call
.github/workflows/beholder.yml   the workflow from the README
```

The action is `uses: wyrd-company/beholder@<ref>`, or a path if the source is
checked out into the fixture.

## The four behaviours to capture

1. **The comment appears.** Open a pull request that adds real nesting to one of
   the fixture functions. Expect one comment carrying the marker, listing that
   symbol first. Record the comment URL.
2. **It is edited, not repeated.** Push twice more, each time changing the
   ranking — deepen a second function, then simplify the first. Expect the same
   comment id each time and no second comment. Record the comment's edit history.
3. **Silence.** Open a second pull request whose only change moves no
   complexity, such as renaming a local variable or reflowing a comment. Expect
   no comment at all, and a run log saying nothing crossed the threshold.
4. **Cold start.** The first run logs that the remote has no stored index and
   analyzes both revisions. Every later run logs the fetched ref and
   `reused a stored analysis` for the base, which does not change between
   pushes. Record both run logs, and the ref on the remote afterwards:

   ```sh
   git ls-remote origin 'refs/beholder/*'
   ```

The SARIF upload is visible on the pull request's Files tab as annotations on
the ranked line ranges, all at note or warning level, none at error level, and
no check turns red.
