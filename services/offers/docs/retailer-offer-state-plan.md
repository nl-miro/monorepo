# Retailer offer state rewrite plan

Define all five structures below in a separate `state_next_generation` module.

Keep getters in a dedicated `impl` block above separate `impl` blocks for other
methods. Apply this ordering to the next-generation state and aggregate modules.

`RetailerOfferStateNextGeneration` should hold:

- `id`: a `RetailerOfferId` instance.
- `retailer_code`: a `RetailerCode` instance.
- `url`: a `String`.
- `identifiers`: an `ExternalOfferIdentifiersNextGeneration` instance.
- `target_market`: a `TargetMarketNextGeneration` enum instance.
- `language`: a `String`.
- `content`: a `ContentNextGeneration` instance.
- `price`: a `PricingNextGeneration` instance.
- `availability`: an `Option<Availability>`.
- `item_condition`: an `Option<ItemCondition>`.
- `created_at`: a `DateTime<Utc>`.
- `updated_at`: a `DateTime<Utc>`.

`retailer_code` and `url` cannot be changed after creation.

## Target market

`TargetMarketNextGeneration` has `Country { code: CountryCode }` and
`Region { code: RegionCode }` variants. Its JSON uses `kind` and `code`, with
uppercase country codes and `eu` for the European Union. Legacy region labels
are rejected. The next-generation state, creation event and aggregate use this
type directly.

## External offer identifiers

`ExternalOfferIdentifiersNextGeneration` should hold:

- `gtin`: an `Option<String>`.
- `sku`: an `Option<String>`.
- `shopify_product_gid`: an `Option<String>`.
- `shopify_product_variant_gid`: an `Option<String>`.

## Pricing

`PricingNextGeneration` should hold:

- `last`: a `PriceNextGeneration` instance.
- `history`: a `Vec<PriceNextGeneration>`.

### Price

`PriceNextGeneration` should hold:

- `current`: a `Money` instance.
- `previous`: an `Option<Money>`.
- `scraped_at`: a `DateTime<Utc>`.

## Content

`ContentNextGeneration` should hold:

- `title`: a `String`.
- `description`: a `String`.
- `images`: a `Vec<String>`.

## Aggregate

Define the aggregate in a separate `aggregate_next_generation` module.

`RetailerOfferAggregate` should hold:

- `state`: a `RetailerOfferStateNextGeneration` instance.
- `sequence`: a `u64`.
- `uncommitted_events`: a `Vec<EventEnvelope<RetailerOfferEvent>>`.
