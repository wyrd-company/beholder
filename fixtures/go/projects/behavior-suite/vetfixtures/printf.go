//go:build vetfixtures

package vetfixtures

import "fmt"

func printfTarget() {
	fmt.Printf("%d", "wrong")
}
