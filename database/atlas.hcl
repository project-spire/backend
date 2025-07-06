data "hcl_schema" "main" {
    paths = fileset("schemas/**/*.hcl")
}

env "dev" {
    src = data.hcl_schema.main.url
}
