# DB

## Local Development Setup

1. Install `pgschema` following the [instruction](https://www.pgschema.com/installation).

2. Install `Diesel CLI` following the [instruction](https://diesel.rs/guides/getting-started).

3. Run DB Container and apply DB schema following the [Schema Modification](#schema-modification) below.

## Schema Modification

1. Plan a schema migration with `pgschema` using [plan.local.sh](plan.local.sh) script.

```shell
./plan.local.sh
```

2. Apply a schema migration with `pgschema` using [apply.local.sh](apply.local.sh) script.

```shell
./apply.local.sh
```

