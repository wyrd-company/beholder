; Import declarations.
;
; Go names a package by its path and binds its last segment, optionally under an
; alias. Nothing nests, so the captures are read directly rather than expanded:
; see ImportStyle::NamedFrom.

((import_spec name: (package_identifier) @alias path: (_) @source) @import)

((import_spec !name path: (_) @source) @import)
