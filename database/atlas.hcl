data "hcl_schema" "main" {
    paths = fileset("schemas/**/*.hcl")
}

env "local" {
    src = data.hcl_schema.main.url
}
