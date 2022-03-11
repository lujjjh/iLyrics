package main

import (
	"log"

	"github.com/lujjjh/ilyrics/app"
)

func main() {
	a, err := app.New()
	if err != nil {
		log.Fatalf("app.New: %v", err)
	}
	a.Run()
}
