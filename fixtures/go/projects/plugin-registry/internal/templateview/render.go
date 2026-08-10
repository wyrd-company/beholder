package templateview

import (
	"bytes"
	"fmt"
	"text/template"

	"example.invalid/garden-ledger/pkg/registry"
)

// Origin is embedded in Model so templates can use promoted fields.
type Origin struct {
	Name string
}

// Model exposes fields and methods to a name-driven template.
type Model struct {
	Origin
	Label   string
	private string
}

// Describe is resolved by a template using a string member name.
func (m Model) Describe() string {
	return m.Label + " for " + m.Name
}

// Fail demonstrates a method whose error result stops template execution.
func (m Model) Fail() (string, error) {
	return "", fmt.Errorf("no view for %s", m.Name)
}

// Render parses dynamic template text with registered functions.
func Render(text string, model Model, functions map[string]any) (string, error) {
	parsed, err := template.New("almanac").Funcs(template.FuncMap(functions)).Parse(text)
	if err != nil {
		return "", err
	}
	var output bytes.Buffer
	if err := parsed.Execute(&output, model); err != nil {
		return "", err
	}
	return output.String(), nil
}

// Probe exercises successful and failed name-driven lookups.
func Probe(text string, model Model) (string, []error) {
	result, renderErr := Render(text, model, registry.TemplateFunctions())
	if renderErr != nil {
		return "", []error{renderErr}
	}

	var errors []error
	for _, missing := range []string{"{{.Missing}}", "{{.private}}", "{{.Fail}}"} {
		if _, err := Render(missing, model, registry.TemplateFunctions()); err != nil {
			errors = append(errors, err)
		}
	}
	return result, errors
}
