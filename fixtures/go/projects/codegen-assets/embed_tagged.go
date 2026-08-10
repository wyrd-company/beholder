//go:build taggedbuild

package codegenassets

import "embed"

//go:embed assets/tagged.txt
var TaggedFile string

//go:embed assets/tagged.txt
var TaggedFS embed.FS
