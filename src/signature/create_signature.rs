use ethers::{
    abi::AbiEncode,
    core::k256::{
        ecdsa::{recoverable, signature::DigestSigner},
        elliptic_curve::FieldBytes,
        Secp256k1,
    },
    signers::LocalWallet,
    types::{transaction::eip712::Eip712, Signature, H256, U256},
    utils::keccak256,
};

use crate::{
    helpers::EthChain,
    prelude::*,
    proxy_digest::Sha256Proxy,
    signature::{
        agent::{l1, mainnet, testnet},
        usdc_transfer,
    },
    Error,
};

pub(crate) fn keccak(x: impl AbiEncode) -> H256 {
    keccak256(x.encode()).into()
}

pub(crate) fn sign_l1_action(
    wallet: &LocalWallet,
    connection_id: H256,
    is_mainnet: bool,
) -> Result<Signature> {
    sign_with_agent(
        wallet,
        EthChain::Localhost,
        if is_mainnet { "a" } else { "b" },
        connection_id,
    )
}

pub(crate) fn sign_usd_transfer_action(
    wallet: &LocalWallet,
    chain_type: EthChain,
    amount: &str,
    destination: &str,
    timestamp: u64,
) -> Result<Signature> {
    match chain_type {
        EthChain::Localhost => Err(Error::ChainNotAllowed),
        EthChain::Arbitrum => Ok(sign_typed_data(
            &usdc_transfer::mainnet::UsdTransferSignPayload {
                destination: destination.to_string(),
                amount: amount.to_string(),
                time: timestamp,
            },
            wallet,
        )?),
        EthChain::ArbitrumGoerli => Ok(sign_typed_data(
            &usdc_transfer::testnet::UsdTransferSignPayload {
                destination: destination.to_string(),
                amount: amount.to_string(),
                time: timestamp,
            },
            wallet,
        )?),
    }
}

pub(crate) fn sign_with_agent(
    wallet: &LocalWallet,
    chain_type: EthChain,
    source: &str,
    connection_id: H256,
) -> Result<Signature> {
    match chain_type {
        EthChain::Localhost => sign_typed_data(
            &l1::Agent {
                source: source.to_string(),
                connection_id,
            },
            wallet,
        ),
        EthChain::Arbitrum => sign_typed_data(
            &mainnet::Agent {
                source: source.to_string(),
                connection_id,
            },
            wallet,
        ),
        EthChain::ArbitrumGoerli => sign_typed_data(
            &testnet::Agent {
                source: source.to_string(),
                connection_id,
            },
            wallet,
        ),
    }
}

fn sign_typed_data<T: Eip712>(payload: &T, wallet: &LocalWallet) -> Result<Signature> {
    let encoded = payload
        .encode_eip712()
        .map_err(|e| Error::Eip712(e.to_string()))?;

    Ok(sign_hash(H256::from(encoded), wallet))
}

fn sign_hash(hash: H256, wallet: &LocalWallet) -> Signature {
    let recoverable_sig: recoverable::Signature =
        wallet.signer().sign_digest(Sha256Proxy::from(hash));

    let v = u8::from(recoverable_sig.recovery_id()) as u64 + 27;

    let r_bytes: FieldBytes<Secp256k1> = recoverable_sig.r().into();
    let s_bytes: FieldBytes<Secp256k1> = recoverable_sig.s().into();
    let r = U256::from_big_endian(r_bytes.as_slice());
    let s = U256::from_big_endian(s_bytes.as_slice());

    Signature { r, s, v }
}

#[cfg(test)]
mod tests {
    use ethers::types::H160;

    use super::*;
    use std::str::FromStr;

