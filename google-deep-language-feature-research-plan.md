# Google Deep Research plan for language-feature coverage

Supply the following research plan to `seek research google` after replacing
`{LANGUAGE}`.

```text
Research title: Comprehensive {LANGUAGE} feature coverage for a code-graph
reference project

Research objective

Compile a comprehensive, source-grounded checklist of {LANGUAGE} language,
toolchain, build, package, and common ecosystem features that a broad,
idiomatic, self-contained reference project should exercise.

The reference project will be used to evaluate the accuracy and completeness
of code-graph generators. Relevant features are those that change the semantic
entities, identities, relationships, resolution, types, visibility, dispatch,
source inclusion, or diagnostics that a correct code graph should represent.
The research must remain independent of any graph schema, product,
implementation, analysis approach, or fixture theme.

Central research question

Which materially distinct {LANGUAGE} behaviors and project conditions must be
represented so that omitting one would leave a meaningful gap in the coverage
of a code-graph reference project?

Research scope

- Language-defined semantics and constructs.
- Official compiler, interpreter, runtime, and language-service behavior.
- Modules, packages, workspaces, dependencies, and visibility boundaries.
- Build systems, package managers, configuration, conditional inclusion, and
  platform variation.
- Generated, synthesized, partial, or multi-location program elements.
- Valid, invalid, ambiguous, implementation-defined, and unspecified behavior.
- Version-dependent behavior and material differences between supported
  implementations.
- Feature interactions that produce behavior not covered by either feature in
  isolation.
- Common ecosystem conventions when they materially affect program meaning or
  project structure.

Avoid treating every grammar production or standard-library function as an
independent feature. Prefer distinctions that require meaningfully different
project content, semantic interpretation, build treatment, or diagnostic
behavior.

Source strategy

Prioritize sources in this order:

1. Language specifications and official language references.
2. Official compiler, interpreter, runtime, and language-service documentation.
3. Official build-system and package-manager documentation.
4. Official enhancement proposals, release notes, and compatibility guides.
5. Maintainer-owned ecosystem documentation.

Use secondary sources only when primary sources do not cover the behavior.
Label secondary evidence clearly. Record disagreements, ambiguity, and missing
authoritative guidance rather than resolving them through unsupported
inference.

Checklist evidence

For each candidate checklist item, record:

- A report-local stable identifier.
- Feature or behavior name.
- Description of the materially distinct behavior.
- Observable project content that would demonstrate it.
- Important variants, interactions, counterexamples, and failure cases.
- Language, toolchain, implementation, platform, package-manager, version, and
  configuration constraints.
- Classification as normative, implementation-defined, unspecified, or
  ecosystem convention.
- Direct citations to the primary sources supporting the item.
- Confidence as high, medium, or low.

Completeness review

The report must also identify:

- Recommended baseline version and implementation assumptions.
- Material changes across recent supported versions or implementations.
- Behaviors that cannot coexist in one configuration.
- Features commonly omitted from language demonstrations.
- Areas where authoritative sources disagree or remain unclear.
- Likely categories, features, and interactions still missing after the main
  investigation.

Deliverable

Produce a structured Markdown research report. End with a completeness audit 
and an explicit list of unresolved research gaps.

The report does not create source code, choose a project theme, design a
fixture layout, define a scoring system, or evaluate a code-graph generator.
```
