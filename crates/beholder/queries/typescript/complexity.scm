; Nesting-weighted cognitive complexity.

; --- control flow: increments and nests ---

(if_statement) @increment @nesting
(for_statement) @increment @nesting
(for_in_statement) @increment @nesting
(while_statement) @increment @nesting
(do_statement) @increment @nesting
(switch_statement) @increment @nesting
(catch_clause) @increment @nesting
(ternary_expression) @increment @nesting

; --- else and else-if ---

(else_clause) @increment.flat
(else_clause (if_statement) @no_increment @no_nesting)

; --- functions nest without adding a decision point ---

(arrow_function) @nesting
(function_expression) @nesting

; --- boolean and nullish operator sequences count once per sequence ---

(binary_expression operator: "&&") @increment.flat
(binary_expression operator: "||") @increment.flat
(binary_expression operator: "??") @increment.flat
(binary_expression left: (binary_expression operator: "&&") @no_increment operator: "&&")
(binary_expression operator: "&&" right: (binary_expression operator: "&&") @no_increment)
(binary_expression left: (binary_expression operator: "||") @no_increment operator: "||")
(binary_expression operator: "||" right: (binary_expression operator: "||") @no_increment)
(binary_expression left: (binary_expression operator: "??") @no_increment operator: "??")
(binary_expression operator: "??" right: (binary_expression operator: "??") @no_increment)

; --- jumps to a label ---

(break_statement (statement_identifier)) @increment.flat
(continue_statement (statement_identifier)) @increment.flat
