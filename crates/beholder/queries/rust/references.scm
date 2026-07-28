; Identifier occurrences that refer to something.
;
; Every pattern captures the referring node as @reference and the name being
; referred to as @name. An optional @qualifier carries whatever narrowed the
; lookup — the `Type` in `Type::method`, the module in `module::thing`.
;
; Occurrences that land exactly on a symbol's own declared name are dropped
; later: a definition is not a reference to itself.

; --- calls ---

((call_expression function: (identifier) @name) @reference
 (#set! kind "call"))

((call_expression
   function: (scoped_identifier path: (_) @qualifier name: (identifier) @name)) @reference
 (#set! kind "call"))

; A method call carries no receiver type here. The name is all tier 2 has, which
; is why method dispatch is the resolver's weakest case.
((call_expression
   function: (field_expression field: (field_identifier) @name)) @reference
 (#set! kind "method"))

; --- types ---

((type_identifier) @name @reference
 (#set! kind "type"))

((scoped_type_identifier path: (_) @qualifier name: (type_identifier) @name) @reference
 (#set! kind "type"))

; --- paths to values: constants, unit structs, enum variants ---

((scoped_identifier path: (_) @qualifier name: (identifier) @name) @reference
 (#set! kind "path"))
