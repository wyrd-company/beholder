package toolkit

// CoreRangeForms covers the standard range sources and all variable counts.
func CoreRangeForms(array [2]string, pointer *[2]string, slice []string, text string, values map[string]int, channel <-chan string) int {
	total := 0
	for range array {
		total++
	}
	for index := range pointer {
		total += index
	}
	for index, value := range slice {
		total += index + len(value)
	}
	for _, value := range text {
		total += int(value)
	}
	for key := range values {
		total += len(key)
	}
	for _, value := range values {
		total += value
	}
	for value := range channel {
		total += len(value)
	}
	return total
}

// ForForms places infinite, condition, and three-clause loops beside range.
func ForForms(limit int) int {
	total := 0
	for {
		total++
		break
	}
	for total < limit {
		total++
	}
	for index := 0; index < limit; index++ {
		total += index
	}
	return total
}

// SwitchForms combines tagged, tagless, and type switches with an init scope.
func SwitchForms(value any) string {
	result := "default"
	switch number := len(result); number {
	case 0:
		result = "empty"
	case 7:
		result = "default"
	default:
		result = "length"
	}

	switch {
	case result == "length":
		result = "tagless"
	case result == "empty":
		result = "empty"
	}

	switch typed := value.(type) {
	case string:
		result += typed
	case int, int64:
		result += "number"
	case Namer:
		result += typed.Name()
	case nil:
		result += "nil"
	default:
		result += "other"
	}
	return result
}

func FallthroughForms(value int) string {
	label := "default"
	switch value {
	case 0:
		label = "zero"
		fallthrough
	case 1:
		label += ":small"
	default:
		label += ":other"
	}
	return label
}

// InitScopeShadowing reuses names in the init, condition, clause, and outer
// blocks without extending any one declaration beyond its legal scope.
func InitScopeShadowing(input int) int {
	value := input
	if value := value + 1; value > 0 {
		value++
	}
	switch value := value; {
	case value > 0:
		value--
	default:
		value++
	}
	return value
}

// BranchingForms has labeled and unlabeled loop exits, continue, goto, and
// switch-only fallthrough in separate legal constructs.
func BranchingForms(limit int) int {
	total := 0
outer:
	for row := 0; row < limit; row++ {
		for column := 0; column < limit; column++ {
			if column == 0 {
				continue
			}
			if row+column > limit {
				break outer
			}
			total++
		}
	}
	if total == 0 {
		goto finish
	}
	total++
finish:
	return total
}

// DeferredMutation uses named results, immediately evaluated arguments, and
// last-in-first-out deferred calls.
func DeferredMutation() (result int) {
	defer func() { result++ }()
	defer func(value int) { result += value }(result)
	result = 1
	return
}

type deferredCounter struct {
	value int
}

func (counter *deferredCounter) Add(value int) {
	counter.value += value
}

func DeferredMethodsAndClosures() int {
	counter := &deferredCounter{}
	defer counter.Add(2)
	defer func(value int) { counter.value += value }(3)
	return counter.value
}

func LoopDefers(limit int) int {
	result := 0
	for index := 0; index < limit; index++ {
		defer func(value int) { result += value }(index)
	}
	return result
}

func BuiltinOperations(values []int, complexValue complex128) (int, int, int, int, int, int, complex128) {
	allocated := new(int)
	*allocated = 1
	result := append(values, 2)
	result = append(result, []int{3, 4}...)
	copied := make([]int, len(result))
	copy(copied, result)
	delete(map[string]int{"north": 1}, "north")
	clear(copied)
	minimum := min(3, 4)
	maximum := max(3, 4)
	return *allocated, len(result), cap(result), len(copied), minimum, maximum, complex(real(complexValue), imag(complexValue))
}

func PrintFamilies(value any) {
	print(value)
	println(value)
}

func CloseChannel(channel chan int) {
	close(channel)
}

func panicValue() any {
	return recover()
}

// recoverPanic is deferred on the goroutine that executes operation.
func recoverPanic(operation func()) (value any) {
	defer func() {
		value = recover()
	}()
	operation()
	return nil
}

func indirectRecover() any {
	return recover()
}

func IndirectRecovery() (value any) {
	defer func() { value = indirectRecover() }()
	panic("indirect")
}

func NestedRecovery() (value any) {
	defer func() { value = recover() }()
	defer func() { panic("nested") }()
	panic("outer")
}

func Repanic() {
	defer func() {
		if recovered := recover(); recovered != nil {
			panic(recovered)
		}
	}()
	panic("initial")
}

func PanicNilBoundary() {
	panic(nil)
}

// RuntimePanicExamples isolates nil, bounds, assertion, divide, closed-channel,
// map, explicit, and panic(nil) operations behind individual recover wrappers.
func RuntimePanicExamples() []any {
	closed := make(chan int)
	close(closed)
	operations := []func(){
		func() { var pointer *int; _ = *pointer },
		func() { values := []int{1}; _ = values[2] },
		func() { var value any = "text"; _ = value.(int) },
		func() { denominator := 0; _ = 1 / denominator },
		func() { closed <- 1 },
		func() { var values map[string]int; values["north"] = 1 },
		func() { panic("explicit") },
		PanicNilBoundary,
	}
	result := make([]any, 0, len(operations))
	for _, operation := range operations {
		result = append(result, recoverPanic(operation))
	}
	return result
}

func PanicValueReference() any {
	return panicValue()
}
