table "character" {
    schema = schema.public

    column "id" {
        type = bigint
    }

    column "account_id" {
        type = bigint
    }

    column "name" {
        type = varchar(16)
    }

    column "race" {
        type = enum.race
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
