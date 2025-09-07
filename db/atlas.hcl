data "hcl_schema" "main" {
    paths = fileset("schema/**/*.hcl")
}

env "dev" {
    src = data.hcl_schema.main.url
}
