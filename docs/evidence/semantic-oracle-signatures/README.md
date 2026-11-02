---
relationships:
  references:
    - semantic-oracle-signatures
---

# Evidence for semantic-oracle signature facts

These files support the technical design `semantic-oracle-signatures`. They are
evidence, not answer keys, and no tool reads them.

| File | Content |
| --- | --- |
| `representation_test.py` | 30 before-and-after pairs, and 14 slot regressions outside the comparison. Compiles or runs a consumer against both versions, then asks each representation whether it reports a change. Needs `rustc`, `tsc`, `go` and `python3`. |
| `representation_results.json` | The output of that test. |
| `sample/` | Signature facts written by hand: 88 records for 87 members. The notation matches the design. Ids of hard forms outside the pilots are placeholders. |
| `select_sample.py` | Selects the seeded random part of the sample. |
| `check_sample_references.py` | Checks each `@{id}` in a sample record against the answer key's `type` references. |
| `check_rust_sample.py` | Compares the Rust sample with rustdoc JSON. |
| `check_python_sample.py` | Compares the Python sample with `inspect.signature`. |
| `count_sample.py` | Counts the sample's slots by class. |
| `estimate_members.py` | A rough count of member declarations per language. |

## Limits

- The author of the design wrote the pairs and the `structured` facts in the
  test. The `breaks` result comes from the compiler or runtime. The pairs test
  the forms and do not estimate how often each case occurs.
- One agent wrote the sample. The wall clock in the design is agent time.
- The checks ran on Rust, TypeScript and Python. The C# and Dart rules have no
  run evidence, because neither toolchain is in the environment that ran them.

## Running

`FIXTURES` is the `fixtures/` directory of a `wyrd-company/beholder-fixtures`
checkout at the pinned commit. The answer-key checks read each key as JSON, for
example `yq -o=json . semantic-oracle.yml > rust.json`, and each sample the same
way. The Rust check also needs rustdoc JSON in `rtarget/doc/`, built in a copy of
the project:

```sh
RUSTC_BOOTSTRAP=1 RUSTDOCFLAGS="-Z unstable-options --output-format json" \
  cargo doc --offline --no-deps --lib --workspace --target-dir rtarget
```
