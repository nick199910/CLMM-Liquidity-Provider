//! Strategy handlers.

use crate::error::{ApiError, ApiResult};
use crate::models::{
    CreateStrategyRequest, ListStrategiesResponse, MessageResponse, StrategyParameters,
    StrategyPerformanceResponse, StrategyResponse, StrategyType,
};
use crate::state::{AppState, StrategyState};
use axum::{
    Json,
    extract::{Path, State},
};
use rust_decimal::Decimal;
use tracing::info;

/// List all strategies.
#[utoipa::path(
    get,
    path = "/strategies",
    tag = "Strategies",
    responses(
        (status = 200, description = "List of strategies", body = ListStrategiesResponse)
    )
)]
pub async fn list_strategies(
    State(state): State<AppState>,
) -> ApiResult<Json<ListStrategiesResponse>> {
    let strategies = state.strategies.read().await;

    let responses: Vec<StrategyResponse> = strategies
        .values()
        .map(|s| {
            let params: StrategyParameters =
                serde_json::from_value(s.config.clone()).unwrap_or(StrategyParameters {
                    tick_width: None,
                    rebalance_threshold_pct: None,
                    max_il_pct: None,
                    eval_interval_secs: None,
                    min_rebalance_interval_hours: None,
                });

            StrategyResponse {
                id: s.id.clone(),
                name: s.name.clone(),
                pool_address: s
                    .config
                    .get("pool_address")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                strategy_type: s
                    .config
                    .get("strategy_type")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or(StrategyType::StaticRange),
                parameters: params,
                running: s.running,
                dry_run: s
                    .config
                    .get("dry_run")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                created_at: s.created_at,
                updated_at: s.updated_at,
            }
        })
        .collect();

    Ok(Json(ListStrategiesResponse {
        total: responses.len(),
        strategies: responses,
    }))
}

/// Get a specific strategy.
#[utoipa::path(
    get,
    path = "/strategies/{id}",
    tag = "Strategies",
    params(
        ("id" = String, Path, description = "Strategy ID")
    ),
    responses(
        (status = 200, description = "Strategy details", body = StrategyResponse),
        (status = 404, description = "Strategy not found")
    )
)]
pub async fn get_strategy(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<StrategyResponse>> {
    let strategies = state.strategies.read().await;
    let strategy = strategies
        .get(&id)
        .ok_or_else(|| ApiError::not_found("Strategy not found"))?;

    let params: StrategyParameters =
        serde_json::from_value(strategy.config.clone()).unwrap_or(StrategyParameters {
            tick_width: None,
            rebalance_threshold_pct: None,
            max_il_pct: None,
            eval_interval_secs: None,
            min_rebalance_interval_hours: None,
        });

    let response = StrategyResponse {
        id: strategy.id.clone(),
        name: strategy.name.clone(),
        pool_address: strategy
            .config
            .get("pool_address")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        strategy_type: strategy
            .config
            .get("strategy_type")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or(StrategyType::StaticRange),
        parameters: params,
        running: strategy.running,
        dry_run: strategy
            .config
            .get("dry_run")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        created_at: strategy.created_at,
        updated_at: strategy.updated_at,
    };

    Ok(Json(response))
}

/// Create a new strategy.
#[utoipa::path(
    post,
    path = "/strategies",
    tag = "Strategies",
    request_body = CreateStrategyRequest,
    responses(
        (status = 201, description = "Strategy created", body = StrategyResponse),
        (status = 400, description = "Invalid request")
    )
)]
pub async fn create_strategy(
    State(state): State<AppState>,
    Json(request): Json<CreateStrategyRequest>,
) -> ApiResult<Json<StrategyResponse>> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now();

    let config = serde_json::json!({
        "pool_address": request.pool_address,
        "strategy_type": request.strategy_type,
        "parameters": request.parameters,
        "auto_execute": request.auto_execute,
        "dry_run": request.dry_run,
    });

    let strategy_state = StrategyState {
        id: id.clone(),
        name: request.name.clone(),
        running: false,
        config: config.clone(),
        created_at: now,
        updated_at: now,
    };

    state
        .strategies
        .write()
        .await
        .insert(id.clone(), strategy_state);

    info!(id = %id, name = %request.name, "Strategy created");

    let response = StrategyResponse {
        id,
        name: request.name,
        pool_address: request.pool_address,
        strategy_type: request.strategy_type,
        parameters: request.parameters,
        running: false,
        dry_run: request.dry_run,
        created_at: now,
        updated_at: now,
    };

    Ok(Json(response))
}

