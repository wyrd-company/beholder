; Nesting-weighted cognitive complexity.
;
; Same five capture classes as every language; see docs/adding-a-language.md.

; --- control flow: increments and nests ---

(if_statement) @increment @nesting
(for_statement) @increment @nesting
(expression_switch_statement) @increment @nesting
(type_switch_statement) @increment @nesting
(select_statement) @increment @nesting

; --- else and else-if: flat +1, and the `if` it wraps is not a second
;     decision point ---

(if_statement alternative: (block) @increment.flat)
(if_statement alternative: (if_statement) @increment.flat @no_increment @no_nesting)

; --- function literals nest without adding a decision point ---

(func_literal) @nesting

; --- boolean operator sequences count once per sequence ---

(binary_expression operator: "&&") @increment.flat
(binary_expression operator: "||") @increment.flat
(binary_expression left: (binary_expression operator: "&&") @no_increment operator: "&&")
(binary_expression operator: "&&" right: (binary_expression operator: "&&") @no_increment)
(binary_expression left: (binary_expression operator: "||") @no_increment operator: "||")
(binary_expression operator: "||" right: (binary_expression operator: "||") @no_increment)

; --- jumps to a label ---

(break_statement (label_name)) @increment.flat
(continue_statement (label_name)) @increment.flat
(goto_statement) @increment.flat
