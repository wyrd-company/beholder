; Identity discriminators.
;
; Go has none. Conditional compilation is per file — a `//go:build` line governs
; the whole file, and two build-tagged variants of a function live in different
; files, so the repo-relative path already tells them apart.
;
; The file exists because the language table requires one, and an empty set of
; patterns is the honest way to say "nothing here discriminates".
