; Identity discriminators.
;
; TypeScript has none that a heuristic can read. Declaration merging lets an
; interface and a namespace share a name deliberately, and overload signatures
; share a name with their implementation — but nothing declared distinguishes
; them the way a Rust #[cfg] does, so those fall back to positional ordinals and
; are documented as such in docs/resolver-accuracy.md.
