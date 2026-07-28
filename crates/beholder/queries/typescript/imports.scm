; Import declarations.
;
; Names and source are separate, so the captures are read directly; see
; ImportStyle::NamedFrom.
;
; A barrel file re-exports rather than imports, so `export * from './x'` is
; captured here too: what it brings into scope is the same question.

((import_statement
   (import_clause (named_imports (import_specifier name: (identifier) @name)))
   source: (string) @source) @import)

((import_statement
   (import_clause (named_imports (import_specifier alias: (identifier) @alias)))
   source: (string) @source) @import)

((import_statement (import_clause (identifier) @name) source: (string) @source) @import)

((import_statement
   (import_clause (namespace_import (identifier) @alias))
   source: (string) @source) @import
 (#set! glob "true"))

((export_statement source: (string) @source) @import
 (#set! glob "true"))
