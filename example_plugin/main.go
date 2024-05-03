package main

import (
	"github.com/extism/go-pdk"
)

type CreatePost struct {
	Name string `json:"name"`
	Body *string `json:"body,omitempty"`
	Community_id int32 `json:"community_id"`
	Url *string `json:"url,omitempty"`
	Alt_text *string `json:"alt_text,omitempty"`
	Honeypot *string `json:"honeypot,omitempty"`
	Nsfw *bool `json:"nsfw,omitempty"`
	Language_id *int32 `json:"language_id,omitempty"`
	Custom_thumbnail *string `json:"custom_thumbnail,omitempty"`
  }

//export api_before_create_post
func api_before_create_post() int32 {
	params := CreatePost{}
	// use json input helper, which automatically unmarshals the plugin input into your struct
	err := pdk.InputJSON(&params)
	if err != nil {
		pdk.SetError(err)
		return 1
	}
	if params.Name == "foobar" {
		params.Name = "Hello plugin!"
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
