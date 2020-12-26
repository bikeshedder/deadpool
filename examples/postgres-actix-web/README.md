# deadpool-postgres + actix-web example

This example combines `deadpool-postgres` with a `actix-web` webservice to
implement a simple API service that responds with JSON read from
PostgreSQL.

# Incompatibility

> **The `actix-web` crate currently is not compatible with `tokio 1.0`.
`deadpool-postgres` and `deadpool` have both been upgraded to use the
latest version of `tokio` and `tokio-postgres`. Because of that this
example only works with `deadpool-postgres 0.5`.**

https://github.com/bikeshedder/deadpool/issues/72

## Running the example

The following instructions assumes that your current user can access the
PostgreSQL running at local host passwordless via unix domain socket. The
default installation of PostgreSQL usually already contains the following line
in its [pg_hba.conf](https://www.postgresql.org/docs/12/auth-pg-hba-conf.html):

```txt
local all all peer
```

All you need to do is to create a PostgreSQL user with the same name as
your system user:

```shell
sudo -u postgres createuser -s my_user_name
```

Now create a database

```shell
createdb deadpool
```

Now you should be able to connect to the newly created database without
without any options:

```shell
psql deadpool
```

Load example data

```shell
psql -f fixture.sql deadpool
```

Create `.env` file in this directory

```env
LISTEN=[::1]:8000
PG__DBNAME=deadpool
```

Run the example

```shell
cargo run --release
```

## Configuration options

If you want to connect to your database using a TCP/IP socket you can use
the following template for your `.env` file:

```env
PG__HOST=127.0.0.1
PG__PORT=5432
PG__USER=deadpool
PG__PASSWORD=somepassword
PG__DBNAME=deadpool
```

For more configuration options see `deadpool_postgres::Config`.
