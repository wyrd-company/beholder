# Independent language-feature research prompt

Send the contents of the following block verbatim to each isolated frontier
research agent after replacing `{LANGUAGE}`. The agent receives no conversation
history or repository context.

```text
You are conducting an independent research pass about one programming language.

Language: {LANGUAGE}

This checklist will guide creation of a reference project used to evaluate the
accuracy and completeness of code-graph generators. Focus on language features
and project conditions that change the semantic entities, identities,
relationships, resolution, types, visibility, dispatch, source inclusion, or
diagnostics that a correct code graph should represent. Do not assume a graph
schema, product, implementation, or analysis approach.

Do not browse the web; inspect pre-existing files or repositories; use skills
or memories; or consult other agents, reports, fixtures, products, or
evaluation systems. You may use shell, read, and write tools only to create,
inspect, format, and validate your own report in the assigned output location.
Do not use them to discover additional context or knowledge. Use only knowledge
already present in your model and the instructions in this prompt. Do not infer
or discuss any more specific downstream context.

Create a comprehensive candidate checklist of language, toolchain, build,
package, and common ecosystem features that a broad, idiomatic,
self-contained project should exercise.

A checklist item belongs when omitting it would leave materially distinct
program behavior, language semantics, project structure, build behavior, or
implementation-defined behavior unrepresented. Do not merely enumerate
grammar productions or standard-library functions.

Consider valid programs, invalid programs, ambiguous behavior,
version-dependent behavior, configuration-dependent behavior, and interactions
between features. Include features requiring multiple files, packages, build
configurations, generated sources, dependencies, or platform variants when
relevant.

Derive your own categories. Do not force the language into a language-neutral
taxonomy.

For every checklist item, provide:

- A report-local stable identifier.
- Feature or behavior name.
- Description of the distinct behavior.
- What observable project content would demonstrate it.
- Important variants, interactions, counterexamples, and failure cases.
- Relevant language, toolchain, platform, or package-manager constraints.
- Whether it is defined by the language, an official implementation, or
  ecosystem convention.
- Confidence: high, medium, or low.
- A likely primary source to verify, if known. Do not invent document titles,
  sections, or links.

Also provide:

- Baseline version and implementation assumptions.
- Features whose support changed materially between recent versions.
- Implementation-defined or unspecified behavior.
- Areas where your knowledge may be incomplete.
- A final audit of categories or feature families that might still be missing.

Do not create source code, choose a project theme, design a fixture layout,
define a scoring system, or recommend an analysis tool.

Return a structured Markdown research report.
```
