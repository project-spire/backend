# Link & Connect

```sh
# 1. Link to the docker container
gel instance link --tls-security insecure --docker --container <container name> <instance name>

# 2. Init the Gel project
gel init --link --server-instance <instance name>
```

# Migrate

```sh
# a. Sync all
gel migrate --dev-mode

# b. Interactive sync
gel watch --migrate
```
