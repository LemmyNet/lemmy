package main

import (
	"github.com/extism/go-pdk"
	"errors"
)
type Metadata struct {
	Name string `json:"name"`
	Url string `json:"url"`
	Description string `json:"description"`
}

//go:wasmexport metadata
func metadata() int32 {
	metadata := Metadata {
		Name: "Test Plugin",
		Url: "https://example.com",
		Description: "Plugin to test Lemmy feature",
	}
	err := pdk.OutputJSON(metadata)
	if err != nil {
		pdk.SetError(err)
		return 1
	}
	return 0
}

//go:wasmexport create_local_post
func create_local_post() int32 {
	params := make(map[string]interface{})
	err := pdk.InputJSON(&params)
	if err != nil {
		pdk.SetError(err)
		return 1
	}
	if params["name"] == "foo" {
		params["name"] = "bar"
	}
	if params["name"] == "blocked" {
		pdk.SetError(errors.New("blocked"))
		return 1
	}

	err = pdk.OutputJSON(params)
	if err != nil {
		pdk.SetError(err)
		return 1
	}
	return 0
}

func main() {}