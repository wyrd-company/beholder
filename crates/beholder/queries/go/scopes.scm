; Qualified-path segments.
;
; Go declares everything at package scope, so nothing nests and there is very
; little to say here. A method's receiver is handled in symbols.scm, because it
; is written inside the method rather than around it.
;
; A function literal assigned inside a function does not declare a symbol, so
; there is no nesting to qualify.

(function_declaration name: (identifier) @name) @scope
