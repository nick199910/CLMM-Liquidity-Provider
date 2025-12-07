//! Position handlers.

use crate::error::{ApiError, ApiResult};
use crate::models::{
    ListPositionsResponse, MessageResponse, OpenPositionRequest, PnLResponse, PositionResponse,
    PositionStatus, RebalanceRequest,
};
use crate::state::AppState;
use axum::{
    Json,
    extract::{Path, State},
};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use tracing::info;

/// List all positions.
#[utoipa::path(
    get,
    path = "/positions",
    tag = "Positions",
    responses(
        (status = 200, description = "List of positions", body = ListPositionsResponse)
    )
)]
pub async fn list_positions(
    State(state): State<AppState>,
) -> ApiResult<Json<ListPositionsResponse>> {
    let positions = state.monitor.get_positions().await;

    let responses: Vec<PositionResponse> = positions
        .iter()
        .map(|p| PositionResponse {
            address: p.address.to_string(),
            pool_address: p.pool.to_string(),
            owner: p.on_chain.owner.to_string(),
            tick_lower: p.on_chain.tick_lower,
            tick_upper: p.on_chain.tick_upper,
            liquidity: p.on_chain.liquidity.to_string(),
            in_range: p.in_range,
            value_usd: p.pnl.current_value_usd,
            pnl: PnLResponse {
                unrealized_pnl_usd: p.pnl.net_pnl_usd,
                unrealized_pnl_pct: p.pnl.net_pnl_pct,
                fees_earned_a: p.pnl.fees_earned_a,
                fees_earned_b: p.pnl.fees_earned_b,
                fees_earned_usd: p.pnl.fees_usd,
                il_pct: p.pnl.il_pct,
                net_pnl_usd: p.pnl.net_pnl_usd,
                net_pnl_pct: p.pnl.net_pnl_pct,
            },
            status: if p.in_range {
                PositionStatus::Active
            } else {
                PositionStatus::OutOfRange
            },
            created_at: None,
        })
        .collect();

    Ok(Json(ListPositionsResponse {
        total: responses.len(),
        positions: responses,
    }))
}

/// Get a specific position.
#[utoipa::path(
    get,
    path = "/positions/{address}",
    tag = "Positions",
    params(
        ("address" = String, Path, description = "Position address")
    ),
    responses(
        (status = 200, description = "Position details", body = PositionResponse),
        (status = 404, description = "Position not found")
    )
)]
pub async fn get_position(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> ApiResult<Json<PositionResponse>> {
    let pubkey = Pubkey::from_str(&address)
        .map_err(|_| ApiError::bad_request("Invalid position address"))?;

    let positions = state.monitor.get_positions().await;
    let position = positions
        .iter()
        .find(|p| p.address == pubkey)
        .ok_or_else(|| ApiError::not_found("Position not found"))?;

    let response = PositionResponse {
        address: position.address.to_string(),
        pool_address: position.pool.to_string(),
        owner: position.on_chain.owner.to_string(),
        tick_lower: position.on_chain.tick_lower,
        tick_upper: position.on_chain.tick_upper,
        liquidity: position.on_chain.liquidity.to_string(),
        in_range: position.in_range,
        value_usd: position.pnl.current_value_usd,
        pnl: PnLResponse {
            unrealized_pnl_usd: position.pnl.net_pnl_usd,
            unrealized_pnl_pct: position.pnl.net_pnl_pct,
            fees_earned_a: position.pnl.fees_earned_a,
            fees_earned_b: position.pnl.fees_earned_b,
            fees_earned_usd: position.pnl.fees_usd,
            il_pct: position.pnl.il_pct,
            net_pnl_usd: position.pnl.net_pnl_usd,
            net_pnl_pct: position.pnl.net_pnl_pct,
        },
        status: if position.in_range {
            PositionStatus::Active
        } else {
            PositionStatus::OutOfRange
        },
        created_at: None,
    };

    Ok(Json(response))
}

/// Open a new position.
#[utoipa::path(
    post,
    path = "/positions",
    tag = "Positions",
    request_body = OpenPositionRequest,
    responses(
        (status = 201, description = "Position opened", body = PositionResponse),
        (status = 400, description = "Invalid request")
    )
)]
pub async fn open_position(
    State(_state): State<AppState>,
    Json(request): Json<OpenPositionRequest>,
) -> ApiResult<Json<MessageResponse>> {
    info!(
        pool = %request.pool_address,
        tick_lower = request.tick_lower,
        tick_upper = request.tick_upper,
        "Opening position"
    );

    // TODO: Implement actual position opening
    // This would use the WhirlpoolExecutor to open a position

    Ok(Json(MessageResponse::new(
        "Position opening not yet implemented",
    )))
}

