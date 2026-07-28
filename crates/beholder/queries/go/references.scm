; Identifier occurrences that refer to something.

; --- calls ---

((call_expression function: (identifier) @name) @reference
 (#set! kind "call"))

; `pkg.Thing(..)` and `value.Method(..)` are the same shape here. The qualifier
; is a package name or a variable; tier 2 cannot tell which without types.
((call_expression
   function: (selector_expression operand: (identifier) @qualifier field: (field_identifier) @name)) @reference
 (#set! kind "call"))

((call_expression function: (selector_expression field: (field_identifier) @name)) @reference
 (#set! kind "method")
 (#set! target "scoped"))

; --- types ---

((type_identifier) @name @reference
 (#set! kind "type"))

((qualified_type package: (package_identifier) @qualifier name: (type_identifier) @name) @reference
 (#set! kind "type"))

; --- composite literals name their type ---

((composite_literal type: (type_identifier) @name) @reference
 (#set! kind "type"))
