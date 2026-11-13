## Anker Australia

| Field                  | Value                                                      |
| ---------------------- | ---------------------------------------------------------- |
| Retailer code          | `ankerau`                                                  |
| Storefront website     | https://www.anker.com/au                                   |
| Shopify shop value     | oneanker-au.myshopify.com                                  |
| Storefront GraphQL API | https://oneanker-au.myshopify.com/api/2026-10/graphql.json |
| Target markets         | Australia (AU)                                             |
| Shipping countries     | AU                                                         |
|                        |                                                            |
|                        |                                                            |

| Supported locales | URL                      | Country/region | Language |
| ----------------- | ------------------------ | -------------- | -------- |
| en-AU             | https://www.anker.com/au | AU             | EN       |

## Anker Canada

| Field                  | Value                                                       |
| ---------------------- | ----------------------------------------------------------- |
| Retailer code          | `ankerca`                                                   |
| Storefront website     | https://www.anker.com/ca                                    |
| Shopify shop value     | anker-canada.myshopify.com                                  |
| Storefront GraphQL API | https://anker-canada.myshopify.com/api/2026-10/graphql.json |
| Target markets         | Canada (CA)                                                 |
| Shipping countries     | CA                                                          |
|                        |                                                             |
|                        |                                                             |

| Supported locales | URL                         | Country/region | Language |
| ----------------- | --------------------------- | -------------- | -------- |
| en-CA             | https://www.anker.com/ca    | CA             | EN       |
| fr-CA             | https://www.anker.com/ca-fr | CA             | FR       |

## Anker Germany

| Field                  | Value                                                                                   |
| ---------------------- | --------------------------------------------------------------------------------------- |
| Retailer code          | `ankerde`                                                                               |
| Storefront website     | https://www.anker.com/de                                                                |
| Shopify shop value     | oneanker-de.myshopify.com                                                               |
| Storefront GraphQL API | https://oneanker-de.myshopify.com/api/2026-10/graphql.json                              |
| Target markets         | Germany (DE)                                                                            |
| Shipping countries     | DE                                                                                      |
| Shipping restrictions  | Excludes standard delivery to postcodes 27498 and 78266; additional express exclusions. |
|                        |                                                                                         |
|                        |                                                                                         |

| Supported locales                 | URL                      | Country/region | Language |
| --------------------------------- | ------------------------ | -------------- | -------- |
| de (HTML), de-DE (alternate link) | https://www.anker.com/de | DE             | DE       |

Note: https://www.anker.com/eu-de redirects to https://www.anker.com/de and uses the same Shopify shop, oneanker-de.myshopify.com. It is an alias of `ankerde`, not a separate retailer.

GraphQL context (verified live on 2026-11-09): `@inContext(country: DE, language: DE)`. The API rejects `country: EU` as an invalid `CountryCode`.

## Anker Europe

| Field                  | Value                                                                                                                                          |
| ---------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| Retailer code          | `ankereu`                                                                                                                                      |
| Storefront website     | https://www.anker.com/eu-en                                                                                                                    |
| Shopify shop value     | ankereu.myshopify.com                                                                                                                          |
| Storefront GraphQL API | https://ankereu.myshopify.com/api/2026-10/graphql.json                                                                                         |
| Target markets         | Europe (EU)                                                                                                                                    |
| Shipping countries     | AT, BE, CZ, HU, LU, PL, PT, NL                                                                                                                 |
| Shipping restrictions  | Power banks excluded for Portugal; Solarbank 2 AC limited to Austria. Negative exclusions elsewhere on the page do not add delivery countries. |
|                        |                                                                                                                                                |
|                        |                                                                                                                                                |

| Supported locales | URL                         | Country/region | Language |
| ----------------- | --------------------------- | -------------- | -------- |
| en-EU             | https://www.anker.com/eu-en | AT             | EN       |

**Pricing decision:** Use Austria's country code (`AT`) to retrieve all Anker
Europe catalog prices from `ankereu.myshopify.com`, with
`@inContext(country: AT, language: EN)`. Use these returned EUR prices for the
Anker Europe price feed.

Croatia has observed price differences, but we intentionally disregard them
and will not collect a separate Croatian price feed. Poland returns prices in
Polish złoty (`PLN`); those prices will be handled separately, outside this
Austria-based EUR feed. This is our chosen collection policy, not a claim that
every country's actual prices are identical.

The Country/region and Language columns specify the intended collection context; Supported locales retains the website locale tags. Here, `AT` / `EN` is the selected pricing context for the `en-EU` storefront. Contexts elsewhere in this inventory are not claimed to be live-verified unless explicitly noted.

## Anker South Korea

| Field                  | Value                                            |
| ---------------------- | ------------------------------------------------ |
| Retailer code          | `ankerkr`                                        |
| Storefront website     | https://ankerkorea.co.kr/                        |
| Shopify shop value     | Unknown                                          |
| Storefront GraphQL API | Unknown                                          |
| Target markets         | South Korea (KR)                                 |
| Shipping countries     | Unknown                                          |
| GraphQL API status     | Unknown; no endpoint established in the research |
|                        |                                                  |
|                        |                                                  |

| Supported locales      | URL                       | Country/region | Language |
| ---------------------- | ------------------------- | -------------- | -------- |
| ko                     | https://ankerkorea.co.kr/ | KR             | KO       |
| ko-KR (alternate link) | https://ankerkorea.co.kr/ | KR             | KO       |

**Shipping evidence:** No reliable enumerated shipping-country definition established in the reviewed sources.

## Anker United States

| Field                  | Value                                                  |
| ---------------------- | ------------------------------------------------------ |
| Retailer code          | `ankercom`                                             |
| Storefront website     | https://www.anker.com/                                 |
| Shopify shop value     | ankerus.myshopify.com                                  |
| Storefront GraphQL API | https://ankerus.myshopify.com/api/2026-10/graphql.json |
| Target markets         | United States (US)                                     |
| Shipping countries     | US                                                     |
| Shipping restrictions  | Product/address restrictions apply.                    |
|                        |                                                        |
|                        |                                                        |

| Supported locales | URL                    | Country/region | Language |
| ----------------- | ---------------------- | -------------- | -------- |
| en-US             | https://www.anker.com/ | US             | EN       |

## Anker Poland

| Field                  | Value                                                                                          |
| ---------------------- | ---------------------------------------------------------------------------------------------- |
| Retailer code          | `ankerpl`                                                                                      |
| Storefront website     | https://www.anker.com/eu-pl                                                                    |
| Shopify shop value     | ankereu.myshopify.com                                                                          |
| Storefront GraphQL API | https://ankereu.myshopify.com/api/2026-10/graphql.json                                         |
| Target markets         | Poland (PL)                                                                                    |
| Shipping countries     | AT, BE, BG, HR, CZ, DK, EE, FI, FR, GR, ES, NL, LT, LU, DE, PL, PT, RO, SE, SK, SI, HU, IT, LV |
| Shipping restrictions  | Destinations explicitly listed in Polish; not inferred from the eu-pl code.                    |
|                        |                                                                                                |
|                        |                                                                                                |

| Supported locales | URL                         | Country/region | Language |
| ----------------- | --------------------------- | -------------- | -------- |
| pl-PL             | https://www.anker.com/eu-pl | PL             | PL       |

## Anker France

| Field                  | Value                                                   |
| ---------------------- | ------------------------------------------------------- |
| Retailer code          | `ankerfr`                                               |
| Storefront website     | https://www.anker.com/fr                                |
| Shopify shop value     | ankerfra.myshopify.com                                  |
| Storefront GraphQL API | https://ankerfra.myshopify.com/api/2026-10/graphql.json |
| Target markets         | France (FR)                                             |
| Shipping countries     | FR, IT, ES                                              |
|                        |                                                         |
|                        |                                                         |