    fn get_wallet() -> Result<LocalWallet> {
        let priv_key = "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e";
        priv_key
            .parse::<LocalWallet>()
            .map_err(|e| Error::Wallet(e.to_string()))
    }
    #[test]
    fn test_sign_l1_action() -> Result<()> {
        let wallet = get_wallet()?;
        let connection_id =
            H256::from_str("0xde6c4037798a4434ca03cd05f00e3b803126221375cd1e7eaaaf041768be06eb")
                .map_err(|e| Error::GenericParse(e.to_string()))?;

        let expected_mainnet_sig = "fa8a41f6a3fa728206df80801a83bcbfbab08649cd34d9c0bfba7c7b2f99340f53a00226604567b98a1492803190d65a201d6805e5831b7044f17fd530aec7841c";
        assert_eq!(
            sign_l1_action(&wallet, connection_id, true)?.to_string(),
            expected_mainnet_sig
        );
        let expected_testnet_sig = "1713c0fc661b792a50e8ffdd59b637b1ed172d9a3aa4d801d9d88646710fb74b33959f4d075a7ccbec9f2374a6da21ffa4448d58d0413a0d335775f680a881431c";
        assert_eq!(
            sign_l1_action(&wallet, connection_id, false)?.to_string(),
            expected_testnet_sig
        );
        Ok(())
    }

    #[test]
    fn test_sign_usd_transfer_action() -> Result<()> {
        let wallet = get_wallet()?;

        let chain_type = EthChain::ArbitrumGoerli;
        let amount = "1";
        let destination = "0x0D1d9635D0640821d15e323ac8AdADfA9c111414";
        let timestamp = 1690393044548;

        let expected_sig = "78f879e7ae6fbc3184dc304317e602507ac562b49ad9a5db120a41ac181b96ba2e8bd7022526a1827cf4b0ba96384d40ec3a5ed4239499c328081f3d0b394bb61b";
        assert_eq!(
            sign_usd_transfer_action(&wallet, chain_type, amount, destination, timestamp)?
                .to_string(),
            expected_sig
        );
        Ok(())
    }

