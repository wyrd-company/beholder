; Import declarations.
;
; Names and source are separate, so the captures are read directly; see
; ImportStyle::NamedFrom.
;
; The `import` property says what the declaration does with the names it names,
; because TypeScript's forms do different things and collapsing them would let
; the resolver invent edges the language forbids. A namespace import binds one
; object and puts nothing in scope unqualified; a re-export sends names out of
; this file rather than bringing any in.

; import { parse } from './text'
((import_statement
   (import_clause (named_imports (import_specifier name: (identifier) @name !alias)))
   source: (string) @source) @import)

; import { format as fmt } from './text' — one binding, `fmt`, pointing at
; `text/format`. Capturing the pair together is what records the real binding.
((import_statement
   (import_clause
     (named_imports (import_specifier name: (identifier) @name alias: (identifier) @alias)))
   source: (string) @source) @import)

; import def from './def'
((import_statement (import_clause (identifier) @name) source: (string) @source) @import)

; import * as ns from './helpers' — `ns.foo()` resolves, `foo()` does not.
((import_statement
   (import_clause (namespace_import (identifier) @alias))
   source: (string) @source) @import
 (#set! import "namespace"))

; export * from './public' — a barrel. Nothing enters this file's scope.
((export_statement source: (string) @source) @import
 (#set! import "reexport"))
