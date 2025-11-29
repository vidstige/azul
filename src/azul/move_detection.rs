use super::{MoveDescription, MoveDestination, MoveError, MoveOrigin, Player, State, Tile, TILES};
use crate::minmax::DeterministicGameState;

fn row_count(row: Option<(Tile, usize)>) -> usize {
    row.map(|(_, count)| count).unwrap_or(0)
}

fn row_space_left(row_index: usize, row: Option<(Tile, usize)>) -> usize {
    let row_size = row_index + 1;
    row_size.saturating_sub(row_count(row))
}

fn row_is_legal(player: &Player, row_index: usize, tile: Tile) -> bool {
    if player.wall.has_tile(row_index, &tile) {
        return false;
    }
    match player.rows[row_index] {
        None => true,
        Some((current_tile, _)) => current_tile == tile,
    }
}

fn collect_legal_rows(player: &Player, tile: Tile) -> Vec<usize> {
    (0..5)
        .filter(|&row_index| row_is_legal(player, row_index, tile))
        .collect()
}

fn detect_target_row(
    before_player: &Player,
    after_player: &Player,
    tile: Tile,
    scoring: bool,
) -> Result<Option<usize>, MoveError> {
    let mut candidate = None;
    for row_index in 0..5 {
        if !row_is_legal(before_player, row_index, tile) {
            continue;
        }
        let before_row = before_player.rows[row_index];
        let after_row = after_player.rows[row_index];
        let changed = if scoring {
            let row_size = row_index + 1;
            let before_count = row_count(before_row);
            if before_count == row_size {
                continue;
            }
            let after_count = row_count(after_row);
            after_row.is_none() || after_count > before_count
        } else {
            before_row != after_row
        };
        if changed {
            if candidate.is_some() {
                return Err(MoveError::AmbiguousTransition);
            }
            candidate = Some(row_index);
        }
    }
    Ok(candidate)
}

fn detect_origin(before: &State, after: &State) -> Result<(MoveOrigin, Tile, usize), MoveError> {
    if before.factories.len() != after.factories.len() {
        return Err(MoveError::IllegalTransition);
    }
    let mut changed_factory = None;
    for index in 0..before.factories.len() {
        if before.factories[index] != after.factories[index] {
            if changed_factory.is_some() {
                return Err(MoveError::AmbiguousTransition);
            }
            changed_factory = Some(index);
        }
    }
    if let Some(factory_index) = changed_factory {
        let before_factory = &before.factories[factory_index];
        let after_factory = &after.factories[factory_index];
        if after_factory.len() != 0 {
            return Err(MoveError::IllegalTransition);
        }
        let mut chosen_tile = None;
        for tile in TILES {
            let factory_count = before_factory[tile];
            if factory_count == 0 {
                continue;
            }
            let center_diff = after.center[tile] as isize - before.center[tile] as isize;
            if center_diff == 0 {
                if chosen_tile.is_some() {
                    return Err(MoveError::AmbiguousTransition);
                }
                chosen_tile = Some((tile, factory_count));
            } else if center_diff as usize != factory_count {
                return Err(MoveError::IllegalTransition);
            }
        }
        if let Some((tile, count)) = chosen_tile {
            return Ok((MoveOrigin::Factory(factory_index), tile, count));
        } else {
            return Err(MoveError::IllegalTransition);
        }
    }
    let mut chosen_tile = None;
    for tile in TILES {
        let before_count = before.center[tile];
        let after_count = after.center[tile];
        if before_count != after_count {
            let diff = before_count as isize - after_count as isize;
            if diff <= 0 {
                return Err(MoveError::IllegalTransition);
            }
            if chosen_tile.is_some() {
                return Err(MoveError::AmbiguousTransition);
            }
            chosen_tile = Some((tile, diff as usize));
        }
    }
    chosen_tile
        .map(|(tile, count)| (MoveOrigin::Center, tile, count))
        .ok_or(MoveError::IllegalTransition)
}

fn determine_destination(
    before: &State,
    after: &State,
    player_index: usize,
    tile: Tile,
    count: usize,
) -> Result<(MoveDestination, usize, usize), MoveError> {
    let player_before = &before.players[player_index];
    let player_after = &after.players[player_index];
    let scoring = after.is_empty();
    let legal_rows = collect_legal_rows(player_before, tile);
    let mut target_row = detect_target_row(player_before, player_after, tile, scoring)?;
    if target_row.is_none() && legal_rows.len() == 1 {
        target_row = Some(legal_rows[0]);
    }
    if let Some(row_index) = target_row {
        if !row_is_legal(player_before, row_index, tile) {
            return Err(MoveError::IllegalTransition);
        }
        let before_row = player_before.rows[row_index];
        let before_count = row_count(before_row);
        let space_left = row_space_left(row_index, before_row);
        let placed = count.min(space_left);
        let discarded = count - placed;
        if placed == 0 && space_left == 0 && legal_rows.len() > 1 {
            return Err(MoveError::AmbiguousTransition);
        }
        if placed == space_left && space_left > 0 {
            if scoring {
                if player_after.rows[row_index].is_some() {
                    return Err(MoveError::IllegalTransition);
                }
            } else {
                let after_count = row_count(player_after.rows[row_index]);
                if after_count != before_count + placed {
                    return Err(MoveError::IllegalTransition);
                }
            }
        } else {
            let after_count = row_count(player_after.rows[row_index]);
            if after_count != before_count + placed {
                return Err(MoveError::IllegalTransition);
            }
        }
        if !scoring {
            let discard_diff =
                player_after.discard[tile] as isize - player_before.discard[tile] as isize;
            if discard_diff != discarded as isize {
                return Err(MoveError::IllegalTransition);
            }
        }
        return Ok((MoveDestination::Row(row_index), placed, discarded));
    }
    if !legal_rows.is_empty() {
        return Err(MoveError::AmbiguousTransition);
    }
    if !scoring {
        let discard_diff =
            player_after.discard[tile] as isize - player_before.discard[tile] as isize;
        if discard_diff != count as isize {
            return Err(MoveError::IllegalTransition);
        }
    }
    Ok((MoveDestination::Discard, 0, count))
}

pub fn describe_move(before: &State, after: &State) -> Result<MoveDescription, MoveError> {
    if before.is_empty() {
        return Err(MoveError::StochasticPhase);
    }
    if before.players.len() != after.players.len() {
        return Err(MoveError::IllegalTransition);
    }
    if after.moves != before.moves + 1 {
        return Err(MoveError::IllegalTransition);
    }
    let player_index = before.current_player();
    let (origin, tile, count) = detect_origin(before, after)?;
    let (destination, placed, discarded) =
        determine_destination(before, after, player_index, tile, count)?;
    Ok(MoveDescription {
        player_index,
        origin,
        tile,
        count,
        destination,
        placed,
        discarded,
    })
}
