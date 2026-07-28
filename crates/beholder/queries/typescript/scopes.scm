; Qualified-path segments.
;
; A class or interface scopes the methods written inside it, which is what keeps
; two same-named methods on different classes apart. Namespaces nest too.

(class_declaration name: (type_identifier) @name) @scope
(abstract_class_declaration name: (type_identifier) @name) @scope
(interface_declaration name: (type_identifier) @name) @scope
(internal_module name: (identifier) @name) @scope
(module name: [(identifier) (string)] @name) @scope
(function_declaration name: (identifier) @name) @scope
