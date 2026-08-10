package tags

import (
	"reflect"
	"strings"
)

// Origin is embedded so consumers can observe promoted fields.
type Origin struct {
	Name string `json:"name" yaml:"name" xml:"name" db:"origin_name"`
}

// Tagged uses several consumer grammars on one declaration.
type Tagged struct {
	Origin `json:",inline" yaml:",inline" xml:",inline" db:",inline"`
	Label  string `json:"label,omitempty" yaml:"title,omitempty" xml:"label,attr" db:"label"`
	Omit   string `json:"-" yaml:"-" xml:"-" db:"-"`
	hidden string
}

// Secondary gives Conflict a second promoted Name.
type Secondary struct {
	Name string `json:"secondary_name"`
}

// Conflict leaves the promoted Name ambiguous for reflection lookup.
type Conflict struct {
	Origin
	Secondary
}

// Report records tag syntax and identity observations.
type Report struct {
	LabelJSON             string
	LabelYAMLOptions      []string
	OmitJSON              string
	PromotedName          string
	PromotionFound        bool
	ConflictFound         bool
	HiddenFieldUnexported bool
	MalformedLookup       bool
	TagsChangeIdentity    bool
}

// Inspect reflects consumer tags and the structures they decorate.
func Inspect() Report {
	typeOfTagged := reflect.TypeFor[Tagged]()
	label, _ := typeOfTagged.FieldByName("Label")
	omit, _ := typeOfTagged.FieldByName("Omit")
	promoted, promotedFound := typeOfTagged.FieldByName("Name")
	hidden, _ := typeOfTagged.FieldByName("hidden")
	_, conflictFound := reflect.TypeFor[Conflict]().FieldByName("Name")

	malformed := reflect.StructTag(`json:"label"` + " broken")
	_, malformedFound := malformed.Lookup("json")

	withLabelTag := reflect.TypeOf(struct {
		Origin `json:",inline" yaml:",inline" xml:",inline" db:",inline"`
		Label  string `json:"label,omitempty" yaml:"title,omitempty" xml:"label,attr" db:"other"`
		Omit   string `json:"-" yaml:"-" xml:"-" db:"-"`
		hidden string
	}{})
	withoutLabelTag := reflect.TypeOf(struct {
		Origin `json:",inline" yaml:",inline" xml:",inline" db:",inline"`
		Label  string `json:"label,omitempty" yaml:"title,omitempty" xml:"label,attr" db:"label"`
		Omit   string `json:"-" yaml:"-" xml:"-" db:"-"`
		hidden string
	}{})

	return Report{
		LabelJSON:             label.Tag.Get("json"),
		LabelYAMLOptions:      strings.Split(label.Tag.Get("yaml"), ","),
		OmitJSON:              omit.Tag.Get("json"),
		PromotedName:          promoted.Name,
		PromotionFound:        promotedFound,
		ConflictFound:         !conflictFound,
		HiddenFieldUnexported: hidden.PkgPath != "",
		MalformedLookup:       malformedFound,
		TagsChangeIdentity:    withLabelTag != withoutLabelTag,
	}
}
