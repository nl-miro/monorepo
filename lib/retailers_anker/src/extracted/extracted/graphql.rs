//! Commented GraphQL artifact models and their conversions.

// use retailer_sauce::shopify::initial_fill::GraphqlRequest;

// #[derive(Deserialize)]
// struct ArtifactWire {
//     acquired_at: Option<String>,
//     schema_version: u32,
//     context: Option<retailer_sauce::shopify::markets::MarketContext>,
//     product: Product,
//     pages: Vec<RecordedPage>,
// }
// impl TryFrom<ArtifactWire> for ExtractedOfferDetails {
//     type Error = String;
//     fn try_from(wire: ArtifactWire) -> Result<Self, String> {
//         if !matches!(
//             (wire.schema_version, wire.context.is_some()),
//             (1, false) | (2, true)
//         ) {
//             return Err("unsupported artifact version or missing/inconsistent context".into());
//         }
//         Ok(Self {
//             schema_version: wire.schema_version,
//             acquired_at: wire.acquired_at,
//             context: wire.context,
//             product: wire.product,
//             pages: wire.pages,
//         })
//     }
// }

// /// Pages use different GraphQL selections. Keep their diagnostic evidence verbatim;
// /// `ProductResponse` models a full initial response and `Product` the merged result.
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct RecordedPage {
//     pub request: GraphqlRequest,
//     pub response: Value,
// }

// /// A full OfferDetails GraphQL response (including partial-data/error responses).
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct ProductResponse {
//     pub data: Option<ProductData>,
//     pub errors: Option<Vec<GraphqlError>>,
//     /// Shopify diagnostic/cost metadata, outside the product source contract.
//     pub extensions: Option<Value>,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct ProductData {
//     pub product: Option<Product>,
//     /// The saved data3 crawl queried full products through this connection.
//     pub products: Vec<Product>,
//     pub localization: Option<Localization>,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct Localization {
//     pub country: Country,
//     pub language: IsoCode,
// }
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct Country {
//     pub iso_code: String,
//     pub currency: IsoCode,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct GraphqlError {
//     pub message: String,
//     pub locations: Option<Vec<ErrorLocation>>,
//     pub path: Option<Vec<ErrorPath>>,
//     pub extensions: Option<Value>,
// }
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct ErrorLocation {
//     pub line: u32,
//     pub column: u32,
// }
// #[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(untagged)]
// pub enum ErrorPath {
//     Field(String),
//     Index(u64),
// }

// impl TryFrom<raw::RecordedPage> for RecordedPage {
//     type Error = String;
//     fn try_from(value: raw::RecordedPage) -> Result<Self, Self::Error> {
//         let raw::RecordedPage { request, response } = value;
//         Ok(Self { request, response })
//     }
// }
// impl TryFrom<raw::ProductResponse> for ProductResponse {
//     type Error = String;
//     fn try_from(value: raw::ProductResponse) -> Result<Self, Self::Error> {
//         let raw::ProductResponse {
//             data,
//             errors,
//             extensions,
//         } = value;
//         Ok(Self {
//             data: opt(data)?,
//             errors: errors.map(vec).transpose()?,
//             extensions,
//         })
//     }
// }
// impl TryFrom<raw::ProductData> for ProductData {
//     type Error = String;
//     fn try_from(value: raw::ProductData) -> Result<Self, Self::Error> {
//         let raw::ProductData {
//             product,
//             products,
//             localization,
//         } = value;
//         Ok(Self {
//             product: opt(product)?,
//             products: products
//                 .into_iter()
//                 .flat_map(|connection| connection.edges)
//                 .map(|edge| Product::try_from(edge.node))
//                 .collect::<Result<Vec<_>, String>>()?,
//             localization: opt(localization)?,
//         })
//     }
// }
// impl TryFrom<raw::Localization> for Localization {
//     type Error = String;
//     fn try_from(value: raw::Localization) -> Result<Self, Self::Error> {
//         let raw::Localization { country, language } = value;
//         Ok(Self {
//             country: Country::try_from(country)?,
//             language: IsoCode::try_from(language)?,
//         })
//     }
// }
// impl TryFrom<raw::Country> for Country {
//     type Error = String;
//     fn try_from(value: raw::Country) -> Result<Self, Self::Error> {
//         let raw::Country { iso_code, currency } = value;
//         Ok(Self {
//             iso_code,
//             currency: IsoCode::try_from(currency)?,
//         })
//     }
// }

// impl TryFrom<raw::GraphqlError> for GraphqlError {
//     type Error = String;
//     fn try_from(value: raw::GraphqlError) -> Result<Self, Self::Error> {
//         let raw::GraphqlError {
//             message,
//             locations,
//             path,
//             extensions,
//         } = value;
//         Ok(Self {
//             message,
//             locations: locations.map(vec).transpose()?,
//             path: path.map(vec).transpose()?,
//             extensions,
//         })
//     }
// }
// impl TryFrom<raw::ErrorLocation> for ErrorLocation {
//     type Error = String;
//     fn try_from(value: raw::ErrorLocation) -> Result<Self, Self::Error> {
//         let raw::ErrorLocation { line, column } = value;
//         Ok(Self { line, column })
//     }
// }
// impl TryFrom<raw::ErrorPath> for ErrorPath {
//     type Error = String;
//     fn try_from(value: raw::ErrorPath) -> Result<Self, Self::Error> {
//         Ok(match value {
//             raw::ErrorPath::Field(field) => Self::Field(field),
//             raw::ErrorPath::Index(index) => Self::Index(index),
//         })
//     }
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct Connection<T> {
//     pub page_info: PageInfo,
//     pub edges: Vec<Edge<T>>,
// }
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct Edge<T> {
//     pub cursor: String,
//     pub node: T,
// }
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct PageInfo {
//     pub has_next_page: bool,
//     pub end_cursor: Option<String>,
// }

// impl<T, U: TryFrom<T, Error = String>> TryFrom<raw::Connection<T>> for Connection<U> {
//     type Error = String;
//     fn try_from(value: raw::Connection<T>) -> Result<Self, Self::Error> {
//         let raw::Connection { page_info, edges } = value;
//         Ok(Self {
//             page_info: PageInfo::try_from(page_info)?,
//             edges: vec(edges)?,
//         })
//     }
// }
// impl<T, U: TryFrom<T, Error = String>> TryFrom<raw::Edge<T>> for Edge<U> {
//     type Error = String;
//     fn try_from(value: raw::Edge<T>) -> Result<Self, Self::Error> {
//         let raw::Edge { cursor, node } = value;
//         Ok(Self {
//             cursor,
//             node: U::try_from(node)?,
//         })
//     }
// }
// impl TryFrom<raw::PageInfo> for PageInfo {
//     type Error = String;
//     fn try_from(value: raw::PageInfo) -> Result<Self, Self::Error> {
//         let raw::PageInfo {
//             has_next_page,
//             end_cursor,
//         } = value;
//         Ok(Self {
//             has_next_page,
//             end_cursor,
//         })
//     }
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct IsoCode {
//     pub iso_code: String,
// }

// impl TryFrom<raw::IsoCode> for IsoCode {
//     type Error = String;
//     fn try_from(value: raw::IsoCode) -> Result<Self, Self::Error> {
//         let raw::IsoCode { iso_code } = value;
//         Ok(Self { iso_code })
//     }
// }