| Supported locales | URL                      | Country/region | Language |
| ----------------- | ------------------------ | -------------- | -------- |
| fr-FR             | https://www.anker.com/fr | FR             | FR       |

## Anker New Zealand

| Field                  | Value                                                  |
| ---------------------- | ------------------------------------------------------ |
| Retailer code          | `ankernz`                                              |
| Storefront website     | https://www.anker.com/nz                               |
| Shopify shop value     | ankernz.myshopify.com                                  |
| Storefront GraphQL API | https://ankernz.myshopify.com/api/2026-10/graphql.json |
| Target markets         | Unknown (conflicting evidence)                         |
| Shipping countries     | Unknown                                                |
|                        |                                                        |
|                        |                                                        |

| Supported locales | URL                      | Country/region | Language |
| ----------------- | ------------------------ | -------------- | -------- |
| en-NZ             | https://www.anker.com/nz | NZ             | EN       |

**Shipping evidence:** No reliable enumerated shipping-country definition established in the reviewed sources.

**Research notes:** Conflicting localization: /nz and html[lang]=en-NZ, but footer/selector label says Australia / English. Target market unresolved. Shipping policy returns 404.

## Anker United Kingdom

| Field                  | Value                                                               |
| ---------------------- | ------------------------------------------------------------------- |
| Retailer code          | `ankeruk`                                                           |
| Storefront website     | https://www.anker.com/uk                                            |
| Shopify shop value     | oneanker-uk.myshopify.com                                           |
| Storefront GraphQL API | https://oneanker-uk.myshopify.com/api/2026-10/graphql.json          |
| Target markets         | United Kingdom (GB)                                                 |
| Shipping countries     | GB                                                                  |
| Shipping restrictions  | Some postcodes and PO boxes excluded; express coverage is narrower. |
|                        |                                                                     |
|                        |                                                                     |

| Supported locales | URL                      | Country/region | Language |
| ----------------- | ------------------------ | -------------- | -------- |
| en-GB             | https://www.anker.com/uk | GB             | EN       |

## Anker Vietnam

| Field                  | Value                                                         |
| ---------------------- | ------------------------------------------------------------- |
| Retailer code          | `ankervn`                                                     |
| Storefront website     | https://www.anker.com/vn                                      |
| Shopify shop value     | ankervnm.myshopify.com                                        |
| Storefront GraphQL API | https://ankervnm.myshopify.com/api/2026-10/graphql.json       |
| Target markets         | Unknown                                                       |
| Shipping countries     | Unknown                                                       |
| GraphQL API status     | Responded with GraphQL error: Online Store channel is locked. |
|                        |                                                               |
|                        |                                                               |

| Supported locales | URL                      | Country/region | Language |
| ----------------- | ------------------------ | -------------- | -------- |
| vi-VN             | https://www.anker.com/vn | VN             | VI       |

**Shipping evidence:** No reliable enumerated shipping-country definition established in the reviewed sources.

**Research notes:** html[lang]=vi-VN establishes the declared language. No explicit target-market label independently confirmed.

## Anker Italy

| Field                  | Value                                                      |
| ---------------------- | ---------------------------------------------------------- |
| Retailer code          | `ankeritalycom`                                            |
| Storefront website     | https://www.ankeritaly.com/                                |
| Shopify shop value     | anker-italy.myshopify.com                                  |
| Storefront GraphQL API | https://anker-italy.myshopify.com/api/2026-10/graphql.json |
| Target markets         | Italy (IT)                                                 |
| Shipping countries     | IT, FR, ES, DE, MT                                         |
|                        |                                                            |
|                        |                                                            |

| Supported locales | URL                         | Country/region | Language |
| ----------------- | --------------------------- | -------------- | -------- |
| it-IT             | https://www.ankeritaly.com/ | IT             | IT       |

**Research notes:** Italian html[lang] and storefront title conflict with English label in some cross-site selectors; English support is not confirmed.

## Anker Japan

| Field                  | Value                                                                    |
| ---------------------- | ------------------------------------------------------------------------ |
| Retailer code          | `ankerjapancom`                                                          |
| Storefront website     | https://www.ankerjapan.com/                                              |
| Shopify shop value     | anker-japan-official-site.myshopify.com                                  |
| Storefront GraphQL API | https://anker-japan-official-site.myshopify.com/api/2026-10/graphql.json |
| Target markets         | Japan (JP)                                                               |
| Shipping countries     | JP                                                                       |
| Shipping restrictions  | Some Okinawa addresses excluded; forwarding services not accepted.       |
|                        |                                                                          |
|                        |                                                                          |

| Supported locales | URL                         | Country/region | Language |
| ----------------- | --------------------------- | -------------- | -------- |
| ja                | https://www.ankerjapan.com/ | JP             | JA       |

## Anker Nordics

| Field                  | Value                                                              |
| ---------------------- | ------------------------------------------------------------------ |
| Retailer code          | `ankernordicscom`                                                  |
| Storefront website     | https://www.ankernordics.com/                                      |
| Shopify shop value     | ankerneu.myshopify.com                                             |
| Storefront GraphQL API | https://ankerneu.myshopify.com/api/2026-10/graphql.json            |
| Target markets         | Denmark (DK), Finland (FI), Iceland (IS), Norway (NO), Sweden (SE) |
| Shipping countries     | SE, NO, DK, FI, IS                                                 |
| Shipping restrictions  | Iceland import clearance and product-specific exclusions apply.    |
|                        |                                                                    |
|                        |                                                                    |

| Supported locales      | URL                           | Country/region | Language |
| ---------------------- | ----------------------------- | -------------- | -------- |
| en-SE                  | https://www.ankernordics.com/ | SE             | EN       |
| en-dk (alternate link) | https://www.ankernordics.com/ | DK             | EN       |
| en-fi (alternate link) | https://www.ankernordics.com/ | FI             | EN       |
| en-is (alternate link) | https://www.ankernordics.com/ | IS             | EN       |
| en-no (alternate link) | https://www.ankernordics.com/ | NO             | EN       |

## Minisforum Australia

| Field                  | Value                                                    |
| ---------------------- | -------------------------------------------------------- |
| Retailer code          | `minisforumau`                                           |
| Storefront website     | https://au.minisforum.com/                               |
| Shopify shop value     | j94nqi-71.myshopify.com                                  |
| Storefront GraphQL API | https://j94nqi-71.myshopify.com/api/2026-10/graphql.json |
| Target markets         | Australia (AU)                                           |
| Shipping countries     | AU                                                       |
|                        |                                                          |
|                        |                                                          |

| Supported locales | URL                        | Country/region | Language |
| ----------------- | -------------------------- | -------------- | -------- |
| en                | https://au.minisforum.com/ | AU             | EN       |

**Shipping evidence:** Status: published. Official Where to Buy table explicitly lists supported destinations. Source: https://minisforumpc.eu/pages/faqs.

**Research notes:** Official Primary Market / Language table; not inferred from Shopify.country. Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. Country/region and Language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query.

## Minisforum Canada

| Field                  | Value                                                        |
| ---------------------- | ------------------------------------------------------------ |
| Retailer code          | `minisforumca`                                               |
| Storefront website     | https://ca.minisforum.com/                                   |
| Shopify shop value     | seaminisforum.myshopify.com                                  |
| Storefront GraphQL API | https://seaminisforum.myshopify.com/api/2026-10/graphql.json |
| Target markets         | Canada (CA)                                                  |
| Shipping countries     | CA                                                           |
|                        |                                                              |
|                        |                                                              |

| Supported locales | URL                        | Country/region | Language |
| ----------------- | -------------------------- | -------------- | -------- |
| en                | https://ca.minisforum.com/ | CA             | EN       |

