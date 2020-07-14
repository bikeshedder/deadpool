# deadpool-postgres + hyper example

This example combines `deadpool-postgres` with a `hyper` webservice to
implement a simple API service that responds with JSON read from
PostgreSQL.

## Prerequisites

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

Now you should be able to run `psql` without any options and connect to
your local running PostgreSQL instance. e.g. by connecting to `template1`:

```shell
psql template1
```

## How to run the example

1. Create a database

    ```shell
    createdb deadpool
    ```

2. Load example data

    ```shell
    psql -f fixture.sql deadpool
    ```

3. Create `.env` file in this directory

    ```env
    LISTEN=[::1]:8000
    PG__DBNAME=deadpool
    ```

4. Run the example

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
