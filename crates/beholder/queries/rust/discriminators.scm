; Identity discriminators.
;
; Two symbols can share a qualified path and still be two different symbols. In
; Rust that is conditional compilation: `#[cfg(unix)] fn platform` and
; `#[cfg(windows)] fn platform` are both `platform`.
;
; A discriminator is declared, content-derived metadata that tells them apart.
; It becomes part of identity whenever it is present, never only when a
; duplicate happens to exist — otherwise deleting one variant would silently
; rewrite the other's identity.
;
; Every discriminator node attaches to the symbol that follows it. Stacked
; discriminators all attach, outermost first.
;
; `require.<capture>` constrains a capture's text, so this matches `cfg` and not
; `#[inline]` or `#[derive(Debug)]` — attributes that say nothing about which
; symbol this is.

((attribute_item (attribute (identifier) @guard) @text) @discriminator
 (#set! require.guard "cfg")
 (#set! format "{text}"))

((attribute_item (attribute (identifier) @guard) @text) @discriminator
 (#set! require.guard "cfg_attr")
 (#set! format "{text}"))
