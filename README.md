# Spire Backend

## Local Development Setup

* Add the following host resolutions to `/etc/hosts`:

```
::1       db.spire.dev
127.0.0.1 db.spire.dev

::1       game.spire.dev
127.0.0.1 game.spire.dev

::1       lobby.spire.dev
127.0.0.1 lobby.spire.dev
```

* Run [secrets/generate.sh](secrets/generate.sh) to generate secret keys. (Change the password in `secrets/db-password.key` if needed.)
