//go:build vetfixtures

package vetfixtures

import "context"

func lostCancelTarget() {
	ctx, cancel := context.WithCancel(context.Background())
	if ctx == nil {
		return
	}
	if false {
		cancel()
	}
}
