//go:build amd64

#include "textflag.h"

// Add is paired with the bodyless declaration in add.go.
TEXT ·Add(SB), NOSPLIT, $0-24
	MOVQ a+0(FP), AX
	ADDQ b+8(FP), AX
	MOVQ AX, ret+16(FP)
	RET