**Shipping evidence:** Status: published. Official Where to Buy table explicitly lists supported destinations. Source: https://minisforumpc.eu/pages/faqs.

**Research notes:** Official Primary Market / Language table; not inferred from Shopify.country. Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. Country/region and Language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query.

## Minisforum Hong Kong

| Field                  | Value                                                       |
| ---------------------- | ----------------------------------------------------------- |
| Retailer code          | `minisforumhk`                                              |
| Storefront website     | https://minisforum.hk/                                      |
| Shopify shop value     | minisforumcn.myshopify.com                                  |
| Storefront GraphQL API | https://minisforumcn.myshopify.com/api/2026-10/graphql.json |
| Target markets         | Hong Kong (HK)                                              |
| Shipping countries     | HK                                                          |
|                        |                                                             |
|                        |                                                             |

| Supported locales | URL                    | Country/region | Language |
| ----------------- | ---------------------- | -------------- | -------- |
| zh-TW             | https://minisforum.hk/ | HK             | ZH_TW    |

**Shipping evidence:** Status: published partial. Homepage explicitly offers Hong Kong delivery, excluding outlying islands/remote areas. Other destinations not established. Source: https://minisforum.hk/.

**Research notes:** Official Primary Market / Language table; not inferred from Shopify.country. html[lang]=zh-TW identifies a language variety, not a Taiwan target market. Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. Country/region and Language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query. The intended language code ZH_TW corresponds to the observed zh-TW locale; the reviewed language is Traditional Chinese (zh-Hant).

## Minisforum South Korea

| Field                  | Value                                                       |
| ---------------------- | ----------------------------------------------------------- |
| Retailer code          | `minisforumkr`                                              |
| Storefront website     | https://minisforum.kr/                                      |
| Shopify shop value     | minisforumkr.myshopify.com                                  |
| Storefront GraphQL API | https://minisforumkr.myshopify.com/api/2026-10/graphql.json |
| Target markets         | South Korea (KR)                                            |
| Shipping countries     | KR                                                          |
|                        |                                                             |
|                        |                                                             |

| Supported locales | URL                    | Country/region | Language |
| ----------------- | ---------------------- | -------------- | -------- |
| ko                | https://minisforum.kr/ | KR             | KO       |

**Shipping evidence:** Status: published. Official Where to Buy table explicitly lists supported destinations. Source: https://minisforumpc.eu/pages/faqs.

**Research notes:** Official Primary Market / Language table; not inferred from Shopify.country. Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. Country/region and Language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query.

## Minisforum Russia

| Field                  | Value                                                              |
| ---------------------- | ------------------------------------------------------------------ |
| Retailer code          | `minisforumru`                                                     |
| Storefront website     | https://minisforum.ru/                                             |
| Shopify shop value     | store-minisforum-ru.myshopify.com                                  |
| Storefront GraphQL API | https://store-minisforum-ru.myshopify.com/api/2026-10/graphql.json |
| Target markets         | Russia (RU)                                                        |
| Shipping countries     | Unknown                                                            |
|                        |                                                                    |
|                        |                                                                    |

| Supported locales | URL                    | Country/region | Language |
| ----------------- | ---------------------- | -------------- | -------- |
| ru                | https://minisforum.ru/ | RU             | RU       |

**Shipping evidence:** Status: unbounded claim. FAQ claims worldwide shipping without an enumerated country list; do not expand to all ISO countries. Source: https://minisforum.ru/pages/faqs.

**Research notes:** Official Primary Market / Language table; not inferred from Shopify.country. html[lang] conflicts with the reviewed language declaration; retained as raw evidence only. Shopify.country=US and locale=en conflict with official Russia/Russian table; not accepted as target-market or English-language evidence. Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. Country/region and Language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query.

## Minisforum Europe

| Field                  | Value                                                                                                                                      |
| ---------------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| Retailer code          | `minisforumeu`                                                                                                                             |
| Storefront website     | https://minisforumpc.eu/                                                                                                                   |
| Shopify shop value     | minisforum-eu.myshopify.com                                                                                                                |
| Storefront GraphQL API | https://minisforum-eu.myshopify.com/api/2026-10/graphql.json                                                                               |
| Target markets         | EU Countries (EU)                                                                                                                          |
| Shipping countries     | CZ, DK, AT, BE, BG, HR, EE, FI, FR, DE, GR, IE, IT, LV, LT, LU, NL, PL, PT, RO, SK, SI, ES, SE, HU, CH, AL, BY, MK, IS, BA, VA, LI, MC, SM |
|                        |                                                                                                                                            |
|                        |                                                                                                                                            |

| Supported locales    | URL                        | Country/region | Language |
| -------------------- | -------------------------- | -------------- | -------- |
| en                   | https://minisforumpc.eu/   | Not selected   | EN       |
| de (advertised only) | https://minisforumpc.eu/de | Not selected   | DE       |

**Shipping evidence:** Status: conflicting lists. Where to Buy table; Spanish island exclusions. Another shipping section additionally lists NO and GB: retain conflict instead of merging. Source: https://minisforumpc.eu/pages/faqs.

**Research notes:** Official Primary Market / Language table; not inferred from Shopify.country. Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. Country/region and Language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query. A collection country has not been selected. EU labels the regional storefront and is not a Shopify CountryCode; the Anker Europe AT pricing decision does not apply here. Rows marked advertised only are alternate links, not verified locale pages or confirmed extra content requests. Their country contexts remain unselected, and the base Shopify backend has not been established for those URLs.

## Minisforum France

| Field                  | Value                                                        |
| ---------------------- | ------------------------------------------------------------ |
| Retailer code          | `minisforumfr`                                               |
| Storefront website     | https://minisforumpc.fr/                                     |
| Shopify shop value     | minisforum-fr.myshopify.com                                  |
| Storefront GraphQL API | https://minisforum-fr.myshopify.com/api/2026-10/graphql.json |
| Target markets         | France (FR)                                                  |
| Shipping countries     | BE, FR, LU                                                   |
|                        |                                                              |
|                        |                                                              |

| Supported locales    | URL                        | Country/region | Language |
| -------------------- | -------------------------- | -------------- | -------- |
| fr                   | https://minisforumpc.fr/   | FR             | FR       |
| en (advertised only) | https://minisforumpc.fr/en | Not selected   | EN       |

**Shipping evidence:** Status: published. Official Where to Buy table explicitly lists supported destinations. Source: https://minisforumpc.eu/pages/faqs.

**Research notes:** Official Primary Market / Language table; not inferred from Shopify.country. Shopify.country=HK on France store; not accepted as target market. Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. Country/region and Language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query. Rows marked advertised only are alternate links, not verified locale pages or confirmed extra content requests. Their country contexts remain unselected, and the base Shopify backend has not been established for those URLs.

## Minisforum United States

| Field                  | Value                                                                                                                                                      |
| ---------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Retailer code          | `minisforumus`                                                                                                                                             |
| Storefront website     | https://store.minisforum.com/                                                                                                                              |
| Shopify shop value     | minisforum.myshopify.com                                                                                                                                   |
| Storefront GraphQL API | https://minisforum.myshopify.com/api/2026-10/graphql.json                                                                                                  |
| Target markets         | United States and others (not enumerated) (US)                                                                                                             |
| Shipping countries     | US, CA, AE, AU, SA, PH, NZ, AG, BS, BB, BZ, BM, KY, DO, EC, SV, GD, HN, JM, LR, MX, NI, PA, KN, LC, TT, TC, VE, FJ, PG, WS, SB, TO, VU, SG, ID, MY, IL, KW |
|                        |                                                                                                                                                            |
|                        |                                                                                                                                                            |

