enum "platform" {
    schema = schema.public
    values = [
        "Dev",
        "Steam"
    ]
}

enum "privilege" {
    schema = schema.public
    values = [
        "None",
        "Manager"
    ]
}

table "account" {
    schema = schema.public
    column "id" {
        type = bigint
    }
    column "platform" {
        type = enum.platform
    }
    column "platform_id" {
        type = bigint
    }
    column "privilege" {
        type = enum.privilege
        default = "None"
    }
    column "created" {
        type = timestamptz
        default = "now()"
    }
    primary_key {
        columns = [column.id]
    }
}
