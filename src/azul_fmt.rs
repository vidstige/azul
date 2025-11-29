use crate::{
    azul::{
        MoveDescription, MoveDestination, MoveError, MoveOrigin, State, Tile, TileSet, TILES, WALL,
    },
    minmax::DeterministicGameState,
};
use std::fmt::{self, Write};

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
    let player_sections: Vec<Vec<String>> = state
        .players
        .iter()
        .enumerate()
        .map(|(index, player)| {
            let mut lines = Vec::new();
            lines.push(format!(
                "{} (points: {})",
                format_player_label(names, index),
                player.points
            ));
            lines.push("  Pattern / Wall:".to_string());
            for row_index in 0..5 {
                let pattern = format_pattern_row(row_index, &player.rows[row_index]);
                let wall_line = format_wall_row(row_index, &player.wall.rows[row_index]);
                lines.push(format!(
                    "    {:>2}: {} || {}",
                    row_index + 1,
                    pattern,
                    wall_line
                ));
            }
            lines.push(format!(
                "  Discard: {}",
                format_tileset_summary(&player.discard)
            ));
            lines
        })
        .collect();
    let column_widths: Vec<usize> = player_sections
        .iter()
        .map(|lines| lines.iter().map(|line| visible_width(line)).max().unwrap_or(0))
        .collect();
    let max_lines = player_sections
        .iter()
        .map(|lines| lines.len())
        .max()
        .unwrap_or(0);
    for line_index in 0..max_lines {
        let mut row = String::new();
        for (player_index, lines) in player_sections.iter().enumerate() {
            let content = lines.get(line_index).map(|line| line.as_str()).unwrap_or("");
            row.push_str(&pad_to_visible_width(
                content,
                column_widths[player_index],
            ));
            if player_index + 1 != player_sections.len() {
                row.push_str("    ");
            }
        }
        buffer.push_str(&row);
        buffer.push('\n');
    }
    buffer.push('\n');
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

fn visible_width(text: &str) -> usize {
    let mut width = 0;
    let mut skipping = false;
    for ch in text.chars() {
        if skipping {
            if ch == 'm' {
                skipping = false;
            }
            continue;
        }
        if ch == '\u{1b}' {
            skipping = true;
            continue;
        }
        width += 1;
    }
    width
}

fn pad_to_visible_width(text: &str, width: usize) -> String {
    let visible = visible_width(text);
    if visible >= width {
        return text.to_string();
    }
    let mut result = String::with_capacity(text.len() + width - visible);
    result.push_str(text);
    for _ in 0..(width - visible) {
        result.push(' ');
    }
    result
}

impl fmt::Display for MoveDescription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let origin = match self.origin {
            MoveOrigin::Factory(index) => format!("factory {}", index + 1),
            MoveOrigin::Center => "the center".to_string(),
        };
        let plural = if self.count == 1 { "" } else { "s" };
        match self.destination {
            MoveDestination::Row(row) => {
                write!(
                    f,
                    "Player #{} took {} {:?} tile{} from {} and placed {} on row {}",
                    self.player_index + 1,
                    self.count,
                    self.tile,
                    plural,
                    origin,
                    self.placed,
                    row + 1
                )?;
                if self.discarded > 0 {
                    write!(f, " ({} discarded)", self.discarded)?;
                }
            }
            MoveDestination::Discard => {
                let pronoun = if self.count == 1 { "it" } else { "them" };
                write!(
                    f,
                    "Player #{} took {} {:?} tile{} from {} and discarded {}",
                    self.player_index + 1,
                    self.count,
                    self.tile,
                    plural,
                    origin,
                    pronoun,
                )?;
            }
        }
        Ok(())
    }
}

impl fmt::Display for MoveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MoveError::StochasticPhase => write!(f, "no deterministic move is available"),
            MoveError::IllegalTransition => write!(f, "state transition is not legal"),
            MoveError::AmbiguousTransition => write!(f, "multiple moves lead to the same state"),
        }
    }
}
