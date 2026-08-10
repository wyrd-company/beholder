package main

import (
	"fmt"
	"os"

	"example.invalid/garden-ledger/internal/runtimeapp"
	forecastschema "example.invalid/garden-ledger/internal/schema"
)

func main() {
	key := "evening"
	if len(os.Args) > 1 {
		key = os.Args[1]
	}
	provider, err := runtimeapp.Select(key)
	forecast := forecastschema.NewForecast("ridge", 12)
	name, _ := forecastschema.Reflect(forecast, "Name")
	if err != nil {
		fmt.Println(err, name)
		return
	}
	fmt.Println(provider, name)
}
