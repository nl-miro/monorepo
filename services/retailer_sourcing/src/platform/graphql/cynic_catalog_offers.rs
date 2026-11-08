// Generated from catalog_offers.graphql with cynic-querygen 3.14.0.
// The operation directive is applied by the caller.
#![allow(dead_code)]
use super::super::schema;

#[derive(cynic::QueryVariables, Debug)]
pub struct CatalogOffersVariables<'a> {
    pub after: Option<&'a str>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/schema.graphql")]
#[cynic(graphql_type = "QueryRoot", variables = "CatalogOffersVariables")]
pub struct CatalogOffers {
    #[arguments(first: 250, after: $after, sortKey: ID)]
    pub products: ProductConnection,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/schema.graphql")]
pub struct ProductConnection {
    pub page_info: PageInfo,
    pub nodes: Vec<Product>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/schema.graphql")]
pub struct Product {
    pub id: cynic::Id,
    pub available_for_sale: bool,
    pub price_range: ProductPriceRange,
    #[arguments(first: 5)]
    pub variants: ProductVariantConnection,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/schema.graphql")]
pub struct ProductVariantConnection {
    pub page_info: PageInfo,
    pub edges: Vec<ProductVariantEdge>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/schema.graphql")]
pub struct ProductVariantEdge {
    pub cursor: String,
    pub node: ProductVariant,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/schema.graphql")]
pub struct ProductVariant {
    pub id: cynic::Id,
    pub price: MoneyV2,
    pub available_for_sale: bool,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/schema.graphql")]
pub struct MoneyV2 {
    pub amount: Decimal,
    pub currency_code: CurrencyCode,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/schema.graphql")]
pub struct PageInfo {
    pub has_next_page: bool,
    pub end_cursor: Option<String>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/schema.graphql")]
pub struct ProductPriceRange {
    pub min_variant_price: MoneyV22,
    pub max_variant_price: MoneyV23,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/schema.graphql")]
#[cynic(graphql_type = "MoneyV2")]
pub struct MoneyV23 {
    pub amount: Decimal,
    pub currency_code: CurrencyCode,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/schema.graphql")]
#[cynic(graphql_type = "MoneyV2")]
pub struct MoneyV22 {
    pub amount: Decimal,
    pub currency_code: CurrencyCode,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
#[cynic(schema_path = "src/platform/graphql/schema.graphql")]
pub enum CurrencyCode {
    Usd,
    Eur,
    Gbp,
    Cad,
    Afn,
    All,
    Dzd,
    Aoa,
    Ars,
    Amd,
    Awg,
    Aud,
    Bbd,
    Azn,
    Bdt,
    Bsd,
    Bhd,
    Bif,
    Bzd,
    Bmd,
    Btn,
    Bam,
    Brl,
    Bob,
    Bwp,
    Bnd,
    Bgn,
    Mmk,
    Khr,
    Cve,
    Xcg,
    Kyd,
    Xaf,
    Clp,
    Cny,
    Cop,
    Kmf,
    Cdf,
    Crc,
    Hrk,
    Czk,
    Dkk,
    Dop,
    Xcd,
    Egp,
    Ern,
    Etb,
    Fkp,
    Xpf,
    Fjd,
    Gip,
    Gmd,
    Ghs,
    Gtq,
    Gyd,
    Gel,
    Htg,
    Hnl,
    Hkd,
    Huf,
    Isk,
    Inr,
    Idr,
    Ils,
    Iqd,
    Jmd,
    Jpy,
    Jep,
    Jod,
    Kzt,
    Kes,
    Kid,
    Kwd,
    Kgs,
    Lak,
    Lvl,
    Lbp,
    Lsl,
    Lrd,
    Ltl,
    Mga,
    Mkd,
    Mop,
    Mwk,
    Mvr,
    Mru,
    Mxn,
    Myr,
    Mur,
    Mdl,
    Mad,
    Mnt,
    Mzn,
    Nad,
    Npr,
    Ang,
    Nzd,
    Nio,
    Ngn,
    Nok,
    Omr,
    Pab,
    Pkr,
    Pgk,
    Pyg,
    Pen,
    Php,
    Pln,
    Qar,
    Ron,
    Rub,
    Rwf,
    Wst,
    Shp,
    Sar,
    Rsd,
    Scr,
    Sgd,
    Sdg,
    Sos,
    Syp,
    Zar,
    Krw,
    Ssp,
    Sbd,
    Lkr,
    Srd,
    Szl,
    Sek,
    Chf,
    Twd,
    Thb,
    Tzs,
    Ttd,
    Tnd,
    Try,
    Tmt,
    Ugx,
    Uah,
    Aed,
    Uyu,
    Uzs,
    Vuv,
    Ves,
    Vnd,
    Xof,
    Yer,
    Zmw,
    Byn,
    Byr,
    Djf,
    Gnf,
    Irr,
    Lyd,
    Sll,
    Std,
    Stn,
    Tjs,
    Top,
    Ved,
    Vef,
    Xxx,
}

#[derive(cynic::Scalar, Debug, Clone)]
pub struct Decimal(pub String);
