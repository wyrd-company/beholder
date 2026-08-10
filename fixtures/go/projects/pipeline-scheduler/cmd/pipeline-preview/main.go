package main

import (
	"context"
	"fmt"

	"example.com/pipeline-scheduler/pipeline"
)

func main() {
	registry := &pipeline.CallbackRegistry{}
	registry.Add(func(name string) { fmt.Println("completed", name) })
	scheduler := pipeline.NewScheduler(2, registry)
	jobs := []pipeline.Job{
		{Name: "north", Run: func(context.Context) error { return nil }},
		{Name: "south", Run: func(context.Context) error { return nil }},
	}
	completed, err := scheduler.Run(context.Background(), jobs)
	if err != nil {
		panic(err)
	}
	fmt.Println(completed)
}
