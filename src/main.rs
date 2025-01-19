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
}

#[derive(Clone)]
struct Player {
    rows: [Option<(Tile, usize)>; 5],
    points: usize,
    wall: [[Option<Tile>; 5]; 5],
    discard: TileSet,
}
impl Player {
    fn new() -> Self {
        Self {
            rows: Default::default(),
            points: 0,
            wall: Default::default(),
            discard: TileSet::new(),
        }
    }

    fn place(&mut self, tile: Tile, count: usize, row_index: usize) -> usize {
        let row_size = row_index + 1;
        let free = if let Some((existing_tile, existing_count)) = self.rows[row_index] {
            if existing_tile == tile {
                row_size - existing_count
            } else {
                0
            }
        } else {
            row_size
        };
        self.rows[row_index] = Some((tile, free));
        count.saturating_sub(free)
    }
}

#[derive(Clone)]
struct State {
    bag: TileSet,
    factories: Vec<TileSet>,
    center: TileSet,
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
        Self { bag, factories: Vec::new(), center: TileSet::new(), players, moves: 0 }
    }
    fn deal<R: Rng>(&mut self, rng: &mut R) {
        // deal factories
        for _ in 0..5 {
            // TODO: What if the bag is empty?
            let tiles = self.bag.draw(rng, 4);
            self.factories.push(tiles);
        }
    }
    fn current_player(&self) -> usize {
        self.moves % self.players.len()
    }
    fn is_game_over(&self) -> bool {
        // game is over if any player has any row with all cells filled
        self.players.iter().any(|player| player.wall.iter().any(|row| row.iter().all(|cell| cell.is_some())))
    }
    fn place_all(&self, tile: Tile, count: usize) -> Vec<State> {
        // Put the "count" number of "tile" on one row. Return a state for each
        // such placement. Furthermore the tiles cannot be placed anyway, place them in the
        // discard
        let player_index = self.current_player();
        let states: Vec<_> = (0..5).map(|row| {
            let mut state = self.clone();
            let discard_count = state.players[player_index].place(tile, count, row);
            state.players[player_index].discard[tile] += discard_count;
            state
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
                    println!("  Taking {:?}", tile);
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
