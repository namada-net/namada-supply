use std::{sync::Arc, time::Duration};

use moka::future::Cache;
use namada_sdk::{address::Address, rpc};
use tendermint_rpc::HttpClient;

const TOTAL_SUPPLY_CACHE_KEY: &str = "total_supply";
const EFFECTIVE_SUPPLY_CACHE_KEY: &str = "effective_supply";

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
                    tracing::error!("Failed to get total supply: {}", e);
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
}
