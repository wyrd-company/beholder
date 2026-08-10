package main

import (
	"fmt"
	"os"

	"example.invalid/garden-ledger/internal/reflectkit"
	"example.invalid/garden-ledger/internal/runtimeapp"
	forecastschema "example.invalid/garden-ledger/internal/schema"
	"example.invalid/garden-ledger/internal/tags"
	"example.invalid/garden-ledger/internal/templateview"
	"example.invalid/garden-ledger/pkg/registry"
)

func main() {
	fieldName, methodName := "Name", "Describe"
	if len(os.Args) > 1 {
		fieldName = os.Args[1]
	}
	if len(os.Args) > 2 {
		methodName = os.Args[2]
	}

	reflection := reflectkit.Inspect(fieldName, methodName)
	tagReport := tags.Inspect()
	forecast := forecastschema.NewForecast("meadow", 18)
	reflectedName, _ := forecastschema.Reflect(forecast, "Name")

	_ = runtimeapp.RegisterManual("manual")
	provider, _ := runtimeapp.Select("morning")
	duplicate := runtimeapp.TryDuplicate("morning")

	templateText := "{{.Name}} / {{.Label}} / {{.Describe}} / {{decorate .Label}}"
	if len(os.Args) > 3 {
		templateText = os.Args[3]
	}
	view, templateErrors := templateview.Probe(templateText, templateview.Model{
		Origin: templateview.Origin{Name: "meadow"},
		Label:  "morning",
	})

	if pluginPath := os.Getenv("GARDEN_PLUGIN"); pluginPath != "" {
		pluginReport, err := runtimeapp.LoadPlugin(pluginPath)
		if err == nil {
			fmt.Println(pluginReport.Version, pluginReport.CallResult, pluginReport.DuplicateOpen, pluginReport.InitCount)
		}
	}

	fmt.Println(provider, reflectedName, reflection.MutationVisible, tagReport.LabelJSON, view, duplicate, len(templateErrors), registry.Names())
}

func commandName() string {
	return "morning"
}
