package parcel

import (
	"fmt"
	"os"
	"strings"
	"sync"
	"testing"
)

var mainStarted bool

func TestMain(m *testing.M) {
	mainStarted = true
	os.Exit(m.Run())
}

func TestPackageInitialization(t *testing.T) {
	if !mainStarted {
		t.Fatal("test main did not start")
	}
	if !packageInitialized() {
		t.Fatal("package initializer did not run")
	}
	if !strings.HasPrefix(fmt.Sprint(defaultStart), "2") {
		t.Fatalf("unexpected unexported default: %d", defaultStart)
	}
}

func TestUnexportedCounter(t *testing.T) {
	counter := seededCounter()
	counter.Increment()
	if got, want := counter.Value(), 3; got != want {
		t.Fatalf("counter value = %d, want %d", got, want)
	}
}

func TestRecursiveSum(t *testing.T) {
	chain := &Node{Value: 1, Next: &Node{Value: 2, Next: &Node{Value: 3}}}
	if got, want := Sum(chain), 6; got != want {
		t.Fatalf("sum = %d, want %d", got, want)
	}
}

func TestSubtestLifecycle(t *testing.T) {
	items := []string{"red", "blue"}
	t.Run("batch", func(t *testing.T) {
		t.Cleanup(func() { t.Log("batch cleanup") })
		for _, item := range items {
			item := item
			t.Run(fmt.Sprintf("item/%s", item), func(t *testing.T) {
				t.Cleanup(func() { t.Logf("cleanup %s", item) })
				if item == "blue" {
					t.Parallel()
				}
				if item == "red" && Add(1, 1) != 2 {
					t.Fatal("unexpected red item result")
				}
			})
		}
	})
}

func TestParallelSubtestNames(t *testing.T) {
	cases := []struct {
		name string
		id   int
	}{
		{name: "batch item", id: 1},
		{name: "batch item", id: 2},
	}
	started := make(chan int, len(cases))
	release := make(chan struct{})
	done := make(chan struct{})
	var orderMu sync.Mutex
	order := make([]int, 0, len(cases))

	go func() {
		for range cases {
			id := <-started
			orderMu.Lock()
			order = append(order, id)
			orderMu.Unlock()
		}
		close(release)
		close(done)
	}()

	for _, testCase := range cases {
		testCase := testCase
		t.Run(testCase.name, func(t *testing.T) {
			t.Parallel()
			started <- testCase.id
			<-release
		})
	}

	t.Cleanup(func() {
		<-done
		orderMu.Lock()
		defer orderMu.Unlock()
		if len(order) != len(cases) {
			t.Fatalf("started %d parallel subtests, want %d", len(order), len(cases))
		}
		if order[0] == order[1] {
			t.Fatalf("parallel subtests reported duplicate identities: %v", order)
		}
		t.Logf("parallel start order: %v", order)
	})
}

func BenchmarkCounter(b *testing.B) {
	b.Run("increment", func(b *testing.B) {
		for i := 0; i < b.N; i++ {
			counter := NewCounter(i)
			counter.Increment()
		}
	})
	b.Run("snapshot", func(b *testing.B) {
		counter := NewCounter(1)
		b.ResetTimer()
		for i := 0; i < b.N; i++ {
			_ = counter.Value()
		}
	})
}

func TestPackageRelativeTestdata(t *testing.T) {
	notes, err := os.ReadFile("testdata/notes.txt")
	if err != nil {
		t.Fatal(err)
	}
	if !strings.Contains(string(notes), "seasonal") {
		t.Fatalf("notes = %q", notes)
	}
}

func testHelper(value int) int {
	return value * 2
}
