use std::{iter, ops::{Index, IndexMut}};

use rand::{distributions::WeightedIndex, prelude::Distribution, thread_rng, Rng};


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
    
    fn len(&self) -> usize {
        self.black + self.white + self.azul + self.yellow + self.red
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
}
impl Player {
    fn new() -> Self {
        Self { rows: Default::default(), points: 0, wall: Default::default() }
    }
    
    fn can_place(&self, tile: Tile, count: usize, index: usize) -> bool {
        let size = index + 1;
        if let Some((existing_tile, existing_count)) = self.rows[index] {
            tile == existing_tile && count <= (size - existing_count) 
        } else {
            count <= size
        }
    }
    
    fn place(&mut self, tile: Tile, count: usize, row: usize) {
        self.rows[row] = Some((tile, count))
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
            println!("{:?}", tiles);
            self.factories.push(tiles);
        }
    }
    
    fn current_player(&self) -> usize {
        self.moves % self.players.len()
    }
}

trait GameState: Sized {
    fn children(&self) -> Vec<Self>;
    fn winner(&self) -> Option<usize>;
}

impl GameState for State {
    fn children(&self) -> Vec<Self> {
        let mut children = Vec::new();
        let player_index = self.current_player();
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
                    // ...put the "count" number of "tile" on one row
                    for row in 0..5 {
                        if state.players[player_index].can_place(tile, count, row) {
                            println!("    Placing {} on row {}", count, row);
                            let mut state = state.clone();
                            state.players[player_index].place(tile, count, row);
                            children.push(state);
                        }
                    }
                }
            }
        }
        children
    }
    fn winner(&self) -> Option<usize> {
        None
    }
}


fn main() {
    let mut rng = thread_rng();
    let mut state = State::new(2);
    state.deal(&mut rng);

    println!("{}", state.children().len());
}
