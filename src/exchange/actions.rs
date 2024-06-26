use crate::{
    exchange::{cancel::CancelRequest, order::OrderRequest},
    signature::agent::mainnet::Agent,
};
use ethers::types::H160;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UsdcTransfer {
    pub chain: String,
    pub payload: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLeverage {
    pub asset: u32,
    pub is_cross: bool,
    pub leverage: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateIsolatedMargin {
    pub asset: u32,
    pub is_buy: bool,
    pub ntli: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BulkOrder {
    pub grouping: String,
    pub orders: Vec<OrderRequest>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BulkCancel {
    pub cancels: Vec<CancelRequest>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AgentConnect {
    pub chain: String,
    pub agent: Agent,
    pub agent_address: H160,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SetReferrer {
    pub code: String,
}

#[cfg(test)]
mod tests {
    use crate::{Actions, SetReferrer};

    #[test]
    fn test_set_referrer() {
        let ref_code = Actions::SetReferrer(SetReferrer {
            code: "OSCAR".to_string(),
        });
        let action = serde_json::to_value(ref_code);
        println!("{:?}", action);
    }
}
