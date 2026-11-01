---
beholder: major
---

Beholder reads and writes git through gix, a pure Rust library, instead of libgit2, so a build no longer compiles the libgit2 C library. `beholder store --fetch REMOTE` fetches the stored index over ssh, https or a local path and keeps local index commits on top of what it fetched. `beholder store --rebuild-onto TIP` rebuilds local index commits on the tip that won a push race. Beholder does not push: publish with `git push` without `+`. After a rejected push, the action's publish step rebuilds on the winning tip instead of re-running the report, and any other push failure stops the step. For library users, `Store` and the revision helpers take `gix::Repository` and `gix::ObjectId`, `Store::push` is removed, and `Store::fetch` returns `Reconciled`. Indexes stored by earlier versions read back unchanged and keep their object ids.
