package main

import (
	"github.com/extism/go-pdk"
	"errors"
)

type CreatePost struct {
	Name string `json:"name"`
	Body string `json:"body"`
	// skipping other fields for now
  }

//export api_create_post
func api_create_post() int32 {
	params := CreatePost{}
	// use json input helper, which automatically unmarshals the plugin input into your struct
	err := pdk.InputJSON(&params)
	if err != nil {
		pdk.SetError(err)
		return 1
	}
	if params.Body == "plugin should block this" {
		pdk.SetError(errors.New("blocked by plugin"))
		return 1
	}
	greeting := `Created post "` + params.Name + `"!`
	pdk.OutputString(greeting)
	return 0
}

func main() {}
