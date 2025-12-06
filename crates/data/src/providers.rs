use crate::MarketDataProvider;
use anyhow::Result;
use async_trait::async_trait;
use clmm_lp_domain::entities::price_candle::PriceCandle;
use clmm_lp_domain::entities::token::Token;
use clmm_lp_domain::value_objects::{amount::Amount, price::Price};
use primitive_types::U256;
use reqwest::Client;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct BirdeyeOhlcvResponse {
    data: BirdeyeData,
    success: bool,
}

#[derive(Deserialize, Debug)]
struct BirdeyeData {
    items: Vec<BirdeyeCandle>,
}

#[derive(Deserialize, Debug)]
struct BirdeyeCandle {
    o: f64,
    h: f64,
    l: f64,
    c: f64,
    v: f64,
    #[serde(rename = "unixTime")]
    unix_time: u64,
}

/// Provider for Birdeye API.
pub struct BirdeyeProvider {
    /// The HTTP client.
    pub client: Client,
    /// The API key.
    pub api_key: String,
}

impl BirdeyeProvider {
    /// Creates a new BirdeyeProvider.
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    fn map_resolution(&self, seconds: u64) -> &'static str {
        match seconds {
            60 => "1m",
            180 => "3m",
            300 => "5m",
            900 => "15m",
            1800 => "30m",
            3600 => "1h",
            7200 => "2h",
            14400 => "4h",
            43200 => "12h",
            86400 => "1d",
            _ => "1h", // Default fallback
        }
    }
}

#[async_trait]
impl MarketDataProvider for BirdeyeProvider {
    async fn get_price_history(
        &self,
        token_a: &Token,
        token_b: &Token,
        start_time: u64,
        end_time: u64,
        resolution: u64,
    ) -> Result<Vec<PriceCandle>> {
        // Birdeye API typically provides data per token vs USD.
        // If we want Token A / Token B price, we might need to fetch both and compute ratio,
        // or find a specific pair endpoint.
        // The standard public-api.birdeye.so/defi/ohlcv takes an 'address'.
        // If 'address' is a Token Mint, it returns Token/USD (usually USDC).
        // If 'address' is a Pair Address? Birdeye docs usually reference Token address.

        // Strategy:
        // 1. Fetch Token A vs USD candles.
        // 2. Fetch Token B vs USD candles.
        // 3. Synthesize Pair candles.
        // This is expensive (2 requests) but robust if direct pair data isn't available or reliable.

        // HOWEVER, for simplicity in this iteration, assuming Token B is USDC (Quote),
        // we just fetch Token A.

        let is_token_b_usd = token_b.symbol.to_uppercase().contains("USD");

        if !is_token_b_usd {
            // Warn or Error: Cross-pair fetching not fully implemented yet.
            // Proceed assuming user wants Token A / USD proxy if B is stable-ish
            tracing::warn!(
                "Cross-pair fetching (non-USD quote) not fully implemented. Returning {}/USD",
                token_a.symbol
            );
        }

        let resolution_str = self.map_resolution(resolution);
        let url = format!(
            "https://public-api.birdeye.so/defi/ohlcv?address={}&type={}&time_from={}&time_to={}",
            token_a.mint_address, resolution_str, start_time, end_time
        );

        let resp = self
            .client
            .get(&url)
            .header("X-API-KEY", &self.api_key)
            .header("accept", "application/json")
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await?;
            return Err(anyhow::anyhow!("Birdeye API error: {} - {}", status, text));
        }

        let data: BirdeyeOhlcvResponse = resp.json().await?;

        if !data.success {
            return Err(anyhow::anyhow!("Birdeye API returned success=false"));
        }

        let candles = data
            .data
            .items
            .into_iter()
            .map(|item| {
                // Convert f64 to Decimal
                let open = Decimal::from_f64(item.o).unwrap_or(Decimal::ZERO);
                let high = Decimal::from_f64(item.h).unwrap_or(Decimal::ZERO);
                let low = Decimal::from_f64(item.l).unwrap_or(Decimal::ZERO);
                let close = Decimal::from_f64(item.c).unwrap_or(Decimal::ZERO);

                // Volume from Birdeye is usually in USD or Token Amount depending on endpoint.
                // /defi/ohlcv usually returns volume in USD or Quote?
                // Docs say 'v': volume. Let's assume it's base token volume for now or handle conversion if USD.
                // Actually Birdeye OHLCV for token usually is USD price, volume in USD.
                // Let's assume 'v' is USD volume.
                // If we need volume in Token A, we approx divide by price?
                // Or we store USD volume if our domain supports it.
                // Domain `PriceCandle` has `volume_token_a: Amount`.
                // Let's estimate Token A volume = Volume USD / Close Price.

                let vol_usd = Decimal::from_f64(item.v).unwrap_or(Decimal::ZERO);
                let vol_token = if close.is_zero() {
                    Decimal::ZERO
                } else {
                    vol_usd / close
                };

                let vol_amount = Amount::from_decimal(vol_token, token_a.decimals);

                PriceCandle {
                    token_a: token_a.clone(),
                    token_b: token_b.clone(),
                    start_timestamp: item.unix_time,
                    duration_seconds: resolution,
                    open: Price::new(open),
                    high: Price::new(high),
                    low: Price::new(low),
                    close: Price::new(close),
                    volume_token_a: vol_amount,
                }
            })
            .collect();

        Ok(candles)
    }
}

/// Mock Provider for Testing
/// Mock market data provider for testing purposes.
pub struct MockMarketDataProvider;

#[async_trait]
impl MarketDataProvider for MockMarketDataProvider {
    async fn get_price_history(
        &self,
        token_a: &Token,
        token_b: &Token,
        start_time: u64,
        _end_time: u64,
        resolution: u64,
    ) -> Result<Vec<PriceCandle>> {
        Ok(vec![PriceCandle {
            token_a: token_a.clone(),
            token_b: token_b.clone(),
            start_timestamp: start_time,
            duration_seconds: resolution,
            open: Price::new(Decimal::from(100)),
            high: Price::new(Decimal::from(100)),
            low: Price::new(Decimal::from(100)),
            close: Price::new(Decimal::from(100)),
            volume_token_a: Amount::new(U256::from(0), token_a.decimals),
        }])
    }
}
