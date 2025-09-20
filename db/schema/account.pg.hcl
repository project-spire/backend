table "account" {
    schema = schema.public
    
    column "id" {
        type = bigint
    }

    column "created" {
        type = timestamptz
        default = sql("now()")
    }

    primary_key {
        columns = [column.id]
    }
}
