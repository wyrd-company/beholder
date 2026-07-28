# @wyrd-company/beholder

The npm launcher for [beholder](https://github.com/wyrd-company/beholder), a
structural index of a codebase that answers two questions: what is risky, and
what depends on what.

```sh
npm install --global @wyrd-company/beholder
beholder --help
```

Installing this package downloads the beholder executable built for the current
platform from the matching GitHub release and verifies it against the release's
`SHA256SUMS` before writing it. The package itself carries no native code; the
`beholder` command is a thin launcher that forwards its arguments, its streams,
and its signals to that executable.

Linux and macOS on arm64, Linux and Windows on x86_64 are supported. Anywhere
else, install the CLI from source with `cargo install beholder-cli`.
