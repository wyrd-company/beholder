//go:build !cgo

package bridge

// ForeignWord is the pure-Go counterpart of the C struct path.
func ForeignWord(value uint32) uint32 { return value }

// ForeignSum is the pure-Go counterpart of the transient C call.
func ForeignSum(data []byte) uint32 {
	var total uint32
	for _, value := range data {
		total += uint32(value)
	}
	return total
}

// ForeignCopy is the pure-Go counterpart of the C allocation path.
func ForeignCopy(data []byte) []byte { return append([]byte(nil), data...) }

// InvokeCallback is the pure-Go counterpart of the C callback.
func InvokeCallback() string { return "callback-pure-go" }
