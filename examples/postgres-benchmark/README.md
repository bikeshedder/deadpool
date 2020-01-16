# deadpool-postgres benchmark

This example compares 16,000 queries run with 16 workers in
parallel. The first implementation creates 16.000 separate
connections to the database while the second one uses `deadpool-postgres`.

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
PG_DBNAME=deadpool
```

Run the example

```shell
cargo run --release
```

## Configuration options

If you want to connect to your database using a TCP/IP socket you can use
the following template for your `.env` file:

```env
PG_HOST=127.0.0.1
PG_PORT=5432
PG_USER=deadpool
PG_PASSWORD=somepassword
PG_DBNAME=deadpool
```

For more configuration options see `deadpool_postgres::Config`.
