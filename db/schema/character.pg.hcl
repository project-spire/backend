enum "race" {
    schema = schema.public
    values = [
        "Human",
        "Orc",
    ]
}

composite "location" {
    schema = schema.public
    field "floor" {
        type = int2
    }
    field "id" {
        type = bigint
    }
}

table "character" {
    schema = schema.public

    column "id" {
        type = bigint
    }

    column "account_id" {
        type = bigint
        null = true
    }

    column "name" {
        type = varchar(16)
        null = true
    }

    column "race" {
        type = enum.race
        null = true
    }

    column "location" {
        type = composite.location
        null = true
    }

    primary_key {
        columns = [column.id]
    }
    
    foreign_key "dev_account_account_id_fkey" {
        columns = [column.account_id]
        ref_columns = [table.account.column.id]
        on_delete = CASCADE
    }
}
