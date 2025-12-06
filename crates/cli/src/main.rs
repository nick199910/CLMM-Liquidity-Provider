use amm_data::{MarketDataProvider, providers::BirdeyeProvider};
use amm_domain::entities::token::Token;
use anyhow::Result;
use clap::{Parser, Subcommand};
use dotenv::dotenv;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Parser)]
#[command(name = "amm-cli")]
#[command(about = "CLMM Liquidity Provider Strategy Optimizer CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Fetch recent market data
    MarketData {
        /// Token A Symbol (e.g., SOL)
        #[arg(short, long, default_value = "SOL")]
        symbol_a: String,

        /// Token A Mint Address
        #[arg(long, default_value = "So11111111111111111111111111111111111111112")]
        mint_a: String,

        /// Hours of history to fetch
        #[arg(short, long, default_value_t = 24)]
        hours: u64,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::MarketData {
            symbol_a,
            mint_a,
            hours,
        } => {
            let api_key = env::var("BIRDEYE_API_KEY")
                .expect("BIRDEYE_API_KEY must be set in .env or environment");

            println!("📡 Initializing Birdeye Provider...");
            let provider = BirdeyeProvider::new(api_key);

            // Define Tokens (Token B assumed USDC for this demo)
            let token_a = Token::new(mint_a, symbol_a, 9, symbol_a);
            let token_b = Token::new(
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                "USDC",
                6,
                "USD Coin",
            );

            let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
            let start_time = now - (hours * 3600);

            println!(
                "🔍 Fetching data for {}/USDC from {} to {}...",
                symbol_a, start_time, now
            );

            // Fetch 1-hour candles
            let candles = provider
                .get_price_history(
                    &token_a, &token_b, start_time, now, 3600, // 1h resolution
                )
                .await?;

            println!("✅ Fetched {} candles:", candles.len());
            println!(
                "{:<20} | {:<10} | {:<10} | {:<10} | {:<10}",
                "Time", "Open", "High", "Low", "Close"
            );
            println!("{}", "-".repeat(70));

            for candle in candles {
                let datetime = chrono::DateTime::from_timestamp(candle.start_timestamp as i64, 0)
                    .unwrap_or_default();
                println!(
                    "{:<20} | {:<10.4} | {:<10.4} | {:<10.4} | {:<10.4}",
                    datetime.format("%Y-%m-%d %H:%M"),
                    candle.open.value,
                    candle.high.value,
                    candle.low.value,
                    candle.close.value
                );
            }
        }
    }

    Ok(())
}
