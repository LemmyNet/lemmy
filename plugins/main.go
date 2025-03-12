package main

import (
	"github.com/extism/go-pdk"
	"errors"
)

//go:wasmexport create_local_post
func api_before_post_post() int32 {
	params := make(map[string]interface{})
	// use json input helper, which automatically unmarshals the plugin input into your struct
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

	// use json output helper, which automatically marshals your struct to the plugin output
	err = pdk.OutputJSON(params)
	if err != nil {
		pdk.SetError(err)
		return 1
	}
	return 0
}

func main() {}