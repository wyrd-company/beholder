package federation

import (
	graphroot "example.invalid/graph-root"
	"example.invalid/module-federation/internal/visibility"
	"example.invalid/nested/feature"
	consumer "example.invalid/replacement-consumer"
	"example.invalid/retracted/notice"
	semver "example.invalid/semver/label"
	semverv2 "example.invalid/semver/v2/label"
)

// Snapshot gathers values from modules selected by the main module's graph.
type Snapshot struct {
	GraphVersion string
	V1Name       string
	V2Name       string
	Redirect     string
	Retraction   string
	Nested       string
	Internal     string
}

// SnapshotValues keeps module metadata and source imports visibly distinct.
func SnapshotValues() Snapshot {
	return Snapshot{
		GraphVersion: graphroot.Version(),
		V1Name:       semver.Name(),
		V2Name:       semverv2.Name(),
		Redirect:     consumer.Value(),
		Retraction:   notice.Status(),
		Nested:       feature.Name(),
		Internal:     visibility.PublicMarker(),
	}
}