    #[test]
    fn test_sign_ref() -> Result<()> {
        let priv_key = "738786125b186eddbdf358dd55308f1c05a12b9b638643aaae37c7d1be99a42e";
        let wallet = priv_key
            .parse::<LocalWallet>()
            .map_err(|e| Error::Wallet(e.to_string()))
            .unwrap();
        let timestamp: u64 = 1702655843561;
        let vault_address = H160::default();
        let user = H160::from_str("0x9D974aEd2EC4eFBb866750cceb6be42eD792e793").unwrap();
        let connection_id1 = keccak(("OSCAR".to_string(), user));
        let connection_id2 = keccak((vault_address, timestamp));
        let connection_id3 = keccak(timestamp);
        let connection_id4 = keccak((true, timestamp));
        let connection_id5 = keccak((true, vault_address, timestamp));
        let connection_id6 = keccak((vault_address, true, timestamp));
        let connection_id7 = keccak(H160::default());
        let connection_id8 = keccak((user, "OSCAR".to_string()));
        let connection_id9 = keccak((user, "OSCAR".to_string(), vault_address, timestamp));
        let connection_id0 = keccak((
            user,
            "string".to_string(),
            "OSCAR".to_string(),
            vault_address,
            timestamp,
        ));
        // let expected_mainnet_sig = "fa8a41f6a3fa728206df80801a83bcbfbab08649cd34d9c0bfba7c7b2f99340f53a00226604567b98a1492803190d65a201d6805e5831b7044f17fd530aec7841c";
        let sig1l1 = sign_l1_action(&wallet, connection_id1, true)?;
        let sig2l1 = sign_l1_action(&wallet, connection_id8, true)?;
        // let sig3 = sign_l1_action(&wallet, connection_id3, true)?;
        // let sig4 = sign_l1_action(&wallet, connection_id4, true)?;
        // let sig5 = sign_l1_action(&wallet, connection_id5, true)?;
        // let sig6 = sign_l1_action(&wallet, connection_id6, true)?;
        // let sig7 = sign_l1_action(&wallet, connection_id7, true)?;
        let sig1 = sign_with_agent(
            &wallet,
            EthChain::Arbitrum,
            "https://hyperliquid.xyz",
            connection_id1,
        )?;
        let sig2 = sign_with_agent(
            &wallet,
            EthChain::Arbitrum,
            "https://hyperliquid.xyz",
            connection_id2,
        )?;
        let sig3 = sign_with_agent(
            &wallet,
            EthChain::Arbitrum,
            "https://hyperliquid.xyz",
            connection_id3,
        )?;
        let sig4 = sign_with_agent(
            &wallet,
            EthChain::Arbitrum,
            "https://hyperliquid.xyz",
            connection_id4,
        )?;
        let sig5 = sign_with_agent(
            &wallet,
            EthChain::Arbitrum,
            "https://hyperliquid.xyz",
            connection_id5,
        )?;
        let sig6 = sign_with_agent(
            &wallet,
            EthChain::Arbitrum,
            "https://hyperliquid.xyz",
            connection_id6,
        )?;
        let sig7: Signature = sign_with_agent(
            &wallet,
            EthChain::Arbitrum,
            "https://hyperliquid.xyz",
            connection_id7,
        )?;
        let sig8: Signature = sign_with_agent(
            &wallet,
            EthChain::Arbitrum,
            "https://hyperliquid.xyz",
            connection_id8,
        )?;
        let sig9: Signature = sign_with_agent(
            &wallet,
            EthChain::Arbitrum,
            "https://hyperliquid.xyz",
            connection_id9,
        )?;
        let sig0: Signature = sign_with_agent(
            &wallet,
            EthChain::Arbitrum,
            "https://hyperliquid.xyz",
            connection_id0,
        )?;

        println!(
            "Sig1: {:?}",
            serde_json::to_string(&sig1).map_err(|e| Error::JsonParse(e.to_string()))?,
        );
        println!(
            "Sig2: {:?}",
            serde_json::to_string(&sig2).map_err(|e| Error::JsonParse(e.to_string()))?,
        );
        println!(
            "Sig3: {:?}",
            serde_json::to_string(&sig3).map_err(|e| Error::JsonParse(e.to_string()))?,
        );
        println!(
            "Sig4: {:?}",
            serde_json::to_string(&sig4).map_err(|e| Error::JsonParse(e.to_string()))?,
        );
        println!(
            "Sig5: {:?}",
            serde_json::to_string(&sig5).map_err(|e| Error::JsonParse(e.to_string()))?,
        );
        println!(
            "Sig6: {:?}",
            serde_json::to_string(&sig6).map_err(|e| Error::JsonParse(e.to_string()))?,
        );
        println!(
            "Sig7: {:?}",
            serde_json::to_string(&sig7).map_err(|e| Error::JsonParse(e.to_string()))?,
        );
        println!(
            "Sig8: {:?}",
            serde_json::to_string(&sig8).map_err(|e| Error::JsonParse(e.to_string()))?,
        );
        println!(
            "Sig9: {:?}",
            serde_json::to_string(&sig9).map_err(|e| Error::JsonParse(e.to_string()))?,
        );
        println!(
            "Sig0: {:?}",
            serde_json::to_string(&sig0).map_err(|e| Error::JsonParse(e.to_string()))?,
        );
        println!(
            "Sig1l1: {:?}",
            serde_json::to_string(&sig1l1).map_err(|e| Error::JsonParse(e.to_string()))?,
        );
        println!(
            "Sig8l1: {:?}",
            serde_json::to_string(&sig2l1).map_err(|e| Error::JsonParse(e.to_string()))?,
        );
        let expected_sig_r = "0xddf593e5d8cfe48de8581f3f43d3e32ad8804ba90e155dd79f7b9f6f75b2a29c";
        let expected_sig_s = "0x71c744fb9f2f7ab96adfddbd214a72cec9a61001f84fc6097c5afad16ae8a064";
        let expected_sig_v = 27;
        println!("Expected Sig R: {}", expected_sig_r);
        println!("Expected Sig S: {}", expected_sig_s);
        println!("Expected Sig V: {}", expected_sig_v);

        Ok(())
    }
}
