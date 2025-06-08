package main

import (
	"flag"
	"github.com/santhosh-tekuri/jsonschema/v6"
	"log"
	"os"
)

func main() {
	root := flag.String("root", "", "path to data root directory")
	log.Printf("root: %s", *root)

	c := jsonschema.NewCompiler()
	dataSchema, err := c.Compile(*root + "/data.schema.json")
	if err != nil {
		log.Fatal(err)
	}

	f, err := os.Open(*root + "/data.json")
	if err != nil {
		log.Fatal(err)
	}
	data, err := jsonschema.UnmarshalJSON(f)
	if err != nil {
		log.Fatal(err)
	}

	err = dataSchema.Validate(data)
	if err != nil {
		log.Fatal(err)
	}
}
