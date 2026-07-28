; Qualified-path segments.
;
; A scope node contributes one segment to the qualified path of every symbol
; beneath it. The segment text comes from the `format` property, which defaults
; to "{name}" and may reference any capture in the pattern by name.
;
; The inherent/trait impl distinction is what keeps two methods with the same
; name on the same type from colliding, so it is spelled out in the segment.

(mod_item name: (identifier) @name) @scope

(trait_item name: (type_identifier) @name) @scope

(function_item name: (identifier) @name) @scope

((impl_item trait: (_) @trait type: (_) @type) @scope
 (#set! format "<{type} as {trait}>"))

((impl_item !trait type: (_) @type) @scope
 (#set! format "{type}"))
