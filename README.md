# Spire Backend

## Local Development Setup

1. Install `protoc`.

2. Run `db` service and prepare schema following [db/README.md](db/README.md)

3. Run [secrets/generate.sh](secrets/generate.sh) to generate secret keys. (Change the password in `secrets/db-password.key` if needed.)

4. Run containers with local environment. Use [compose.local.sh](compose.local.sh) for easy command. For example,
```sh
./compose.local.sh up -d db lobby-server
```
