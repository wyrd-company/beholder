//go:build vetfixtures

package vetfixtures

import "sync"

type lockedValue struct {
	sync.Mutex
}

func copyLockTarget(value lockedValue) lockedValue {
	return value
}
