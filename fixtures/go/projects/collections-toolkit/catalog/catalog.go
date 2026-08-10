// Package catalog supplies cross-package types used by the collection helpers.
package catalog

import "fmt"

// Token is a defined string type. TokenAlias has the identical type identity.
type Token string

type TokenAlias = Token

// NamedInt is a defined integer type used for identity and conversion examples.
type NamedInt int

type NamedIntAlias = NamedInt

// Envelope is a generic value with a method owned by its defined type.
type Envelope[T any] struct {
	Value T
	Label string
}

// EnvelopeAlias is a generic alias. It adds no new method set of its own.
type EnvelopeAlias[T any] = Envelope[T]

func NewEnvelope[T any](value T, label string) Envelope[T] {
	return Envelope[T]{Value: value, Label: label}
}

func (e Envelope[T]) String() string {
	return fmt.Sprintf("%s:%v", e.Label, e.Value)
}

// PublicRecord demonstrates cross-package keyed and unkeyed literal rules.
type PublicRecord struct {
	Name   string
	Count  int
	hidden int
}

func NewPublicRecord(name string, count int) PublicRecord {
	return PublicRecord{Name: name, Count: count}
}

func (r PublicRecord) HiddenValue() int {
	return r.hidden
}

func (r PublicRecord) Read() string {
	return r.Name
}

// Base and Detail are embedded by toolkit records at different depths.
type Base struct {
	Name string
}

func (b Base) Describe() string {
	return b.Name
}

type Detail struct {
	Count int
}

func (d *Detail) Describe() string {
	if d == nil {
		return "missing"
	}
	return fmt.Sprintf("count:%d", d.Count)
}

// Reader is implemented implicitly by several values in this package.
type Reader interface {
	Read() string
}

// Sealed can only be implemented inside this package because marker is private.
type Sealed interface {
	marker()
}

type sealedRecord struct{}

func (sealedRecord) marker() {}

func NewSealedRecord() Sealed {
	return sealedRecord{}
}

// IntLike and OrderedLike are constraints consumed by the parent package.
type IntLike interface {
	~int | ~int32 | ~int64
}

type OrderedLike interface {
	~int | ~string
}