| Supported locales       | URL                                | Country/region | Language |
| ----------------------- | ---------------------------------- | -------------- | -------- |
| en                      | https://store.minisforum.com/      | US             | EN       |
| en-CA (advertised only) | https://store.minisforum.com/en-ca | Not selected   | EN       |
| en-CL (advertised only) | https://store.minisforum.com/en-os | Not selected   | EN       |
| en-PE (advertised only) | https://store.minisforum.com/en-os | Not selected   | EN       |

**Shipping evidence:** Status: published. Official supported-region table; US territories and military addresses excluded. Source: https://minisforumpc.eu/pages/faqs.

**Research notes:** Official Primary Market / Language table; not inferred from Shopify.country. Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. Country/region and Language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query. Rows marked advertised only are alternate links, not verified locale pages or confirmed extra content requests. Their country contexts remain unselected, and the base Shopify backend has not been established for those URLs.

## Minisforum Japan

| Field                  | Value                                                        |
| ---------------------- | ------------------------------------------------------------ |
| Retailer code          | `minisforumjp`                                               |
| Storefront website     | https://www.minisforum.jp/                                   |
| Shopify shop value     | minisforum-jp.myshopify.com                                  |
| Storefront GraphQL API | https://minisforum-jp.myshopify.com/api/2026-10/graphql.json |
| Target markets         | Japan (JP)                                                   |
| Shipping countries     | JP                                                           |
|                        |                                                              |
|                        |                                                              |

| Supported locales | URL                        | Country/region | Language |
| ----------------- | -------------------------- | -------------- | -------- |
| ja                | https://www.minisforum.jp/ | JP             | JA       |

**Shipping evidence:** Status: published. Official Where to Buy table explicitly lists supported destinations. Source: https://minisforumpc.eu/pages/faqs.

**Research notes:** Official Primary Market / Language table; not inferred from Shopify.country. Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. Country/region and Language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query.

## Minisforum United Kingdom

| Field                  | Value                                                        |
| ---------------------- | ------------------------------------------------------------ |
| Retailer code          | `minisforumuk`                                               |
| Storefront website     | https://www.minisforum.uk/                                   |
| Shopify shop value     | minisforum-uk.myshopify.com                                  |
| Storefront GraphQL API | https://minisforum-uk.myshopify.com/api/2026-10/graphql.json |
| Target markets         | United Kingdom (GB)                                          |
| Shipping countries     | IE, GB                                                       |
|                        |                                                              |
|                        |                                                              |

| Supported locales | URL                        | Country/region | Language |
| ----------------- | -------------------------- | -------------- | -------- |
| en                | https://www.minisforum.uk/ | GB             | EN       |

**Shipping evidence:** Status: published. Official Where to Buy table explicitly lists supported destinations. Source: https://minisforumpc.eu/pages/faqs.

**Research notes:** Official Primary Market / Language table; not inferred from Shopify.country. Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. Country/region and Language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query.

## Ugreen Canada

| Field                  | Value                                                    |
| ---------------------- | -------------------------------------------------------- |
| Retailer code          | `ugreenca`                                               |
| Storefront website     | https://ca.ugreen.com/                                   |
| Shopify shop value     | ugreen-ca.myshopify.com                                  |
| Storefront GraphQL API | https://ugreen-ca.myshopify.com/api/2026-10/graphql.json |
| Target markets         | Canada (CA)                                              |
| Shipping countries     | Unknown                                                  |
|                        |                                                          |
|                        |                                                          |

| Supported locales    | URL                      | Country/region | Language |
| -------------------- | ------------------------ | -------------- | -------- |
| en                   | https://ca.ugreen.com/   | CA             | EN       |
| fr (advertised only) | https://ca.ugreen.com/fr | Not selected   | FR       |

**Shipping evidence:** Status: conflict. Canada-branded storefront policy instead says United States. Do not accept US or infer CA as verified delivery coverage. Source: https://ca.ugreen.com/policies/shipping-policy.

**Research notes:** Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. Country/region and Language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query. Rows marked advertised only are alternate links, not verified locale pages or confirmed extra content requests. Their country contexts remain unselected, and the base Shopify backend has not been established for those URLs.

## Ugreen Germany

| Field                  | Value                                                    |
| ---------------------- | -------------------------------------------------------- |
| Retailer code          | `ugreende`                                               |
| Storefront website     | https://de.ugreen.com/                                   |
| Shopify shop value     | ugreen-de.myshopify.com                                  |
| Storefront GraphQL API | https://ugreen-de.myshopify.com/api/2026-10/graphql.json |
| Target markets         | Germany (DE)                                             |
| Shipping countries     | DE, AT, DK, FR, NL, PL                                   |
|                        |                                                          |
|                        |                                                          |

| Supported locales | URL                    | Country/region | Language |
| ----------------- | ---------------------- | -------------- | -------- |
| de                | https://de.ugreen.com/ | DE             | DE       |

**Shipping evidence:** Status: published.  Source: https://us.ugreen.com/pages/shipping-policy.

**Research notes:** Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. Country/region and Language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query.

## Ugreen Europe

| Field                  | Value                                                                                                  |
| ---------------------- | ------------------------------------------------------------------------------------------------------ |
| Retailer code          | `ugreeneu`                                                                                             |
| Storefront website     | https://eu.ugreen.com/                                                                                 |
| Shopify shop value     | ugreen-eu.myshopify.com                                                                                |
| Storefront GraphQL API | https://ugreen-eu.myshopify.com/api/2026-10/graphql.json                                               |
| Target markets         | Europe (EU)                                                                                            |
| Shipping countries     | BG, HR, EE, HU, PT, RO, SI, SE, AT, BE, CZ, FR, DE, IT, MT, NL, PL, ES, DK, LV, FI, LU, SK, CY, LT, CH |
|                        |                                                                                                        |
|                        |                                                                                                        |

