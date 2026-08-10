//go:build vettests

package vetfixtures

import "testing"

func TestWrongShape(t *testing.T, extra int) {
	t.Helper()
	_ = extra
}