/// Close a position.
#[utoipa::path(
    delete,
    path = "/positions/{address}",
    tag = "Positions",
    params(
        ("address" = String, Path, description = "Position address")
    ),
    responses(
        (status = 200, description = "Position closed", body = MessageResponse),
        (status = 404, description = "Position not found")
    )
)]
pub async fn close_position(
    State(_state): State<AppState>,
    Path(address): Path<String>,
) -> ApiResult<Json<MessageResponse>> {
    let _pubkey = Pubkey::from_str(&address)
        .map_err(|_| ApiError::bad_request("Invalid position address"))?;

    info!(position = %address, "Closing position");

    // TODO: Implement actual position closing

    Ok(Json(MessageResponse::new(
        "Position closing not yet implemented",
    )))
}

/// Collect fees from a position.
#[utoipa::path(
    post,
    path = "/positions/{address}/collect",
    tag = "Positions",
    params(
        ("address" = String, Path, description = "Position address")
    ),
    responses(
        (status = 200, description = "Fees collected", body = MessageResponse),
        (status = 404, description = "Position not found")
    )
)]
pub async fn collect_fees(
    State(_state): State<AppState>,
    Path(address): Path<String>,
) -> ApiResult<Json<MessageResponse>> {
    let _pubkey = Pubkey::from_str(&address)
        .map_err(|_| ApiError::bad_request("Invalid position address"))?;

    info!(position = %address, "Collecting fees");

    // TODO: Implement actual fee collection

    Ok(Json(MessageResponse::new(
        "Fee collection not yet implemented",
    )))
}

/// Rebalance a position.
#[utoipa::path(
    post,
    path = "/positions/{address}/rebalance",
    tag = "Positions",
    params(
        ("address" = String, Path, description = "Position address")
    ),
    request_body = RebalanceRequest,
    responses(
        (status = 200, description = "Position rebalanced", body = MessageResponse),
        (status = 404, description = "Position not found")
    )
)]
pub async fn rebalance_position(
    State(_state): State<AppState>,
    Path(address): Path<String>,
    Json(request): Json<RebalanceRequest>,
) -> ApiResult<Json<MessageResponse>> {
    let _pubkey = Pubkey::from_str(&address)
        .map_err(|_| ApiError::bad_request("Invalid position address"))?;

    info!(
        position = %address,
        new_tick_lower = request.new_tick_lower,
        new_tick_upper = request.new_tick_upper,
        "Rebalancing position"
    );

    // TODO: Implement actual rebalancing

    Ok(Json(MessageResponse::new(
        "Rebalancing not yet implemented",
    )))
}

/// Get position PnL details.
#[utoipa::path(
    get,
    path = "/positions/{address}/pnl",
    tag = "Positions",
    params(
        ("address" = String, Path, description = "Position address")
    ),
    responses(
        (status = 200, description = "Position PnL", body = PnLResponse),
        (status = 404, description = "Position not found")
    )
)]
pub async fn get_position_pnl(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> ApiResult<Json<PnLResponse>> {
    let pubkey = Pubkey::from_str(&address)
        .map_err(|_| ApiError::bad_request("Invalid position address"))?;

    let positions = state.monitor.get_positions().await;
    let position = positions
        .iter()
        .find(|p| p.address == pubkey)
        .ok_or_else(|| ApiError::not_found("Position not found"))?;

    let response = PnLResponse {
        unrealized_pnl_usd: position.pnl.net_pnl_usd,
        unrealized_pnl_pct: position.pnl.net_pnl_pct,
        fees_earned_a: position.pnl.fees_earned_a,
        fees_earned_b: position.pnl.fees_earned_b,
        fees_earned_usd: position.pnl.fees_usd,
        il_pct: position.pnl.il_pct,
        net_pnl_usd: position.pnl.net_pnl_usd,
        net_pnl_pct: position.pnl.net_pnl_pct,
    };

    Ok(Json(response))
}
