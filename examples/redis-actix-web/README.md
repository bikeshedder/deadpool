# `deadpool-redis` + `actix-web` example

This example combines `deadpool-redis` with a `actix-web` webservice to
implement a simple API service that responds to a PING Redis query.

## Configuration options

The code assumes that your current user can access Redis running at redis://127.0.0.1:6379.
You can override the Redis connection url by setting the `REDIS_URL` ENV variable.
For more configuration options see `deadpool_redis::Config`.
