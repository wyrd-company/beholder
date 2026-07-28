; Symbols: one record per function and type.
;
; Arrow functions bound to a const are how much of modern TypeScript declares a
; function, so a lexical declaration whose value is a function counts. An
; anonymous arrow passed as an argument does not: it has no name a later
; revision could match on.

((function_declaration name: (identifier) @name) @symbol
 (#set! kind "function"))

((generator_function_declaration name: (identifier) @name) @symbol
 (#set! kind "function"))

((method_definition name: (property_identifier) @name) @symbol
 (#set! kind "function"))

((variable_declarator
   name: (identifier) @name
   value: [(arrow_function) (function_expression)]) @symbol
 (#set! kind "function"))

((class_declaration name: (type_identifier) @name) @symbol
 (#set! kind "type"))

((abstract_class_declaration name: (type_identifier) @name) @symbol
 (#set! kind "type"))

((interface_declaration name: (type_identifier) @name) @symbol
 (#set! kind "type"))

((type_alias_declaration name: (type_identifier) @name) @symbol
 (#set! kind "type"))

((enum_declaration name: (identifier) @name) @symbol
 (#set! kind "type"))
