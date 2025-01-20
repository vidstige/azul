use std::{iter, ops::{Index, IndexMut}};

use rand::{distributions::WeightedIndex, prelude::Distribution, seq::SliceRandom, thread_rng, Rng};


#[derive(Clone, Copy, Debug, PartialEq)]
enum Tile {
    BLACK,
    WHITE,
    AZUL,
    YELLOW,
    RED,
}
const TILES: [Tile; 5] = [Tile::BLACK, Tile::WHITE, Tile::AZUL, Tile::YELLOW, Tile::RED];

impl TryFrom<usize> for Tile {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Tile::BLACK),
            1 => Ok(Tile::WHITE),
            2 => Ok(Tile::AZUL),
            3 => Ok(Tile::YELLOW),
            4 => Ok(Tile::RED),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug)]
struct TileSet {
    black: usize,
    white: usize,
    azul: usize,
    yellow: usize,
    red: usize,
}

impl Index<Tile> for TileSet {
    type Output = usize;

    fn index(&self, tile: Tile) -> &Self::Output {
        match tile {
            Tile::BLACK => &self.black,
            Tile::WHITE => &self.white,
            Tile::AZUL => &self.azul,
            Tile::YELLOW => &self.yellow,
            Tile::RED => &self.red,
        }
    }
}

impl IndexMut<Tile> for TileSet {
    fn index_mut(&mut self, tile: Tile) -> &mut Self::Output {
        match tile {
            Tile::BLACK => &mut self.black,
            Tile::WHITE => &mut self.white,
            Tile::AZUL => &mut self.azul,
            Tile::YELLOW => &mut self.yellow,
            Tile::RED => &mut self.red,
        }
    }
}

impl FromIterator<Tile> for TileSet {
    fn from_iter<T: IntoIterator<Item = Tile>>(iter: T) -> Self {
        let mut tileset = TileSet::new();
        for item in iter {
            tileset.push(item);
        }
        tileset
    }
}

impl TileSet {
    fn new() -> Self {
        Self { black: 0, white: 0, azul: 0, yellow: 0, red: 0 }
    }
    fn drain(&mut self, tile: Tile) -> usize {
        let count = self[tile];
        self[tile] = 0;
        count
    }
    
    fn draw_one<R: Rng>(&mut self, rng: &mut R) -> Tile {
        let weights = [self.black, self.white, self.azul, self.yellow, self.red];
        let distribution = WeightedIndex::new(&weights).unwrap();
        let tile: Tile = distribution.sample(rng).try_into().unwrap();
        self[tile] = self[tile].saturating_sub(1);
        tile
    }
    fn draw<R: Rng>(&mut self, rng: &mut R, count: usize) -> TileSet {
        let mut tileset = TileSet::new();
        for _ in 0..count {
            let tile = self.draw_one(rng);
            tileset.push(tile);
        }
        tileset
    }

    fn push(&mut self, tile: Tile) {
        self[tile] += 1;
    }
    
    fn extend(&mut self, tileset: TileSet) {
        self.black += tileset.black;
        self.white += tileset.white;
        self.azul += tileset.azul;
        self.yellow += tileset.yellow;
        self.red += tileset.red;
    }
    
    fn len(&self) -> usize {
        self.black + self.white + self.azul + self.yellow + self.red
    }
}

#[derive(Clone)]
struct Wall {
    rows: [[bool; 5]; 5],
}
const WALL: [[Tile; 5]; 5] = [
    [Tile::AZUL, Tile::YELLOW, Tile::RED, Tile::BLACK, Tile::WHITE],
    [Tile::WHITE, Tile::AZUL, Tile::YELLOW, Tile::RED, Tile::BLACK],
    [Tile::BLACK, Tile::WHITE, Tile::AZUL, Tile::YELLOW, Tile::RED],
    [Tile::RED, Tile::BLACK, Tile::WHITE, Tile::AZUL, Tile::YELLOW],
    [Tile::YELLOW, Tile::RED, Tile::BLACK, Tile::WHITE, Tile::AZUL],
];
impl Wall {
    fn new() -> Self {
        Wall { rows: Default::default() }
    }
    fn len(&self) -> usize {
        self.rows.as_flattened().iter().filter(|p| **p).count()
    }
    fn has_tile(&self, row_index: usize, tile: &Tile) -> bool {
        let colum_index = WALL[row_index].iter().position(|cell| cell == tile).unwrap();
        self.rows[row_index][colum_index]
    }    
}

#[derive(Clone)]
struct Player {
    rows: [Option<(Tile, usize)>; 5],
    points: usize,
    wall: Wall,
    discard: TileSet,
}
impl Player {
    fn new() -> Self {
        Self {
            rows: Default::default(),
            points: 0,
            wall: Wall::new(),
            discard: TileSet::new(),
        }
    }

    fn maybe_place(&mut self, tile: Tile, count: usize, row_index: usize) -> bool {
        let row_size = row_index + 1;
        if let Some((current_tile, ref mut current_count)) = self.rows[row_index] {
            if current_tile != tile || self.wall.has_tile(row_index, &tile) {
                // another tile is used - we can't place here at all
                return false
            }
            // some spaces used. return how many are left
            let space_left = row_size - *current_count;
            // we got space left - add what we can, discard rest
            let discard_count = count.saturating_sub(space_left);
            *current_count += count - discard_count;
            self.discard[tile] += discard_count;
        } else {
            // unoccupied row - we can use the entire row
            let space_left = row_size;
            // we got space left - add what we can, discard rest
            let discard_count = count.saturating_sub(space_left);
            self.rows[row_index] = Some((tile, count - discard_count));
            self.discard[tile] += discard_count;
        }
        true
    }
    
