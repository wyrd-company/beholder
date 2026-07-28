; Nesting-weighted cognitive complexity.
;
; Four capture classes, resolved per node with suppression winning:
;
;   @increment       +1, plus the current nesting level
;   @increment.flat  +1, ignoring nesting
;   @nesting         descendants are one level deeper
;   @no_increment    suppresses any increment on this node
;   @no_nesting      suppresses any nesting on this node
;
; Nesting resets to zero at every symbol boundary, so a score always describes
; the shape of one symbol.

; --- control flow: increments and nests ---

(if_expression) @increment @nesting
(match_expression) @increment @nesting
(while_expression) @increment @nesting
(loop_expression) @increment @nesting
(for_expression) @increment @nesting

; --- else and else-if: flat +1 for the branch, and the `if` it wraps is not a
;     second decision point ---

(else_clause) @increment.flat
(else_clause (if_expression) @no_increment @no_nesting)

; --- closures nest without adding a decision point ---

(closure_expression) @nesting

; --- boolean operator sequences count once per sequence, so an operator
;     directly under the same operator is suppressed ---

(binary_expression operator: "&&") @increment.flat
(binary_expression operator: "||") @increment.flat
(binary_expression left: (binary_expression operator: "&&") @no_increment operator: "&&")
(binary_expression operator: "&&" right: (binary_expression operator: "&&") @no_increment)
(binary_expression left: (binary_expression operator: "||") @no_increment operator: "||")
(binary_expression operator: "||" right: (binary_expression operator: "||") @no_increment)

; --- jumps to a label ---

(break_expression (label)) @increment.flat
(continue_expression (label)) @increment.flat
