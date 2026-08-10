# Canonical language-feature checklist synthesis prompt

Send the contents of the following block to a fresh synthesis agent for each
language. Supply only the three completed research reports. Do not provide
conversation history, repository context, a fixture theme, or information
about downstream consumers.

```text
You are an independent synthesis agent.

Language: {LANGUAGE}

Inputs:

- One isolated Codex research report.
- One isolated Claude Fable research report.
- One Google Deep Research report with citations.

You have no downstream product or evaluation context. Do not infer one. Do not
choose a project theme or design source code.

Create the canonical language-feature checklist by reconciling the three
reports.

Do not use majority vote. Resolve conflicts through cited primary sources.
Preserve a unique item when it is plausible but not yet verified. Mark it as a
research gap rather than silently removing it.

Merge duplicates only when they describe the same materially observable
behavior. Preserve language-specific distinctions, version boundaries,
mutually exclusive configurations, negative cases, ambiguity, and feature
interactions.

Each canonical item must contain:

- Stable canonical identifier.
- Feature or behavior.
- Fixture obligation: what a project must demonstrate.
- Required variants, interactions, counterexamples, and failure cases.
- Version, implementation, platform, and configuration constraints.
- Classification: language-defined, implementation-defined, unspecified, or
  ecosystem convention.
- Primary-source citations.
- Provenance showing which input reports proposed it.
- Status: required, conditional, or unresolved research gap.

Separate unresolved disagreements and unsupported claims from the canonical
required checklist.

Finish with a completeness audit covering:

- Input-report items omitted during synthesis and why.
- Categories represented by only one source.
- Areas with weak or missing primary evidence.
- Likely blind spots requiring another research pass.

Return a structured Markdown document suitable to serve as the single source
of truth for fixture creation.
```
