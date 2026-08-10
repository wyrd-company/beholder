package toolkit

import (
	"reflect"
	"strconv"
)

// Untyped constants retain arbitrary precision until a context supplies a type.
const (
	BoolConstant    = true
	RuneConstant    = 'R'
	IntegerConstant = 42
	FloatConstant   = 2.5
	ComplexConstant = 2.5 + 3i
	StringConstant  = "north"
)

// ConstantDefaults records the default types selected by assignment context.
func ConstantDefaults() (bool, rune, int, float64, complex128, string) {
	var boolean = BoolConstant
	var character = RuneConstant
	var integer = IntegerConstant
	var decimal = FloatConstant
	var complexNumber = ComplexConstant
	var text = StringConstant
	return boolean, character, integer, decimal, complexNumber, text
}

func ConstantContexts() (int64, int64, bool, int) {
	const shift = 1 << 5
	var wide int64 = IntegerConstant
	var shifted int64 = shift
	comparison := IntegerConstant < 100
	return wide, shifted, comparison, inferConstant(IntegerConstant)
}

func inferConstant[T ~int](value T) int {
	return int(value)
}

// IotaNames includes explicit lists, omitted lists, multiple values, shifts,
// and a blank identifier. The second declaration demonstrates reset behavior.
const (
	IotaZero, IotaLabel = iota, "zero"
	IotaOne, _          // implicit repetition of both expressions
	IotaTwo, IotaWord   = iota, "two"
	IotaGap, _          = iota, "gap"
)

const (
	ShiftZero = 1 << iota
	ShiftOne
	ShiftTwo = 1 << iota
)

const ResetIota = iota

// NamedCount is distinct from int, while CountAlias preserves its identity.
type NamedCount int

type CountAlias = NamedCount

func (count NamedCount) String() string {
	return strconv.Itoa(int(count))
}

func DefinedAndAlias() (NamedCount, CountAlias, bool, bool) {
	defined := NamedCount(7)
	aliased := CountAlias(defined)
	assignable := aliased == defined
	converted := NamedCount(int(aliased)) == defined
	return defined, aliased, assignable, converted
}

func ReflectionIdentity() (string, string) {
	defined := reflect.TypeOf(NamedCount(0))
	aliased := reflect.TypeOf(CountAlias(0))
	return defined.String(), aliased.String()
}
