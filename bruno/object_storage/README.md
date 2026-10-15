# Object storage requests

Open this folder as a Bruno collection and select `local`. It targets
`http://localhost:3003`, matching `PORT=3003 make serve` in
[`services/object_storage`](../../services/object_storage/README.md).
All requests inherit the collection's bearer authentication. The local
environment contains the service's public development token.

Run the collection in sequence. REST and GraphQL creation save `recordId` and
`graphqlRecordId` as runtime variables for later requests. The GraphQL retry
also retries the REST-created record, checking that both APIs share storage.
Health, OpenAPI, Swagger UI, and debug-only GraphiQL requests are included.

The environment contains two fixed UUIDv7 keys. Repeating a collection run
reuses its existing artifacts. To test fresh creations, set `idempotencyKey`
and `graphqlIdempotencyKey` to new UUIDv7 values. Changing content with the
same bucket/key produces a conflict. The APIs call the key `idempotency_key`
(REST) and `idempotencyKey` (GraphQL); its database column is `external_id`.

## Run the JavaScript request tests

Requires Node.js and [Bruno CLI](https://github.com/usebruno/bruno/blob/main/packages/bruno-cli/readme.md).
The `.bru` files include JavaScript assertions and response scripts; these run
in Bruno's default safe runtime without external modules.

```sh
npm install -g @usebruno/cli@4.2.1
cd bruno/object_storage
bru run --env local
# Override the address for a separately started service:
bru run --env local --env-var baseUrl=http://localhost:3057
```

Run against a development or disposable database. A successful first run stores
two immutable artifacts; there is no API delete operation. GraphiQL's test
accepts 200 in debug builds and authenticated 404 in release builds. To run a
retrieval on its own, first create the record or set its ID in the environment.
