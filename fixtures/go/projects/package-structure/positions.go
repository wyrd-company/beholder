package pantry

// ---
// relationships:
//   references:
//     - coverage
// ---

// line logical-source.go:17
const LogicalDirectiveValue = 17

/*line block-source.go:29*/
const BlockDirectiveValue = 29

// line near-miss-source.go:41
const NearMissCommentValue = 41

const (
	DecimalLiteral      = 1_024
	BinaryLiteral       = 0b1010
	HexLiteral          = 0xCAFE
	OctalLiteral        = 0o755
	LegacyOctalLiteral  = 0755
	FloatLiteral        = 6.022e23
	DigitSeparatedFloat = 1_000.25
)

func automaticSemicolons() int {
	value := DecimalLiteral
	value++
	if value > 0 {
		return value
	}
	return 0
}

func numericLiterals() float64 {
	return FloatLiteral + DigitSeparatedFloat + float64(BinaryLiteral+HexLiteral+OctalLiteral+LegacyOctalLiteral)
}
