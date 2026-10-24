
GOAL: create full flow from structuralize (which is last point in `retailer-sourcing` app implement)

[.] Upgrade `retailer-sourcing` to be using local `mulac` and utilize new changes (examples can be seen on `offers` service) ([plan](docs/plans/01-retailer-sourcing-local-mulac.md))
[.] Start setting up `customer-facing` service by setting up `mulac's` queue runners and health endpoint ([plan](docs/plans/02-customer-facing-queue-runners.md))
[.] Add flow for converting data from `structurize` to `offer_export`: name it `processing for export` ([plan](docs/plans/03-processing-for-export.md))
[ ] Add listener on `offer_export` event and write down message into `outbox`
[.] For now, `outbox publisher` on `retailer-sourcing` should use `offers` api ({offers}/api/inbox) ([plan](docs/plans/05-retailer-sourcing-http-outbox.md))
[ ] For now, `outbox publisher` on `offers` should use `customer-facing` api ({customer-facing}/api/inbox)
[ ] Create `offers` entity and record on `customer-facing` that has same properites as `read_offers` in `offers` service
[ ] Add `offers` api (both rest and graphql) to retrieve limited number of offers with offset
[ ] Set up `customers` app in a way that on homepage shows 30 offers, using nice design and built using only html (no javascript ATM)



NOTES:
- We are aware that `For now` is going against ADR-16, but this is shortterm
- depending on local mulac will be needed in this phase and possibly next ones
- `customer-facing` will take given events and persist them into `offers` table
