use std::{iter, ops::{Index, IndexMut}};

use rand::{seq::SliceRandom, thread_rng, Rng};


#[derive(Clone, Copy, Debug)]
enum Tile {
    BLACK,
    WHITE,
    AZUL,
    YELLOW,
    RED,
}

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

impl TileSet {
    fn new() -> Self {
        Self { black: 0, white: 0, azul: 0, yellow: 0, red: 0 }
    }
    fn drain(&mut self, tile: Tile) -> usize {
        let count = self[tile];
        self[tile] = 0;
        count
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
}

#[derive(Clone)]
struct State {
    bag: Vec<Tile>,
    factories: Vec<[Tile; 4]>,
    center: Vec<Tile>,
    players: Vec<Player>,
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
        Self { bag, factories: Vec::new(), center: Vec::new(), players }
    }
    fn deal<R: Rng>(&self, rng: &mut R) -> Self{
        let mut state = self.clone();
        // shuffle bag
        state.bag.shuffle(rng);
        // deal factories
        for _ in 0..5 {
            // TODO: What if the bag is empty?
            let tiles: Vec<_> = state.bag.drain(0..4).collect();
            state.factories.push(tiles.try_into().unwrap());
        }
        state
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
        for index in 0..self.factories.len() {
            let mut state = self.clone();
            let factory = state.factories.remove(index);
            // ...and select one color
            state.center.extend(factory);
            children.push(state);
        }
        children
    }
    fn winner(&self) -> Option<usize> {
        None
    }
}


fn main() {
    let mut rng = thread_rng();
    let state = State::new(2).deal(&mut rng);
    for state in state.children() {
        println!("{}", state.bag.len())
    }
}
