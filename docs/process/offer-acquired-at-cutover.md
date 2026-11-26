# Offer acquisition time cutover

The operator runs this reset by hand against the service databases when
deploying [ADR 91](../adr/0091-offer-acquired-at.md). The new code reads only
`acquired_at`, `OfferCreated`, `OfferAcquiredAtUpdated` and unversioned offer
exports. Older rows, events and queued messages would fail to decode, so the
offer data in all three services starts empty.

The reset deliberately discards every offer, including offers imported by hand
through the Offers REST API. Nothing is backfilled.

## 1. Stop processing and writes

1. Stop the command and event workers of Retailer Sourcing, Offers and Customer
   Facing.
2. Stop outbox delivery in all three services, so no pending message reaches
   another service's inbox.
3. Suspend incoming writes: initial-fill and import requests, the Offers REST
   import and inbox submissions to Offers and Customer Facing.

## 2. Wipe offer data

Run each block in its service's database, inside one transaction.

Retailer Sourcing, all raw and derived rows, queues, event streams, saga
records and exports:

```sql
TRUNCATE offer_details_raw, offer_details_extracted, offer_details_normalized,
         offer_details_structuralized, offer_export,
         sitemaps_raw, sitemaps_processed, sitemaps_consolidated,
         shopify_graphql_initial_fills,
         command_entries, event_entries, inbox_entries, outbox_entries;
```

Offers, the event store, its projections and pending work:

```sql
TRUNCATE event_store, read_offers, read_offer_images, read_offer_identities,
         command_entries, event_entries, inbox_entries, outbox_entries;
```

Customer Facing, the offer projection, its receipts and pending work:

```sql
TRUNCATE offers, offer_images, applied_offer_messages,
         command_entries, event_entries, inbox_entries, outbox_entries;
```

If a service has added offer, saga or queue tables since this list was written,
truncate them too. Check with `\dt` before running the statement. Leave
object-storage contents in place. Old objects are no longer referenced.

## 3. Migrate and deploy

1. Run `make migrate` in `services/retailer_sourcing`, `services/offers` and
   `services/customer_facing` against the empty databases.
2. Deploy the three services.
3. Check each service's health endpoint.

## 4. Resume

Resume workers, outbox delivery and incoming writes in all three services
together. Then start new initial fills or downloads to repopulate offers.
