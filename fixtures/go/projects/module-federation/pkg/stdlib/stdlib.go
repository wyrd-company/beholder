package stdlib

import (
	legacyrand "math/rand"
	randv2 "math/rand/v2"
)

// Samples uses two standard-library packages with the same declared package name.
func Samples() (int, uint64) {
	legacy := legacyrand.New(legacyrand.NewSource(7)).Intn(10)
	modern := randv2.Uint64N(10)
	return legacy, modern
}
