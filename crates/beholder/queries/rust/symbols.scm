; Symbols: one record per function and type.
;
; Each pattern captures the whole symbol as @symbol and sets `kind`. Any other
; capture in the pattern is a template variable for the `format` property, which
; defaults to "{name}".
;
; Modules, impl blocks and traits are not symbols in their own right; they
; contribute qualified-path segments and live in scopes.scm.

((function_item name: (identifier) @name) @symbol
 (#set! kind "function"))

((struct_item name: (type_identifier) @name) @symbol
 (#set! kind "type"))

((enum_item name: (type_identifier) @name) @symbol
 (#set! kind "type"))

((union_item name: (type_identifier) @name) @symbol
 (#set! kind "type"))

((trait_item name: (type_identifier) @name) @symbol
 (#set! kind "type"))

((type_item name: (type_identifier) @name) @symbol
 (#set! kind "type"))