| Supported locales       | URL                         | Country/region | Language |
| ----------------------- | --------------------------- | -------------- | -------- |
| en                      | https://eu.ugreen.com/      | Not selected   | EN       |
| bg (advertised only)    | https://eu.ugreen.com/bg    | Not selected   | BG       |
| bg-BG (advertised only) | https://eu.ugreen.com/bg-bg | Not selected   | BG       |
| cs (advertised only)    | https://eu.ugreen.com/cs    | Not selected   | CS       |
| cs-CZ (advertised only) | https://eu.ugreen.com/cs-cz | Not selected   | CS       |
| da (advertised only)    | https://eu.ugreen.com/da    | Not selected   | DA       |
| da-DK (advertised only) | https://eu.ugreen.com/da-dk | Not selected   | DA       |
| de (advertised only)    | https://eu.ugreen.com/de    | Not selected   | DE       |
| de-AT (advertised only) | https://eu.ugreen.com/de-at | Not selected   | DE       |
| de-BE (advertised only) | https://eu.ugreen.com/de-be | Not selected   | DE       |
| de-CH (advertised only) | https://eu.ugreen.com/de-ch | Not selected   | DE       |
| el (advertised only)    | https://eu.ugreen.com/el    | Not selected   | EL       |
| el-GR (advertised only) | https://eu.ugreen.com/el-gr | Not selected   | EL       |
| en-AT (advertised only) | https://eu.ugreen.com/en-at | Not selected   | EN       |
| en-BE (advertised only) | https://eu.ugreen.com/en-be | Not selected   | EN       |
| en-BG (advertised only) | https://eu.ugreen.com/en-bg | Not selected   | EN       |
| en-CH (advertised only) | https://eu.ugreen.com/en-ch | Not selected   | EN       |
| en-CZ (advertised only) | https://eu.ugreen.com/en-cz | Not selected   | EN       |
| en-DK (advertised only) | https://eu.ugreen.com/en-dk | Not selected   | EN       |
| en-EE (advertised only) | https://eu.ugreen.com/en-ee | Not selected   | EN       |
| en-ES (advertised only) | https://eu.ugreen.com/en-es | Not selected   | EN       |
| en-FI (advertised only) | https://eu.ugreen.com/en-fi | Not selected   | EN       |
| en-FR (advertised only) | https://eu.ugreen.com/en-fr | Not selected   | EN       |
| en-GR (advertised only) | https://eu.ugreen.com/en-gr | Not selected   | EN       |
| en-HR (advertised only) | https://eu.ugreen.com/en-hr | Not selected   | EN       |
| en-HU (advertised only) | https://eu.ugreen.com/en-hu | Not selected   | EN       |
| en-IT (advertised only) | https://eu.ugreen.com/en-it | Not selected   | EN       |
| en-LV (advertised only) | https://eu.ugreen.com/en-lv | Not selected   | EN       |
| en-NL (advertised only) | https://eu.ugreen.com/en-nl | Not selected   | EN       |
| en-PL (advertised only) | https://eu.ugreen.com/en-pl | Not selected   | EN       |
| en-PT (advertised only) | https://eu.ugreen.com/en-pt | Not selected   | EN       |
| en-RO (advertised only) | https://eu.ugreen.com/en-ro | Not selected   | EN       |
| en-SE (advertised only) | https://eu.ugreen.com/en-se | Not selected   | EN       |
| en-SI (advertised only) | https://eu.ugreen.com/en-si | Not selected   | EN       |
| en-SK (advertised only) | https://eu.ugreen.com/en-sk | Not selected   | EN       |
| es (advertised only)    | https://eu.ugreen.com/es    | Not selected   | ES       |
| es-ES (advertised only) | https://eu.ugreen.com/es-es | Not selected   | ES       |
| et (advertised only)    | https://eu.ugreen.com/et    | Not selected   | ET       |
| et-EE (advertised only) | https://eu.ugreen.com/et-ee | Not selected   | ET       |
| fi (advertised only)    | https://eu.ugreen.com/fi    | Not selected   | FI       |
| fr (advertised only)    | https://eu.ugreen.com/fr    | Not selected   | FR       |
| fr-BE (advertised only) | https://eu.ugreen.com/fr-be | Not selected   | FR       |
| fr-FR (advertised only) | https://eu.ugreen.com/fr-fr | Not selected   | FR       |
| hr (advertised only)    | https://eu.ugreen.com/hr    | Not selected   | HR       |
| hr-HR (advertised only) | https://eu.ugreen.com/hr-hr | Not selected   | HR       |
| hu (advertised only)    | https://eu.ugreen.com/hu    | Not selected   | HU       |
| hu-HU (advertised only) | https://eu.ugreen.com/hu-hu | Not selected   | HU       |
| it (advertised only)    | https://eu.ugreen.com/it    | Not selected   | IT       |
| it-IT (advertised only) | https://eu.ugreen.com/it-it | Not selected   | IT       |
| lv (advertised only)    | https://eu.ugreen.com/lv    | Not selected   | LV       |
| lv-LV (advertised only) | https://eu.ugreen.com/lv-lv | Not selected   | LV       |
| nl (advertised only)    | https://eu.ugreen.com/nl    | Not selected   | NL       |
| nl-BE (advertised only) | https://eu.ugreen.com/nl-be | Not selected   | NL       |
| nl-NL (advertised only) | https://eu.ugreen.com/nl-nl | Not selected   | NL       |
| pl (advertised only)    | https://eu.ugreen.com/pl    | Not selected   | PL       |
| pl-PL (advertised only) | https://eu.ugreen.com/pl-pl | Not selected   | PL       |
| pt (advertised only)    | https://eu.ugreen.com/pt    | Not selected   | PT       |
| pt-PT (advertised only) | https://eu.ugreen.com/pt-pt | Not selected   | PT       |
| ro (advertised only)    | https://eu.ugreen.com/ro    | Not selected   | RO       |
| ro-RO (advertised only) | https://eu.ugreen.com/ro-ro | Not selected   | RO       |
| sk (advertised only)    | https://eu.ugreen.com/sk    | Not selected   | SK       |
| sk-SK (advertised only) | https://eu.ugreen.com/sk-sk | Not selected   | SK       |
| sl (advertised only)    | https://eu.ugreen.com/sl    | Not selected   | SL       |
| sl-SI (advertised only) | https://eu.ugreen.com/sl-si | Not selected   | SL       |
| sv (advertised only)    | https://eu.ugreen.com/sv    | Not selected   | SV       |
| sv-SE (advertised only) | https://eu.ugreen.com/sv-se | Not selected   | SV       |

**Shipping evidence:** Status: published. Portugal and Spain have named island/enclave exclusions. Source: https://us.ugreen.com/pages/shipping-policy.

**Research notes:** Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. Country/region and Language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query. A collection country has not been selected. EU labels the regional storefront and is not a Shopify CountryCode; the Anker Europe AT pricing decision does not apply here. Rows marked advertised only are alternate links, not verified locale pages or confirmed extra content requests. Their country contexts remain unselected, and the base Shopify backend has not been established for those URLs.

## Ugreen France

| Field                  | Value                                                    |
| ---------------------- | -------------------------------------------------------- |
| Retailer code          | `ugreenfr`                                               |
| Storefront website     | https://fr.ugreen.com/                                   |
| Shopify shop value     | ugreen-fr.myshopify.com                                  |
| Storefront GraphQL API | https://ugreen-fr.myshopify.com/api/2026-10/graphql.json |
| Target markets         | France (FR)                                              |
| Shipping countries     | FR, AT, BE, DE, IT, ES                                   |
|                        |                                                          |
|                        |                                                          |

| Supported locales | URL                    | Country/region | Language |
| ----------------- | ---------------------- | -------------- | -------- |
| fr                | https://fr.ugreen.com/ | FR             | FR       |

**Shipping evidence:** Status: published.  Source: https://us.ugreen.com/pages/shipping-policy.

**Research notes:** Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. Country/region and Language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query.

## Ugreen South Korea

| Field                  | Value                                                    |
| ---------------------- | -------------------------------------------------------- |
| Retailer code          | `ugreenkr`                                               |
| Storefront website     | https://kr.ugreen.com/                                   |
| Shopify shop value     | ugreen-kr.myshopify.com                                  |
| Storefront GraphQL API | https://ugreen-kr.myshopify.com/api/2026-10/graphql.json |
| Target markets         | South Korea (KR)                                         |
| Shipping countries     | Unknown                                                  |
|                        |                                                          |
|                        |                                                          |

| Supported locales | URL                    | Country/region | Language |
| ----------------- | ---------------------- | -------------- | -------- |
| ko                | https://kr.ugreen.com/ | KR             | KO       |

**Shipping evidence:** Status: unknown. No reliable enumerated shipping-country definition established in the reviewed sources.

**Research notes:** Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. Country/region and Language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query.

## Ugreen NAS Australia

| Field                  | Value                                                        |
| ---------------------- | ------------------------------------------------------------ |
| Retailer code          | `ugreennasau`                                                |
| Storefront website     | https://nas-au.ugreen.com/                                   |
| Shopify shop value     | nas-ugreen-au.myshopify.com                                  |
| Storefront GraphQL API | https://nas-ugreen-au.myshopify.com/api/2026-10/graphql.json |
| Target markets         | Australia (AU)                                               |
| Shipping countries     | Unknown                                                      |
|                        |                                                              |
|                        |                                                              |

| Supported locales | URL                        | Country/region | Language |
| ----------------- | -------------------------- | -------------- | -------- |
| en                | https://nas-au.ugreen.com/ | AU             | EN       |

**Shipping evidence:** Status: conflict. Australia storefront policy instead says United States. Coverage unresolved. Source: https://ai-au.ugreen.com/pages/shipping.

**Research notes:** Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. Country/region and Language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query. Related selector URL: https://ai-au.ugreen.com/. Retained here as notes rather than separate retailer tables.

