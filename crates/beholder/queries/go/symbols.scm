; Symbols: one record per function and type.
;
; A method's receiver type is part of its name, not a surrounding scope. Scopes
; are read from a symbol's ancestors, and a method's receiver is written inside
; the method itself, so `format` is what puts it into the identity.
;
; The pointer star is deliberately NOT part of that identity: the base type is
; captured out of both `T` and `*T`. Go forbids `func (t T) Do()` and
; `func (t *T) Do()` from coexisting on one base type, so the two forms name the
; same method and switching between them is an ordinary modification of it. The
; alternative — treating the star as identity — would report every such switch
; as one symbol deleted and an unrelated one added, which is exactly the phantom
; pair the identity gate exists to prevent.

((function_declaration name: (identifier) @name) @symbol
 (#set! kind "function"))

((method_declaration
   receiver: (parameter_list
     (parameter_declaration
       type: [(pointer_type (type_identifier) @receiver)
              (type_identifier) @receiver
              (generic_type (type_identifier) @receiver)
              (pointer_type (generic_type (type_identifier) @receiver))]))
   name: (field_identifier) @name) @symbol
 (#set! kind "function")
 (#set! format "{receiver}.{name}"))

((type_declaration (type_spec name: (type_identifier) @name)) @symbol
 (#set! kind "type"))
