use crate::{
    azul::{State, Tile, TileSet, TILES, WALL},
    minmax::DeterministicGameState,
};
use std::fmt::Write;

pub fn print_state(state: &State, names: &[&str]) {
    println!("{}", render_state(state, names));
}

pub fn render_state(state: &State, names: &[&str]) -> String {
    let mut buffer = String::new();
    let player_count = state.players.len();
    if player_count == 0 {
        return "No players in state".to_string();
    }
    let current_player = state.current_player();
    writeln!(&mut buffer, "=== Azul State ===").unwrap();
    writeln!(
        &mut buffer,
        "Current player: {} (#{})",
        format_player_label(names, current_player),
        current_player + 1
    )
    .unwrap();
    writeln!(&mut buffer, "Factories:").unwrap();
    if state.factories.is_empty() {
        writeln!(&mut buffer, "  (empty)").unwrap();
    } else {
        for (index, factory) in state.factories.iter().enumerate() {
            writeln!(
                &mut buffer,
                "  F{:>2}: {}",
                index + 1,
                format_tileset_compact(factory)
            )
            .unwrap();
        }
    }
    writeln!(
        &mut buffer,
        "Center: {}",
        format_tileset_compact(&state.center)
    )
    .unwrap();
    writeln!(&mut buffer, "Tray: {}", format_tileset_summary(&state.tray)).unwrap();
    writeln!(&mut buffer, "Bag: {}", format_tileset_summary(&state.bag)).unwrap();
    buffer.push('\n');
    for (index, player) in state.players.iter().enumerate() {
        writeln!(
            &mut buffer,
            "{} (points: {})",
            format_player_label(names, index),
            player.points
        )
        .unwrap();
        writeln!(&mut buffer, "  Pattern / Wall:").unwrap();
        for row_index in 0..5 {
            let pattern = format_pattern_row(row_index, &player.rows[row_index]);
            let wall_line = format_wall_row(row_index, &player.wall.rows[row_index]);
            writeln!(
                &mut buffer,
                "    {:>2}: {} || {}",
                row_index + 1,
                pattern,
                wall_line
            )
            .unwrap();
        }
        writeln!(
            &mut buffer,
            "  Discard: {}",
            format_tileset_summary(&player.discard)
        )
        .unwrap();
        buffer.push('\n');
    }
    buffer
}

fn format_player_label(names: &[&str], index: usize) -> String {
    names
        .get(index)
        .map(|name| name.to_string())
        .unwrap_or_else(|| format!("Player {}", index + 1))
}

fn tile_letter(tile: Tile) -> char {
    match tile {
        Tile::BLACK => 'B',
        Tile::WHITE => 'W',
        Tile::AZUL => 'A',
        Tile::YELLOW => 'Y',
        Tile::RED => 'R',
    }
}

fn tile_color_code(tile: Tile) -> &'static str {
    match tile {
        Tile::BLACK => "30",
        Tile::WHITE => "37",
        Tile::AZUL => "36",
        Tile::YELLOW => "33",
        Tile::RED => "31",
    }
}

fn colored_tile(tile: Tile) -> String {
    format!(
        "\x1b[1;{}m{}\x1b[0m",
        tile_color_code(tile),
        tile_letter(tile)
    )
}

fn format_tileset_compact(tileset: &TileSet) -> String {
    if tileset.len() == 0 {
        return "-".to_string();
    }
    if tileset.len() <= 12 {
        let mut tiles = Vec::new();
        for tile in TILES {
            for _ in 0..tileset[tile] {
                tiles.push(colored_tile(tile));
            }
        }
        if tiles.is_empty() {
            "-".to_string()
        } else {
            tiles.join(" ")
        }
    } else {
        format_tileset_summary(tileset)
    }
}

fn format_tileset_summary(tileset: &TileSet) -> String {
    let mut parts = Vec::new();
    for tile in TILES {
        let count = tileset[tile];
        if count > 0 {
            parts.push(format!("{}x{}", colored_tile(tile), count));
        }
    }
    if parts.is_empty() {
        "-".to_string()
    } else {
        parts.join(" ")
    }
}

fn format_pattern_row(row_index: usize, row: &Option<(Tile, usize)>) -> String {
    let size = row_index + 1;
    let start = 5 - size;
    let mut cells = vec![".".to_string(); 5];
    if let Some((tile, count)) = row {
        let filled = (*count).min(size);
        for i in 0..filled {
            cells[start + i] = colored_tile(*tile);
        }
        for i in filled..size {
            cells[start + i] = "_".to_string();
        }
    } else {
        for i in 0..size {
            cells[start + i] = "_".to_string();
        }
    }
    cells.join(" ")
}

fn format_wall_row(row_index: usize, wall_row: &[bool; 5]) -> String {
    let mut cells = Vec::new();
    for (column_index, occupied) in wall_row.iter().enumerate() {
        if *occupied {
            cells.push(colored_tile(WALL[row_index][column_index]));
        } else {
            cells.push(".".to_string());
        }
    }
    cells.join(" ")
}