## Ugreen NAS Canada

| Field                  | Value                                                        |
| ---------------------- | ------------------------------------------------------------ |
| Retailer code          | `ugreennasca`                                                |
| Storefront website     | https://nas-ca.ugreen.com/                                   |
| Shopify shop value     | ugreen-nas-ca.myshopify.com                                  |
| Storefront GraphQL API | https://ugreen-nas-ca.myshopify.com/api/2026-10/graphql.json |
| Target markets         | Canada (CA)                                                  |
| Shipping countries     | CA                                                           |
|                        |                                                              |
|                        |                                                              |

| Supported locales | URL                        | Country/region | Language |
| ----------------- | -------------------------- | -------------- | -------- |
| en                | https://nas-ca.ugreen.com/ | CA             | EN       |

**Shipping evidence:** Status: published. Most areas; no exhaustive excluded-postcode list. Source: https://ai-ca.ugreen.com/pages/shipping.

**Research notes:** Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. Country/region and Language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query. Related selector URL: https://ai-ca.ugreen.com/. Retained here as notes rather than separate retailer tables.

## Ugreen NAS Germany

| Field                  | Value                                                        |
| ---------------------- | ------------------------------------------------------------ |
| Retailer code          | `ugreennasde`                                                |
| Storefront website     | https://nas-de.ugreen.com/                                   |
| Shopify shop value     | ugreen-de-nas.myshopify.com                                  |
| Storefront GraphQL API | https://ugreen-de-nas.myshopify.com/api/2026-10/graphql.json |
| Target markets         | Germany (DE)                                                 |
| Shipping countries     | DE, AT                                                       |
|                        |                                                              |
|                        |                                                              |

| Supported locales | URL                        | Country/region | Language |
| ----------------- | -------------------------- | -------------- | -------- |
| de (selector)     | https://nas-de.ugreen.com/ | DE             | DE       |

**Shipping evidence:** Status: conflicting lists. Delivery-area section and product pages agree on DE/AT. Later legacy text lists additional European countries; do not merge. Source: https://ai-de.ugreen.com/pages/shipping.

**Research notes:** Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. Country/region and Language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query. The base language is declared by the storefront selector; direct page-language confirmation is unavailable in the research. Related selector URL: https://ai-de.ugreen.com/. Retained here as notes rather than separate retailer tables.

## Ugreen NAS Spain

| Field                  | Value                                                        |
| ---------------------- | ------------------------------------------------------------ |
| Retailer code          | `ugreennases`                                                |
| Storefront website     | https://nas-es.ugreen.com/                                   |
| Shopify shop value     | ugreen-nas-es.myshopify.com                                  |
| Storefront GraphQL API | https://ugreen-nas-es.myshopify.com/api/2026-10/graphql.json |
| Target markets         | Spain (ES)                                                   |
| Shipping countries     | ES                                                           |
|                        |                                                              |
|                        |                                                              |

| Supported locales | URL                        | Country/region | Language |
| ----------------- | -------------------------- | -------------- | -------- |
| es (selector)     | https://nas-es.ugreen.com/ | ES             | ES       |

**Shipping evidence:** Status: published. Canary Islands excluded. Source: https://ai-es.ugreen.com/pages/shipping.

**Research notes:** Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. Country/region and Language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query. The base language is declared by the storefront selector; direct page-language confirmation is unavailable in the research. Related selector URL: https://ai-es.ugreen.com/. Retained here as notes rather than separate retailer tables.

## Ugreen NAS Europe

| Field                  | Value                                                              |
| ---------------------- | ------------------------------------------------------------------ |
| Retailer code          | `ugreennaseu`                                                      |
| Storefront website     | https://nas-eu.ugreen.com/                                         |
| Shopify shop value     | ugreen-nas-eu.myshopify.com                                        |
| Storefront GraphQL API | https://ugreen-nas-eu.myshopify.com/api/2026-10/graphql.json       |
| Target markets         | Europe (EU)                                                        |
| Shipping countries     | DE, FR, IT, NL, ES, BE, PT, RO, CZ, PL, HR, AT, BG, LT, IE, SE, GR |
|                        |                                                                    |
|                        |                                                                    |

| Supported locales | URL                        | Country/region | Language |
| ----------------- | -------------------------- | -------------- | -------- |
| en (selector)     | https://nas-eu.ugreen.com/ | Not selected   | EN       |

**Shipping evidence:** Status: published. France overseas territories and Spain Canary Islands excluded. Source: https://ai-eu.ugreen.com/en-es/pages/shipping.

**Research notes:** Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. Country/region and Language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query. A collection country has not been selected. EU labels the regional storefront and is not a Shopify CountryCode; the Anker Europe AT pricing decision does not apply here. The base language is declared by the storefront selector; direct page-language confirmation is unavailable in the research. Related selector URL: https://ai-eu.ugreen.com/. Retained here as notes rather than separate retailer tables. Additional paths with unverified language support: https://nas-eu.ugreen.com/de, https://nas-eu.ugreen.com/en-be, https://nas-eu.ugreen.com/en-es, https://nas-eu.ugreen.com/en-fr, https://nas-eu.ugreen.com/en-it, https://nas-eu.ugreen.com/en-nl, https://nas-eu.ugreen.com/en-pt, https://nas-eu.ugreen.com/en-ro, https://nas-eu.ugreen.com/en-se, https://nas-eu.ugreen.com/es-es, https://nas-eu.ugreen.com/fr-be, https://nas-eu.ugreen.com/fr-fr, https://nas-eu.ugreen.com/it-it, https://nas-eu.ugreen.com/nl-be, https://nas-eu.ugreen.com/nl-nl, https://nas-eu.ugreen.com/pt-pt, https://nas-eu.ugreen.com/ro-ro, https://nas-eu.ugreen.com/sv-se. Their path names are not treated as confirmed locales.

## Ugreen NAS France

| Field                  | Value                                                        |
| ---------------------- | ------------------------------------------------------------ |
| Retailer code          | `ugreennasfr`                                                |
| Storefront website     | https://nas-fr.ugreen.com/                                   |
| Shopify shop value     | nas-ugreen-fr.myshopify.com                                  |
| Storefront GraphQL API | https://nas-ugreen-fr.myshopify.com/api/2026-10/graphql.json |
| Target markets         | France (FR)                                                  |
| Shipping countries     | FR                                                           |
|                        |                                                              |
|                        |                                                              |

| Supported locales | URL                        | Country/region | Language |
| ----------------- | -------------------------- | -------------- | -------- |
| fr (selector)     | https://nas-fr.ugreen.com/ | FR             | FR       |

**Shipping evidence:** Status: published.  Source: https://ai-fr.ugreen.com/policies/shipping-policy.

**Research notes:** Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. Country/region and Language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query. The base language is declared by the storefront selector; direct page-language confirmation is unavailable in the research. Related selector URL: https://ai-fr.ugreen.com/. Retained here as notes rather than separate retailer tables.

## Ugreen NAS Italy

| Field                  | Value                                                        |
| ---------------------- | ------------------------------------------------------------ |
| Retailer code          | `ugreennasit`                                                |
| Storefront website     | https://nas-it.ugreen.com/                                   |
| Shopify shop value     | ugreen-nas-it.myshopify.com                                  |
| Storefront GraphQL API | https://ugreen-nas-it.myshopify.com/api/2026-10/graphql.json |
| Target markets         | Italy (IT)                                                   |
| Shipping countries     | IT                                                           |
|                        |                                                              |
|                        |                                                              |

| Supported locales | URL                        | Country/region | Language |
| ----------------- | -------------------------- | -------------- | -------- |
| it (selector)     | https://nas-it.ugreen.com/ | IT             | IT       |

