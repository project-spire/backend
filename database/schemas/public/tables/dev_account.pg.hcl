table "dev_account" {
    schema = schema.public
    column "id" {
        type = varchar(16)
    }
    column "account_id" {
        type = bigint
    }
    primary_key {
        columns = [column.id]
    }
    foreign_key "dev_account_fk" {
        columns = [column.account_id]
        ref_columns = [table.account.column.id]
    }
}
