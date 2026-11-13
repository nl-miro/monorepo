# Shopify extraction notes

## Extracted offer details

The main structure is named **extracted offer details**. It gathers product,
offer, and offer-related data in one payload.

The acquisition time (`acquired_at`) is stored at the root. It records when the
data was fetched into our system, because this time is not stored in any of
the events.

## Context

The context records the locale, country, currency, and language represented
by the payload.

## Product and related data

The payload is split into product, collections, images, and media. Collections,
images, and media are extracted into separate collections alongside the product
to reduce duplication and organize the volume of related information.

The product stores only their IDs as references.
