package codegenassets

import "embed"

//go:embed assets/greetings.txt
var Greeting string

//go:embed assets/data.json
var Data []byte

//go:embed assets/*.txt
var PatternFS embed.FS

//go:embed all:assets
var AllFS embed.FS

//go:embed assets/sub/nested.txt
var SubFS embed.FS

// GreetingContent returns the embedded greeting file content.
func GreetingContent() string {
	return Greeting
}

// DataContent returns the embedded JSON bytes.
func DataContent() []byte {
	return Data
}
