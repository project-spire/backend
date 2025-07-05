enum "race" {
    schema = schema.public
    values = [
        "Human",
        "Barbarian"
    ]
}

table "character" {
    schema = schema.public
    column "id" {
        type = bigint
    }
    column "account_id" {
        type = bigint
        null = true
        default = null
    }
    column "name" {
        type = varchar(24)
    }
    column "race" {
        type = enum.race
    }
    column "created" {
        type = timestamptz
        default = "now()"
    }
    primary_key {
        columns = [column.id]
    }
    foreign_key "character_fk" {
        columns = [column.account_id]
        ref_columns = [table.account.column.id]
    }
}
