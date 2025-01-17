use std::iter;

use rand::{seq::SliceRandom, thread_rng, Rng};


#[derive(Clone)]
enum Tile {
    FIRST,
    BLACK,
    WHITE,
    AZUL,
    YELLOW,
    RED,
}

#[derive(Clone)]
struct Player {
    rows: [Vec<Tile>; 5],
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
    fn new<R: Rng>(players: usize, rng: &mut R) -> State {
        let mut bag: Vec<_> = [
            iter::repeat(Tile::BLACK).take(20),
            iter::repeat(Tile::WHITE).take(20),
            iter::repeat(Tile::AZUL).take(20),
            iter::repeat(Tile::YELLOW).take(20),
            iter::repeat(Tile::RED).take(20),
        ].into_iter().flat_map(|it| it).collect();
        bag.shuffle(rng);
        let players = iter::repeat(Player::new()).take(players).collect();
        Self { bag, factories: Vec::new(), center: Vec::new(), players }
    }
}

trait GameState: Sized {
    fn children(&self) -> Vec<Self>;
    fn winner(&self) -> Option<usize>;
}

impl GameState for State {
    fn children(&self) -> Vec<Self> {
        let mut children = Vec::new();
        for index in 0..self.factories.len() {
            let mut state = self.clone();
            let factory = state.factories.remove(index);
            state.center.extend(factory);
        }
        children
    }
    fn winner(&self) -> Option<usize> {
        None
    }
}


fn main() {
    let state = State::new(2, &mut thread_rng());
    for state in state.children() {
        println!("{}", state.bag.len())
    }
}
