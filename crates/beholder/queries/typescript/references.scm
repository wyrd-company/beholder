; Identifier occurrences that refer to something.

((call_expression function: (identifier) @name) @reference
 (#set! kind "call"))

((call_expression
   function: (member_expression object: (identifier) @qualifier property: (property_identifier) @name)) @reference
 (#set! kind "call"))

((call_expression function: (member_expression property: (property_identifier) @name)) @reference
 (#set! kind "method")
 (#set! target "scoped"))

((new_expression constructor: (identifier) @name) @reference
 (#set! kind "type"))

((type_identifier) @name @reference
 (#set! kind "type"))

((nested_type_identifier module: (_) @qualifier name: (type_identifier) @name) @reference
 (#set! kind "type"))
