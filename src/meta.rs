use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Meta {
    pub universe: Vec<AssetMeta>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AssetMeta {
    pub max_leverage: u32,
    pub name: String,
    pub only_isolated: bool,
    pub sz_decimals: u32,
}
