---
relationships:
  references:
    - github-action
    - install
---

# Publishing

Pull requests run `task ci`, Intentional checks and Beholder reports. Linux
x86_64 and arm64, macOS and Windows test the locked workspace with Rust 1.85
and run the npm launcher tests. Formatting and Clippy use the stable Rust toolchain.

The Release workflow accepts manual dispatch for a build-only run. It runs the
CI workflow against the selected commit, then checks
manifest version agreement, builds the four supported archive targets and
retains the archives as workflow artifacts. The Linux arm64 archive is executed
on a native arm64 runner before the GitHub Release job can start. It does not
require publication
credentials or publish packages. A manually selected tag also runs build-only.

A plain Semantic Versioning tag push starts publication. Intentional manages
versions, changelog entries and release tags. The tag must agree with Cargo and
npm versions and have a matching changelog section.

The workflow creates or verifies the GitHub Release with all four archives and
`SHA256SUMS`. The repository must enable immutable releases so that the release
and its assets can be verified. Only after that job succeeds do the three
package channels run:

- crates.io: `beholder`, followed by `beholder-cli`;
- npm: `@wyrd-company/beholder`;
- Homebrew: `wyrd-company/tools/beholder`.

Each channel verifies matching published content on a retry and rejects
conflicting content. Clean-client jobs install the published CLI through each
channel. Release runs for the same Git ref execute in sequence.

Publication requires the configured `CARGO_REGISTRY_TOKEN`,
`FORMULAE_PUBLISH_KEY` and `NPM_TOKEN` secrets. Values remain in GitHub Actions.
The GitHub Release job uses the workflow token.

Documentation publishes independently when `docs/**` changes on `main`, or by
manual dispatch of Publish docs. Manual dispatch publishes documentation from
the selected ref. Documentation publication runs in sequence across refs. It uses
`WYRD_TOOLS_DOCS_PUBLISHER_APP_ID` and
`WYRD_TOOLS_DOCS_PUBLISHER_PRIVATE_KEY`. A package release is not required.
