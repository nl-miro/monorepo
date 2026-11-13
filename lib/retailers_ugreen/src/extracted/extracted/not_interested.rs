//! Commented source types omitted from extracted output and their conversions.

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct PriceRange {
//     // #[serde(rename = "__typename")]
//     // pub type_name: String,
//     pub max_variant_price: Money,
//     pub min_variant_price: Money,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct Count {
//     // #[serde(rename = "__typename")]
//     // pub type_name: String,
//     pub count: i64,
//     pub precision: String,
// }

// impl TryFrom<raw::PriceRange> for PriceRange {
//     type Error = String;
//     fn try_from(value: raw::PriceRange) -> Result<Self, Self::Error> {
//         let raw::PriceRange {
//             type_name: _,
//             max_variant_price,
//             min_variant_price,
//         } = value;
//         Ok(Self {
//             // type_name,
//             max_variant_price: Money::try_from(max_variant_price)?,
//             min_variant_price: Money::try_from(min_variant_price)?,
//         })
//     }
// }

// impl TryFrom<raw::Count> for Count {
//     type Error = String;
//     fn try_from(value: raw::Count) -> Result<Self, Self::Error> {
//         let raw::Count {
//             type_name: _,
//             count,
//             precision,
//         } = value;
//         Ok(Self {
//             // type_name,
//             count,
//             precision,
//         })
//     }
// }