/// Update a strategy.
#[utoipa::path(
    put,
    path = "/strategies/{id}",
    tag = "Strategies",
    params(
        ("id" = String, Path, description = "Strategy ID")
    ),
    request_body = CreateStrategyRequest,
    responses(
        (status = 200, description = "Strategy updated", body = StrategyResponse),
        (status = 404, description = "Strategy not found")
    )
)]
pub async fn update_strategy(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<CreateStrategyRequest>,
) -> ApiResult<Json<StrategyResponse>> {
    let mut strategies = state.strategies.write().await;
    let strategy = strategies
        .get_mut(&id)
        .ok_or_else(|| ApiError::not_found("Strategy not found"))?;

    let now = chrono::Utc::now();

    let config = serde_json::json!({
        "pool_address": request.pool_address,
        "strategy_type": request.strategy_type,
        "parameters": request.parameters,
        "auto_execute": request.auto_execute,
        "dry_run": request.dry_run,
    });

    strategy.name = request.name.clone();
    strategy.config = config;
    strategy.updated_at = now;

    info!(id = %id, "Strategy updated");

    let response = StrategyResponse {
        id,
        name: request.name,
        pool_address: request.pool_address,
        strategy_type: request.strategy_type,
        parameters: request.parameters,
        running: strategy.running,
        dry_run: request.dry_run,
        created_at: strategy.created_at,
        updated_at: now,
    };

    Ok(Json(response))
}

/// Delete a strategy.
#[utoipa::path(
    delete,
    path = "/strategies/{id}",
    tag = "Strategies",
    params(
        ("id" = String, Path, description = "Strategy ID")
    ),
    responses(
        (status = 200, description = "Strategy deleted", body = MessageResponse),
        (status = 404, description = "Strategy not found")
    )
)]
pub async fn delete_strategy(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<MessageResponse>> {
    let mut strategies = state.strategies.write().await;

    if strategies.remove(&id).is_none() {
        return Err(ApiError::not_found("Strategy not found"));
    }

    info!(id = %id, "Strategy deleted");

    Ok(Json(MessageResponse::new("Strategy deleted")))
}

/// Start a strategy.
#[utoipa::path(
    post,
    path = "/strategies/{id}/start",
    tag = "Strategies",
    params(
        ("id" = String, Path, description = "Strategy ID")
    ),
    responses(
        (status = 200, description = "Strategy started", body = MessageResponse),
        (status = 404, description = "Strategy not found")
    )
)]
pub async fn start_strategy(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<MessageResponse>> {
    let mut strategies = state.strategies.write().await;
    let strategy = strategies
        .get_mut(&id)
        .ok_or_else(|| ApiError::not_found("Strategy not found"))?;

    if strategy.running {
        return Err(ApiError::Conflict(
            "Strategy is already running".to_string(),
        ));
    }

    strategy.running = true;
    strategy.updated_at = chrono::Utc::now();

    info!(id = %id, "Strategy started");

    // TODO: Actually start the strategy executor

    Ok(Json(MessageResponse::new("Strategy started")))
}

/// Stop a strategy.
#[utoipa::path(
    post,
    path = "/strategies/{id}/stop",
    tag = "Strategies",
    params(
        ("id" = String, Path, description = "Strategy ID")
    ),
    responses(
        (status = 200, description = "Strategy stopped", body = MessageResponse),
        (status = 404, description = "Strategy not found")
    )
)]
pub async fn stop_strategy(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<MessageResponse>> {
    let mut strategies = state.strategies.write().await;
    let strategy = strategies
        .get_mut(&id)
        .ok_or_else(|| ApiError::not_found("Strategy not found"))?;

    if !strategy.running {
        return Err(ApiError::Conflict("Strategy is not running".to_string()));
    }

    strategy.running = false;
    strategy.updated_at = chrono::Utc::now();

    info!(id = %id, "Strategy stopped");

    // TODO: Actually stop the strategy executor

    Ok(Json(MessageResponse::new("Strategy stopped")))
}

/// Get strategy performance.
#[utoipa::path(
    get,
    path = "/strategies/{id}/performance",
    tag = "Strategies",
    params(
        ("id" = String, Path, description = "Strategy ID")
    ),
    responses(
        (status = 200, description = "Strategy performance", body = StrategyPerformanceResponse),
        (status = 404, description = "Strategy not found")
    )
)]
pub async fn get_strategy_performance(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<StrategyPerformanceResponse>> {
    let strategies = state.strategies.read().await;
    if !strategies.contains_key(&id) {
        return Err(ApiError::not_found("Strategy not found"));
    }

    // Get aggregate stats from lifecycle tracker
    let stats = state.lifecycle.get_aggregate_stats().await;

    let response = StrategyPerformanceResponse {
        strategy_id: id,
        total_pnl_usd: stats.total_pnl_usd,
        total_pnl_pct: stats.avg_pnl_pct,
        total_fees_usd: stats.total_fees_usd,
        total_il_pct: Decimal::ZERO, // Would need to track per strategy
        rebalance_count: stats.total_rebalances,
        total_tx_costs_lamports: stats.total_tx_costs_lamports,
        win_rate_pct: Decimal::ZERO, // Would need to track per strategy
    };

    Ok(Json(response))
}