**Shipping evidence:** Status: published.  Source: https://ai-it.ugreen.com/policies/shipping-policy.

**Research notes:** Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. Country/region and Language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query. The base language is declared by the storefront selector; direct page-language confirmation is unavailable in the research. Related selector URL: https://ai-it.ugreen.com/. Retained here as notes rather than separate retailer tables.

## Ugreen NAS United Kingdom

| Field                  | Value                                                        |
| ---------------------- | ------------------------------------------------------------ |
| Retailer code          | `ugreennasuk`                                                |
| Storefront website     | https://nas-uk.ugreen.com/                                   |
| Shopify shop value     | ugreen-nas-uk.myshopify.com                                  |
| Storefront GraphQL API | https://ugreen-nas-uk.myshopify.com/api/2026-10/graphql.json |
| Target markets         | United Kingdom (GB)                                          |
| Shipping countries     | GB                                                           |
|                        |                                                              |
|                        |                                                              |

| Supported locales | URL                        | Country/region | Language |
| ----------------- | -------------------------- | -------------- | -------- |
| en (selector)     | https://nas-uk.ugreen.com/ | GB             | EN       |

**Shipping evidence:** Status: published.  Source: https://ai-uk.ugreen.com/pages/shipping.

**Research notes:** Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. Country/region and Language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query. The base language is declared by the storefront selector; direct page-language confirmation is unavailable in the research. Related selector URL: https://ai-uk.ugreen.com/. Retained here as notes rather than separate retailer tables.

## Ugreen NAS United States

| Field                  | Value                                                     |
| ---------------------- | --------------------------------------------------------- |
| Retailer code          | `ugreennas`                                               |
| Storefront website     | https://nas.ugreen.com/                                   |
| Shopify shop value     | ugreen-nas.myshopify.com                                  |
| Storefront GraphQL API | https://ugreen-nas.myshopify.com/api/2026-10/graphql.json |
| Target markets         | United States (US)                                        |
| Shipping countries     | US                                                        |
|                        |                                                           |
|                        |                                                           |

| Supported locales | URL                     | Country/region | Language |
| ----------------- | ----------------------- | -------------- | -------- |
| en                | https://nas.ugreen.com/ | US             | EN       |

**Shipping evidence:** Status: published. Excludes Alaska, Hawaii, Rhode Island, listed territories and military addresses. Source: https://ai.ugreen.com/pages/shipping.

**Research notes:** Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. Country/region and Language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query. Related selector URL: https://ai.ugreen.com/. Retained here as notes rather than separate retailer tables.

## Ugreen NAS Japan

| Field                  | Value                                                        |
| ---------------------- | ------------------------------------------------------------ |
| Retailer code          | `ugreennasjp`                                                |
| Storefront website     | https://nas.ugreen.jp/                                       |
| Shopify shop value     | ugreen-nas-jp.myshopify.com                                  |
| Storefront GraphQL API | https://ugreen-nas-jp.myshopify.com/api/2026-10/graphql.json |
| Target markets         | Japan (JP)                                                   |
| Shipping countries     | JP                                                           |
|                        |                                                              |
|                        |                                                              |

| Supported locales | URL                    | Country/region | Language |
| ----------------- | ---------------------- | -------------- | -------- |
| ja (selector)     | https://nas.ugreen.jp/ | JP             | JA       |

**Shipping evidence:** Status: published. Some Okinawa, outlying islands and restricted delivery areas excluded. Source: https://store.ugreen.jp/policies/shipping-policy.

**Research notes:** Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. Country/region and Language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query. The base language is declared by the storefront selector; direct page-language confirmation is unavailable in the research. Related selector URL: https://store.ugreen.jp/. Retained here as notes rather than separate retailer tables.

## Ugreen Netherlands

| Field                  | Value                                                    |
| ---------------------- | -------------------------------------------------------- |
| Retailer code          | `ugreennl`                                               |
| Storefront website     | https://nl.ugreen.com/                                   |
| Shopify shop value     | ugreen-nl.myshopify.com                                  |
| Storefront GraphQL API | https://ugreen-nl.myshopify.com/api/2026-10/graphql.json |
| Target markets         | Netherlands (NL)                                         |
| Shipping countries     | NL, BE, FR, DE, ES                                       |
|                        |                                                          |
|                        |                                                          |

| Supported locales | URL                    | Country/region | Language |
| ----------------- | ---------------------- | -------------- | -------- |
| nl (selector)     | https://nl.ugreen.com/ | NL             | NL       |

**Shipping evidence:** Status: published.  Source: https://us.ugreen.com/pages/shipping-policy.

**Research notes:** Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. Country/region and Language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query. The base language is declared by the storefront selector; direct page-language confirmation is unavailable in the research.

## Ugreen Japan

| Field                  | Value                                                    |
| ---------------------- | -------------------------------------------------------- |
| Retailer code          | `ugreenjp`                                               |
| Storefront website     | https://ugreen.jp/                                       |
| Shopify shop value     | ugreen-jp.myshopify.com                                  |
| Storefront GraphQL API | https://ugreen-jp.myshopify.com/api/2026-10/graphql.json |
| Target markets         | Japan (JP)                                               |
| Shipping countries     | Unknown                                                  |
|                        |                                                          |
|                        |                                                          |

| Supported locales | URL                | Country/region | Language |
| ----------------- | ------------------ | -------------- | -------- |
| ja                | https://ugreen.jp/ | JP             | JA       |

**Shipping evidence:** Status: unknown. No reliable enumerated shipping-country definition established in the reviewed sources.

**Research notes:** Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. Country/region and Language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query.

## Ugreen United Kingdom

| Field                  | Value                                            |
| ---------------------- | ------------------------------------------------ |
| Retailer code          | `ugreenuk`                                       |
| Storefront website     | https://uk.ugreen.com/                           |
| Shopify shop value     | Unknown                                          |
| Storefront GraphQL API | Unknown                                          |
| Target markets         | United Kingdom (GB)                              |
| Shipping countries     | GB                                               |
| GraphQL API status     | Unknown; no endpoint established in the research |
|                        |                                                  |
|                        |                                                  |

| Supported locales | URL                    | Country/region | Language |
| ----------------- | ---------------------- | -------------- | -------- |
| en (selector)     | https://uk.ugreen.com/ | GB             | EN       |

**Shipping evidence:** Status: published. Northern Ireland is included with a separate delivery timeframe. Source: https://us.ugreen.com/pages/shipping-policy.

**Research notes:** No Shopify shop domain was verified for this URL in the reviewed sources. This does not establish whether the storefront uses Shopify. No backend is inherited from a parent retailer or inferred from a URL, language or market. Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. Country/region and Language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The base language is declared by the storefront selector; direct page-language confirmation is unavailable in the research.

## Ugreen United States

| Field                  | Value                                            |
| ---------------------- | ------------------------------------------------ |
| Retailer code          | `ugreenus`                                       |
| Storefront website     | https://us.ugreen.com/                           |
| Shopify shop value     | Unknown                                          |
| Storefront GraphQL API | Unknown                                          |
| Target markets         | United States (US)                               |
| Shipping countries     | US                                               |
| GraphQL API status     | Unknown; no endpoint established in the research |
|                        |                                                  |
|                        |                                                  |

| Supported locales | URL                    | Country/region | Language |
| ----------------- | ---------------------- | -------------- | -------- |
| en (selector)     | https://us.ugreen.com/ | US             | EN       |

**Shipping evidence:** Status: published. Excludes Alaska, Hawaii, listed territories and military addresses. Source: https://us.ugreen.com/pages/shipping-policy.

**Research notes:** No Shopify shop domain was verified for this URL in the reviewed sources. This does not establish whether the storefront uses Shopify. No backend is inherited from a parent retailer or inferred from a URL, language or market. Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. Country/region and Language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The base language is declared by the storefront selector; direct page-language confirmation is unavailable in the research.

