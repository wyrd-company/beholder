; Symbols: one record per function and type.
;
; A method's receiver type is part of its name, not a surrounding scope. Scopes
; are read from a symbol's ancestors, and a method's receiver is written inside
; the method itself, so `format` is what puts it into the identity. Keeping the
; pointer star matters: a value receiver and a pointer receiver are different
; methods.

((function_declaration name: (identifier) @name) @symbol
 (#set! kind "function"))

((method_declaration
   receiver: (parameter_list (parameter_declaration type: (_) @receiver))
   name: (field_identifier) @name) @symbol
 (#set! kind "function")
 (#set! format "{receiver}.{name}"))

((type_declaration (type_spec name: (type_identifier) @name)) @symbol
 (#set! kind "type"))
