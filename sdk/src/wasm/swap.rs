use crate::clamm::compute_swap_step;
use crate::sqrt_price::{get_max_tick, get_min_tick, SqrtPrice};
use crate::token_amount::TokenAmount;
use crate::{
    CalculateSwapResult, FeeTier, Tickmap, UpdatePoolTick, MAX_SQRT_PRICE, MAX_SWAP_STEPS,
    MIN_SQRT_PRICE,
};
use crate::{LiquidityTick, Pool};
use decimal::*;
use traceable_result::TrackableResult;
use traceable_result::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use wasm_wrapper::wasm_wrapper;

extern crate console_error_panic_hook;
use std::panic;

type LiquidityTicks = Vec<LiquidityTick>;

#[wasm_wrapper]
pub fn simulate_invariant_swap(
    tickmap: Tickmap,
    fee_tier: FeeTier,
    mut pool: Pool,
    ticks: LiquidityTicks,
    x_to_y: bool,
    amount: TokenAmount,
    by_amount_in: bool,
    sqrt_price_limit: SqrtPrice,
) -> TrackableResult<CalculateSwapResult> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    if amount.is_zero() {
        return Err(err!("Amount is zero"));
    }

    if x_to_y {
        if pool.sqrt_price <= sqrt_price_limit || sqrt_price_limit > SqrtPrice::new(MAX_SQRT_PRICE)
        {
            return Err(err!("Wrong limit"));
        }
    } else if pool.sqrt_price >= sqrt_price_limit
        || sqrt_price_limit < SqrtPrice::new(MIN_SQRT_PRICE)
    {
        return Err(err!("Wrong limit"));
    }

    let tick_limit = if x_to_y {
        get_min_tick(fee_tier.tick_spacing as u16)?
    } else {
        get_max_tick(fee_tier.tick_spacing as u16)?
    };

    let start_sqrt_price = pool.sqrt_price;

    let mut global_insufficient_liquidity = false;
    let mut state_outdated = false;
    let mut max_swap_steps_reached = false;

    let mut swap_step_number = 0;
    let mut crossed_ticks: Vec<LiquidityTick> = vec![];
    let mut remaining_amount = amount;
    let mut total_amount_in = TokenAmount(0);
    let mut total_amount_out = TokenAmount(0);
    let mut total_fee_amount = TokenAmount(0);

    while !remaining_amount.is_zero() {
        let closer_limit = tickmap.get_closer_limit(
            sqrt_price_limit,
            x_to_y,
            pool.current_tick_index as i32,
            fee_tier.tick_spacing as u16,
        );
        let (swap_limit, limiting_tick) = if let Ok(closer_limit) = closer_limit {
            closer_limit
        } else {
            global_insufficient_liquidity = true;
            break;
        };

        let result = compute_swap_step(
            pool.sqrt_price,
            swap_limit,
            pool.liquidity,
            remaining_amount,
            by_amount_in,
            fee_tier.fee,
        )?;
        swap_step_number += 1;

        // make remaining amount smaller
        if by_amount_in {
            let intermediate = result
                .amount_in
                .checked_add(result.fee_amount)
                .map_err(|_| {
                    err!(&format!(
                        "InvariantError::AddOverflow({:?}, {:?})",
                        result.amount_in.get(),
                        result.fee_amount.get()
                    ))
                })?;
            remaining_amount = remaining_amount.checked_sub(intermediate).map_err(|_| {
                err!(&format!(
                    "InvariantError::SubUnderflow({:?}, {:?})",
                    remaining_amount.get(),
                    intermediate.get()
                ))
            })?;
        } else {
            remaining_amount = remaining_amount
                .checked_sub(result.amount_out)
                .map_err(|_| {
                    err!(&format!(
                        "InvariantError::SubUnderflow({:?}, {:?})",
                        remaining_amount.get(),
                        result.amount_out.get()
                    ))
                })?;
        }

        // pool.add_fee(result.fee_amount, x_to_y, protocol_fee)?;
        total_fee_amount = total_fee_amount
            .checked_add(result.fee_amount)
            .map_err(|_| {
                err!(&format!(
                    "InvariantError::AddOverflow({:?}, {:?})",
                    total_fee_amount.get(),
                    result.fee_amount.get()
                ))
            })?;

        pool.sqrt_price = result.next_sqrt_price;

        let intermediate = total_amount_in.checked_add(result.amount_in).map_err(|_| {
            err!(&format!(
                "InvariantError::AddOverflow({:?}, {:?})",
                total_amount_in.get(),
                result.amount_in.get()
            ))
        })?;
        total_amount_in = intermediate.checked_add(result.fee_amount).map_err(|_| {
            err!(&format!(
                "InvariantError::AddOverflow({:?}, {:?})",
                intermediate.get(),
                result.fee_amount.get()
            ))
        })?;
        total_amount_out = total_amount_out
            .checked_add(result.amount_out)
            .map_err(|_| {
                err!(&format!(
                    "InvariantError::AddOverflow({:?}, {:?})",
                    total_amount_out.get(),
                    result.amount_out.get()
                ))
            })?;

        // Fail if price would go over swap limit
        if pool.sqrt_price == sqrt_price_limit && !remaining_amount.is_zero() {
            global_insufficient_liquidity = true;
            break;
        }

        let mut tick_update = {
            if let Some((tick_index, is_initialized)) = limiting_tick {
                if is_initialized {
                    let tick = ticks.iter().find(|t| t.index as i32 == tick_index);

                    match tick {
                        Some(tick) => UpdatePoolTick::TickInitialized(*tick),
                        None => {
                            state_outdated = true;
                            break;
                        }
                    }
                } else {
                    UpdatePoolTick::TickUninitialized(tick_index as i64)
                }
            } else {
                UpdatePoolTick::NoTick
            }
        };

        let tick_update_return = pool.update_tick(
            result,
            swap_limit,
            &mut tick_update,
            remaining_amount,
            by_amount_in,
            x_to_y,
            pool.last_timestamp,
            fee_tier,
        );
        let (amount_to_add, amount_after_tick_update, has_crossed) =
            if let Ok(tick_update_return) = tick_update_return {
                tick_update_return
            } else {
                state_outdated = true;
                break;
            };

        remaining_amount = amount_after_tick_update;
        total_amount_in = total_amount_in.checked_add(amount_to_add).map_err(|_| {
            err!(&format!(
                "InvariantError::AddOverflow({:?}, {:?})",
                total_amount_in.get(),
                amount_to_add.get()
            ))
        })?;

        if let UpdatePoolTick::TickInitialized(tick) = tick_update {
            if has_crossed {
                crossed_ticks.push(tick);
            }
        }

        let reached_tick_limit = match x_to_y {
            true => pool.current_tick_index <= tick_limit as i64,
            false => pool.current_tick_index >= tick_limit as i64,
        };

        if reached_tick_limit {
            global_insufficient_liquidity = true;
            break;
        }

        if swap_step_number > MAX_SWAP_STEPS {
            max_swap_steps_reached = true;
            break;
        }
    }

    Ok(CalculateSwapResult {
        amount_in: total_amount_in,
        amount_out: total_amount_out,
        start_sqrt_price,
        target_sqrt_price: pool.sqrt_price,
        fee: total_fee_amount,
        crossed_ticks,
        global_insufficient_liquidity,
        state_outdated,
        max_swap_steps_reached,
    })
}