## Ugreen Main Site

| Field                  | Value                                                           |
| ---------------------- | --------------------------------------------------------------- |
| Retailer code          | `ugreencom`                                                     |
| Storefront website     | https://www.ugreen.com/                                         |
| Shopify shop value     | ugreen-brandsite.myshopify.com                                  |
| Storefront GraphQL API | https://ugreen-brandsite.myshopify.com/api/2026-10/graphql.json |
| Target markets         | United States (US)                                              |
| Shipping countries     | Unknown                                                         |
|                        |                                                                 |
|                        |                                                                 |

| Supported locales       | URL                          | Country/region | Language |
| ----------------------- | ---------------------------- | -------------- | -------- |
| en                      | https://www.ugreen.com/      | US             | EN       |
| en-US                   | https://www.ugreen.com/      | US             | EN       |
| ar (advertised only)    | https://www.ugreen.com/ar-ae | Not selected   | AR       |
| ar-SA (advertised only) | https://www.ugreen.com/ar-sa | Not selected   | AR       |
| de-DE (advertised only) | https://www.ugreen.com/de-de | Not selected   | DE       |
| en (advertised only)    | https://www.ugreen.com/en-ae | Not selected   | EN       |
| en-AU (advertised only) | https://www.ugreen.com/en-au | Not selected   | EN       |
| en-CA (advertised only) | https://www.ugreen.com/en-ca | Not selected   | EN       |
| en-DE (advertised only) | https://www.ugreen.com/en-de | Not selected   | EN       |
| en-ES (advertised only) | https://www.ugreen.com/en-es | Not selected   | EN       |
| en-AT (advertised only) | https://www.ugreen.com/en-eu | Not selected   | EN       |
| en-BE (advertised only) | https://www.ugreen.com/en-eu | Not selected   | EN       |
| en-BG (advertised only) | https://www.ugreen.com/en-eu | Not selected   | EN       |
| en-CH (advertised only) | https://www.ugreen.com/en-eu | Not selected   | EN       |
| en-CY (advertised only) | https://www.ugreen.com/en-eu | Not selected   | EN       |
| en-CZ (advertised only) | https://www.ugreen.com/en-eu | Not selected   | EN       |
| en-DK (advertised only) | https://www.ugreen.com/en-eu | Not selected   | EN       |
| en-EE (advertised only) | https://www.ugreen.com/en-eu | Not selected   | EN       |
| en-FI (advertised only) | https://www.ugreen.com/en-eu | Not selected   | EN       |
| en-HR (advertised only) | https://www.ugreen.com/en-eu | Not selected   | EN       |
| en-HU (advertised only) | https://www.ugreen.com/en-eu | Not selected   | EN       |
| en-IS (advertised only) | https://www.ugreen.com/en-eu | Not selected   | EN       |
| en-LT (advertised only) | https://www.ugreen.com/en-eu | Not selected   | EN       |
| en-LU (advertised only) | https://www.ugreen.com/en-eu | Not selected   | EN       |
| en-LV (advertised only) | https://www.ugreen.com/en-eu | Not selected   | EN       |
| en-MT (advertised only) | https://www.ugreen.com/en-eu | Not selected   | EN       |
| en-NO (advertised only) | https://www.ugreen.com/en-eu | Not selected   | EN       |
| en-PL (advertised only) | https://www.ugreen.com/en-eu | Not selected   | EN       |
| en-PT (advertised only) | https://www.ugreen.com/en-eu | Not selected   | EN       |
| en-RO (advertised only) | https://www.ugreen.com/en-eu | Not selected   | EN       |
| en-SE (advertised only) | https://www.ugreen.com/en-eu | Not selected   | EN       |
| en-SI (advertised only) | https://www.ugreen.com/en-eu | Not selected   | EN       |
| en-SK (advertised only) | https://www.ugreen.com/en-eu | Not selected   | EN       |
| en-FR (advertised only) | https://www.ugreen.com/en-fr | Not selected   | EN       |
| en-GB (advertised only) | https://www.ugreen.com/en-gb | Not selected   | EN       |
| en-ID (advertised only) | https://www.ugreen.com/en-id | Not selected   | EN       |
| en-IT (advertised only) | https://www.ugreen.com/en-it | Not selected   | EN       |
| en-MY (advertised only) | https://www.ugreen.com/en-my | Not selected   | EN       |
| en-NL (advertised only) | https://www.ugreen.com/en-nl | Not selected   | EN       |
| en-PH (advertised only) | https://www.ugreen.com/en-ph | Not selected   | EN       |
| en-SA (advertised only) | https://www.ugreen.com/en-sa | Not selected   | EN       |
| en-SG (advertised only) | https://www.ugreen.com/en-sg | Not selected   | EN       |
| en-TH (advertised only) | https://www.ugreen.com/en-th | Not selected   | EN       |
| en-TR (advertised only) | https://www.ugreen.com/en-tr | Not selected   | EN       |
| en-VN (advertised only) | https://www.ugreen.com/en-vn | Not selected   | EN       |
| es-ES (advertised only) | https://www.ugreen.com/es-es | Not selected   | ES       |
| fr-FR (advertised only) | https://www.ugreen.com/fr-fr | Not selected   | FR       |
| id-ID (advertised only) | https://www.ugreen.com/id-id | Not selected   | ID       |
| it-IT (advertised only) | https://www.ugreen.com/it-it | Not selected   | IT       |
| nl-US (advertised only) | https://www.ugreen.com/nl    | Not selected   | NL       |
| nl-NL (advertised only) | https://www.ugreen.com/nl-nl | Not selected   | NL       |
| th-TH (advertised only) | https://www.ugreen.com/th-th | Not selected   | TH       |
| tr-US (advertised only) | https://www.ugreen.com/tr    | Not selected   | TR       |
| tr-TR (advertised only) | https://www.ugreen.com/tr-tr | Not selected   | TR       |
| vi-US (advertised only) | https://www.ugreen.com/vi    | Not selected   | VI       |
| vi-VN (advertised only) | https://www.ugreen.com/vi-vn | Not selected   | VI       |

**Shipping evidence:** Status: unknown. No reliable enumerated shipping-country definition established in the reviewed sources.

**Research notes:** Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. Country/region and Language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query. Rows marked advertised only are alternate links, not verified locale pages or confirmed extra content requests. Their country contexts remain unselected, and the base Shopify backend has not been established for those URLs.

## Ugreen India

| Field                  | Value                                                    |
| ---------------------- | -------------------------------------------------------- |
| Retailer code          | `ugreenin`                                               |
| Storefront website     | https://www.ugreenindia.com/                             |
| Shopify shop value     | 990f9c-b7.myshopify.com                                  |
| Storefront GraphQL API | https://990f9c-b7.myshopify.com/api/2026-10/graphql.json |
| Target markets         | India (IN)                                               |
| Shipping countries     | IN                                                       |
|                        |                                                          |
|                        |                                                          |

| Supported locales    | URL                          | Country/region | Language |
| -------------------- | ---------------------------- | -------------- | -------- |
| en                   | https://www.ugreenindia.com/ | IN             | EN       |
| en (advertised only) | https://ugreenindia.com/     | Not selected   | EN       |
| hi (advertised only) | https://ugreenindia.com/hi   | Not selected   | HI       |

**Shipping evidence:** Status: published. India-wide B2C policy; postcode/carrier restrictions apply. Source: https://ugreenindia.com/policies/shipping-policy.

**Research notes:** Store explicitly labels India; Hindi alternate is separately advertised. Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. Country/region and Language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query. Rows marked advertised only are alternate links, not verified locale pages or confirmed extra content requests. Their country contexts remain unselected, and the base Shopify backend has not been established for those URLs.
