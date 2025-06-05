use std::{str::FromStr, sync::Arc, time::Duration};

use moka::future::Cache;
use namada_sdk::{address::Address, rpc};
use tendermint_rpc::HttpClient;

const TOTAL_SUPPLY_CACHE_KEY: &str = "total_supply";
const EFFECTIVE_SUPPLY_CACHE_KEY: &str = "effective_supply";
const CIRCULATING_SUPPLY_CACHE_KEY: &str = "circulating_supply";

const NON_CIRC_ADDRESSES: &[&str] = &[
    "tnam1qxdzup2hcvhswcgw5kerd5lfkf04t64y3scgqm5v",
    "tnam1qxt7uxhj9r00mfm4u870e7ghz6j20jrdz58gm5kj",
    "tnam1qyez9fd9nkaxfj4u2f2k0vavr8mm69azcgds45rr",
    "tnam1qqp69rzwsgnqdm0d4qfhw4qa4s6v3tlzm5069f4j",
    "tnam1qrucghh3hw2zq8xtqzdj44nh5nrmnkn0usqng8yq",
];

#[derive(Clone)]
pub struct Client {
    pub client: HttpClient,
    native_token: Address,
    cache: Arc<Cache<String, String>>,
}

impl Client {
    pub async fn new(client: HttpClient) -> Self {
        let native_token = rpc::query_native_token(&client)
            .await
            .expect("Failed to query native token");

        Self {
            client,
            cache: Arc::new(
                Cache::builder()
                    .time_to_live(Duration::from_secs(60))
                    .build(),
            ),
            native_token,
        }
    }

    pub async fn get_native_total_supply(&mut self) -> Result<String, String> {
        let supply = if let Some(supply) = self.cache.get(TOTAL_SUPPLY_CACHE_KEY).await {
            tracing::debug!("Cache hit for total supply");
            supply.clone()
        } else {
            let supply = rpc::get_token_total_supply(&self.client, &self.native_token)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to get total supply: {}", e);
                    "RPC Timeout".to_string()
                })?;
            self.cache
                .insert(
                    TOTAL_SUPPLY_CACHE_KEY.to_string(),
                    supply.to_string_native(),
                )
                .await;
            supply.to_string_native()
        };

        Ok(supply)
    }

    pub async fn get_effective_total_supply(&mut self) -> Result<String, String> {
        let supply = if let Some(supply) = self.cache.get(EFFECTIVE_SUPPLY_CACHE_KEY).await {
            tracing::debug!("Cache hit for effective supply");
            supply.clone()
        } else {
            let supply = rpc::get_effective_native_supply(&self.client)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to get effective supply: {}", e);
                    "RPC Timeout".to_string()
                })?;
            self.cache
                .insert(
                    EFFECTIVE_SUPPLY_CACHE_KEY.to_string(),
                    supply.to_string_native(),
                )
                .await;
            supply.to_string_native()
        };

        Ok(supply)
    }

    pub async fn get_circulating_supply(&mut self) -> Result<String, String> {
        let supply = if let Some(supply) = self.cache.get(CIRCULATING_SUPPLY_CACHE_KEY).await {
            tracing::debug!("Cache hit for circulating supply");
            supply.clone()
        } else {
            let mut circ_supply = rpc::get_effective_native_supply(&self.client)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to get circulating supply: {}", e);
                    "RPC Timeout".to_string()
                })?;

            for addr in NON_CIRC_ADDRESSES {
                let addr = Address::from_str(addr).unwrap();
                let balance = rpc::get_token_balance(&self.client, &self.native_token, &addr, None)
                    .await
                    .map_err(|e| {
                        tracing::error!("Failed to get balance of address {}: {}", addr, e);
                        "RPC Timeout".to_string()
                    })?;
                circ_supply = circ_supply.checked_sub(balance).unwrap();
            }

            self.cache
                .insert(
                    CIRCULATING_SUPPLY_CACHE_KEY.to_string(),
                    circ_supply.to_string_native(),
                )
                .await;
            circ_supply.to_string_native()
        };

        Ok(supply)
    }
}