    // TODO: rename this function
    fn finish_up(&mut self, tray: &mut TileSet) {
        for (row_index, row) in self.rows.iter_mut().enumerate() {
            /*let row_size = row_index + 1;
            if let Some((tile, count)) = row.clone() {
                if count == row_size {
                    //self.wall[] // add tile to wall
                    tray[tile] += count - 1; // add rest back to tray
                    *row = None;  // clear row
                }
            }*/
        }
    }

    fn tile_count(&self) -> usize {
        [
            self.rows.iter().flat_map(|r| r).map(|(_, count)| count).sum(),
            self.wall.len(),
            self.discard.len(),
        ].iter().sum()
    }
}

#[derive(Clone)]
struct State {
    bag: TileSet,
    factories: Vec<TileSet>,
    center: TileSet,
    tray: TileSet,
    players: Vec<Player>,
    moves: usize,
}

impl State {
    fn new(players: usize) -> State {
        let bag = [
            iter::repeat(Tile::BLACK).take(20),
            iter::repeat(Tile::WHITE).take(20),
            iter::repeat(Tile::AZUL).take(20),
            iter::repeat(Tile::YELLOW).take(20),
            iter::repeat(Tile::RED).take(20),
        ].into_iter().flat_map(|it| it).collect();
        let players = iter::repeat(Player::new()).take(players).collect();
        Self {
            bag,
            factories: Vec::new(),
            center: TileSet::new(),
            tray: TileSet::new(),
            players,
            moves: 0,
        }
    }
    fn tile_count(&self) -> usize {
        [
            self.bag.len(),
            self.factories.iter().map(|f| f.len()).sum(),
            self.center.len(),
            self.tray.len(),
            self.players.iter().map(|p| p.tile_count()).sum(),
        ].iter().sum()
    }
    fn deal<R: Rng>(&mut self, rng: &mut R) {
        // deal factories
        for _ in 0..5 {
            // TODO: What if the bag is empty?
            let tiles = self.bag.draw(rng, 4);
            self.factories.push(tiles);
        }
    }
    fn is_empty(&self) -> bool {
        self.factories.iter().map(|factory| factory.len()).sum::<usize>() + self.center.len() == 0
    }
    // clean up by updating score, dealing new tiles, etc
    fn finish_up(&mut self) {
        // are the more tiles?
        if !self.is_empty() {
            return;
        }
        // 1. Score and move tiles to tray/wall
        for player in &mut self.players {
            player.finish_up(&mut self.tray);
        }
        // 2. Deal new factories
    }
    fn current_player(&self) -> usize {
        self.moves % self.players.len()
    }
    fn is_game_over(&self) -> bool {
        // game is over if any player has any row with all cells filled
        self.players.iter().any(|player| player.wall.rows.iter().any(|row| row.iter().all(|cell| *cell)))
    }
    fn place_all(&self, tile: Tile, count: usize) -> Vec<State> {
        // Put the "count" number of "tile" on one row. Return a state for each
        // such placement. Furthermore the tiles cannot be placed anywhere, place
        // them in the discard
        let player_index = self.current_player();
        let states: Vec<_> = (0..5).flat_map(|row| {
            println!("    placing in row {}", row);
            let mut state = self.clone();
            state.players[player_index].maybe_place(tile, count, row).then_some(state)
        }).collect();
        if states.is_empty() {
            // player must discard all tiles :-(
            let mut state = self.clone();
            state.players[player_index].discard[tile] += count;
            vec![state]
        } else {
            states
        }
    }
    
    fn self_check(&self) {
        let n = self.tile_count();
        if n != 100 {
            println!("bad tile count {}", n);
            panic!();
        }
    }
}

trait GameState: Sized {
    fn children(&self) -> Vec<Self>;
    fn winner(&self) -> Option<usize>;
}

impl GameState for State {
    fn children(&self) -> Vec<Self> {
        let mut children = Vec::new();
        // take the tiles from one of the factories...
        for factory_index in 0..self.factories.len() {
            let mut state = self.clone();
            println!("Taking factory #{}", factory_index);
            let factory = state.factories.remove(factory_index);
            // ...and select one color
            for tile in TILES {
                // take tile and leave rest in center
                let mut state = state.clone();
                let mut factory = factory.clone();
                let count = factory.drain(tile);
                if count > 0 {
                    println!("  Taking {} of {:?}", count, tile);
                    state.center.extend(factory);
                    children.extend(state.place_all(tile, count));
                }
            }
        }
        // Or take all tiles of one type from the center
        for tile in TILES {
            // take tile from center
            let mut state = self.clone();
            let count = state.center.drain(tile);
            if count > 0 {
                println!("  Taking {:?} from center", tile);
                children.extend(state.place_all(tile, count));
            }
        }
        children
    }
    fn winner(&self) -> Option<usize> {
        if self.is_game_over() {
            self.players.iter().map(|player| player.points).max()
        } else {
            None
        }
    }
}

fn main() {
    let rng = &mut thread_rng();
    let mut state = State::new(2);
    state.deal(rng);
    while state.winner().is_none() {
        let children = state.children();
        state = children.choose(rng).unwrap().clone();
    }
}
