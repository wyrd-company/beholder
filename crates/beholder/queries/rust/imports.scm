; Import declarations.
;
; The capture is the whole import tree. Expanding it into individual bindings is
; generic, driven by the `import_syntax` entry in the language table: what
; separates path segments, what groups a list, what introduces an alias, and
; what a glob looks like.
;
; Keeping the expansion out of the query is deliberate. A use tree nests
; arbitrarily — `use a::{b, c::{d, e as f}}` — and a query that tried to enumerate
; the shapes would be a list of special cases per language.

(use_declaration argument: (_) @import)
