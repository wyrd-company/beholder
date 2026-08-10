---
relationships:
  implements:
    - fixtures/go/plan/projects/plugin-registry
  references:
    - reports/go/synthesis/checklist
---

# Plugin registry coverage

| Canonical identifier | Stable source path | Named coverage | Build context | Additional locators |
| --- | --- | --- | --- | --- |
| GO-CAN-DEC-004 | `cmd/morning/main.go` | `package main`, `func main`, and shared `runtimeapp` and `registry` packages | default | `cmd/evening/main.go:main`; `plugins/seasonal/main.go:main`; legal main-package test `cmd/morning/main_test.go:TestMainPackageIsTestable` |
| GO-CAN-DYN-001 | `internal/reflectkit/reflectkit.go` | `Inspect` uses `Type`, `Value`, `TypeFor`, `New`, dynamic `FieldByName`, dynamic `MethodByName`, conversion, interface tests, and mutation | default | `Subject.Name` is exported, `Subject.secret` is unexported, pointer `Elem` is addressable and settable, missing and wrong-kind paths are reported, and `recoverMisuse` captures a valid panic-producing misuse |
| GO-CAN-DYN-002 | `internal/tags/tags.go` | `Tagged` carries `json`, `yaml`, `xml`, and `db` consumer tags; `Inspect` reflects rename, omit, promotion, conflict, unexported fields, malformed runtime syntax, and tag-dependent type identity | default | `Origin` is embedded for promotion; `Conflict` demonstrates an ambiguous promoted name; `reflect.StructTag` receives a malformed runtime string without invalid Go source |
| GO-CAN-DYN-003 | `internal/providers/core/register.go` | `init` registers two providers and `runtimeapp.Select` chooses by runtime key | default | `internal/providers/tagged/register_tagged.go:init` is enabled by blank import plus `tagprovider`; `runtimeapp.RegisterManual` is the explicit-registration contrast; missing and duplicate keys are surfaced |
| GO-CAN-DYN-004 | `plugins/seasonal/main.go` | Plugin `init`, function `Provide`, and variable `Version` are exported for string lookup | `plugin` on Linux amd64 with `CGO_ENABLED=1` | `internal/pluginloader/loader_linux_amd64.go:Load` performs missing lookup, wrong assertion, duplicate open, and init-once observation; `runtimeapp.LoadPlugin` is the host entry point |
| GO-CAN-DYN-007 | `schema/forecast.json` | Source schema paired with generated `internal/schema/generated.go` binding and descriptor registry | default | `internal/schemagen/main.go` is the checked-in generator; generated `GeneratorVersion` and `SourceSchema` preserve provenance; `forecastschema.NewForecast` and `forecastschema.Reflect` are used by commands |
| GO-CAN-DYN-008 | `internal/templateview/render.go` | Dynamic `text/template` resolves exported fields, methods, registered functions, embedded promotion, and method errors | default | `Probe` also renders missing and unexported members; `cmd/morning/main.go` supplies dynamic template text and `registry.TemplateFunctions()` |
