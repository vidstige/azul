use std::{iter, mem, ops::{Index, IndexMut}};

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
    fn count(&self, (y, x): (usize, usize), dx: i32, dy: i32) -> usize {
        let mut points: usize = 0;
        let (mut y, mut x) = (y, x);
        while (0..5).contains(&y) && (0..5).contains(&x) && self.rows[y][x] {
            points += 1;
            x = (x as i32 + dx) as usize;
            y = (y as i32 + dy) as usize;
        }
        points
    }
    fn points_at(&self, colum_index: usize, row_index: usize) -> usize {
        let mut points: usize = 0;
        let position = (row_index, colum_index);
        points += self.count(position, 1, 0);  // right
        points += self.count(position, -1, 0);  // left
        points += self.count(position, 0, -1);  // up
        points += self.count(position, 0, 1);  // down
        points -= 3; // center is counted thrice
        points
    }
    fn bonus_points_at(&self, colum_index: usize, row_index: usize) -> usize {
        [
            (0..5).all(|column| self.rows[row_index][column]).then_some(2),
            (0..5).all(|row| self.rows[row][colum_index]).then_some(7),
            // TODO: Bonus points if all tiles placed.
        ].iter().flatten().sum()
    }
    fn add_tile(&mut self, row_index: usize, tile: Tile) -> usize {
        let colum_index = WALL[row_index].iter().position(|cell| cell == &tile).unwrap();
        assert!(!self.rows[row_index][colum_index], "Tile was already assigned!");
        self.rows[row_index][colum_index] = true;
        self.points_at(colum_index, row_index) + self.bonus_points_at(colum_index, row_index)
    }
}

fn discard_points(count: usize) -> usize {
    const SLOTS: [usize; 5] = [1, 1, 2, 2, 2];
    (0..count).map(|i| if i < SLOTS.len() { SLOTS[i] } else { 3 }).sum()
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
        if self.wall.has_tile(row_index, &tile) {
            return false;
        }
        let row_size = row_index + 1;
        if let Some((current_tile, ref mut current_count)) = self.rows[row_index] {
            if current_tile != tile {
                // another tile is used - we can't place here at all
                return false;
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
    
    fn prepare_next_round(&mut self, tray: &mut TileSet) {
        // start by going through rows and award points for filled rows
        for (row_index, row) in self.rows.iter_mut().enumerate() {
            let row_size = row_index + 1;
            if let Some((tile, count)) = row.clone() {
                if count == row_size {
                    let points = self.wall.add_tile(row_index, tile); // add one tile to wall
                    self.points += points;                    
                    tray[tile] += count - 1; // add rest back to tray
                    *row = None;  // clear row
                }
            }
        }
        // subtract tiles in discard
        self.points = self.points.saturating_sub(discard_points(self.discard.len()));
        // move discard into tray
        let mut tmp = TileSet::new();
        mem::swap(&mut tmp, &mut self.discard);
        tray.extend(tmp);
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
struct State<R: Rng + Clone> {
    rng: R,
    bag: TileSet,
    factories: Vec<TileSet>,
    center: TileSet,
    tray: TileSet,
    players: Vec<Player>,
    moves: usize,
}

impl<R> State<R> where R: Rng + Clone {
    fn new(players: usize, rng: R) -> Self {
        let bag = [
            iter::repeat(Tile::BLACK).take(20),
            iter::repeat(Tile::WHITE).take(20),
            iter::repeat(Tile::AZUL).take(20),
            iter::repeat(Tile::YELLOW).take(20),
            iter::repeat(Tile::RED).take(20),
        ].into_iter().flat_map(|it| it).collect();
        let players = iter::repeat(Player::new()).take(players).collect();
        Self {
            rng,
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
    fn deal(&mut self) {
        let n = 5; // TODO: Compute based on number of players
        if self.bag.len() < 4 * n {
            // move tiles from tray to bag
            let mut tmp = TileSet::new();
            mem::swap(&mut tmp, &mut self.tray);
            self.bag.extend(tmp);
        }
        // deal factories
        for _ in 0..n {
            let tiles = self.bag.draw(&mut self.rng, 4);
            self.factories.push(tiles);
        }
    }
    fn is_empty(&self) -> bool {
        self.factories.iter().map(|factory| factory.len()).sum::<usize>() + self.center.len() == 0
    }
    // clean up by updating score, dealing new tiles, etc
    fn prepare_next_round(&mut self) {
        // are the more tiles?
        if !self.is_empty() {
            return;
        }
        // 1. Score and move tiles to tray/wall
        for player in &mut self.players {
            player.prepare_next_round(&mut self.tray);
        }
        // 2. Deal new factories
        self.deal();
    }
    fn is_game_over(&self) -> bool {
        // game is over if any player has any row with all cells filled
        self.players.iter().any(|player| player.wall.rows.iter().any(|row| row.iter().all(|cell| *cell)))
    }
    fn place_all(&self, tile: Tile, count: usize) -> Vec<Self> {
        // Put the "count" number of "tile" on one row. Return a state for each
        // such placement. Furthermore the tiles cannot be placed anywhere, place
        // them in the discard
        let player_index = self.current_player();
        let states: Vec<_> = (0..5).flat_map(|row| {
            //println!("    placing in row {}", row);
            let mut state = self.clone();
            if state.players[player_index].maybe_place(tile, count, row) {
                state.prepare_next_round();
                Some(state)
            } else {
                None
            }
        }).collect();
        if states.is_empty() {
            // player must discard all tiles :-(
            let mut state = self.clone();
            state.players[player_index].discard[tile] += count;
            state.prepare_next_round();
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

trait GameState: Sized + Clone {
    fn current_player(&self) -> usize;
    fn children(&self) -> Vec<Self>;
    fn winner(&self) -> Option<usize>;
}

impl<R> GameState for State<R> where R: Rng + Clone {
    fn current_player(&self) -> usize {
        self.moves % self.players.len()
    }
    fn children(&self) -> Vec<Self> {
        let mut children = Vec::new();
        // take the tiles from one of the factories...
        for factory_index in 0..self.factories.len() {
            let mut state = self.clone();
            //println!("Taking factory #{}", factory_index);
            let factory = state.factories.remove(factory_index);
            // ...and select one color
            for tile in TILES {
                // take tile and leave rest in center
                let mut factory = factory.clone();
                let count = factory.drain(tile);
                if count > 0 {
                    let mut state = state.clone();
                    //println!("  Taking {} of {:?}", count, tile);
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
                //println!("  Taking {:?} from center", tile);
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

// evaulation code
trait Evaluation<S: GameState> {
    fn evaulate(&self, state: &S, player: usize) -> i32;
}

impl<R> Evaluation<State<R>> for State<R> where R: Rng + Clone {
    fn evaulate(&self, state: &State<R>, player: usize) -> i32 {
        state.players[player].points as i32
    }
}

// search code
fn minmax<S: GameState, E: Evaluation<S>>(state: S, evaluation: &E, player: usize, depth: usize, alpha: i32, beta: i32) -> (S, i32) {
    if depth == 0 {
        let e = evaluation.evaulate(&state, state.current_player());
        return (state, e);
    }
    if let Some(winner) = state.winner() {
        if winner == player {
            return (state, i32::MAX);
        } else {
            return (state, i32::MIN);
        }
    }
    
    if state.current_player() == player {
        let mut best_value = i32::MIN;
        let mut best_state = None;
        let mut alpha = alpha;
        for child in state.children() {
            let (new_state, new_value) = minmax(child, evaluation, player, depth - 1, alpha, beta);
            if new_value >= best_value {
                best_value = new_value;
                best_state = Some(new_state);
            }            
            if best_value > beta {
                break // β cutoff
            }
            alpha = alpha.max(best_value);
        }
        (best_state.unwrap(), best_value)
    } else {
        let mut best_value = i32::MAX;
        let mut best_state = None;
        let mut beta = beta;
        for child in state.children() {
            let (new_state, new_value) = minmax(child, evaluation, player, depth - 1, alpha, beta);
            if new_value <= best_value {
                best_value = new_value;
                best_state = Some(new_state);
            }            
            if best_value < alpha {
                break // α cutoff
            }
            beta = beta.min(best_value);
        }
        (best_state.unwrap(), best_value)
    }
}

fn search<S: GameState, E: Evaluation<S>>(state: S, evaluation: &E, depth: usize) -> S {
    let player = state.current_player();
    minmax(state, evaluation, player, depth, i32::MIN, i32::MAX).0
}

// Could not come up with a good name for a basic stupid evaluation
struct Fish {

}
impl Fish {
    fn new() -> Self {
        Fish {}
    }
}
impl<R: Rng + Clone> Evaluation<State<R>> for Fish {
    fn evaulate(&self, state: &State<R>, player: usize) -> i32 {
        state.players[player].points as i32
    }
}

fn random_move<S: GameState, R: Rng>(state: &S, rng: &mut R) -> S {
    let children = state.children();
    children.choose(rng).unwrap().clone()
}

fn main() {
    let mut rng = thread_rng();
    let evaluation = Fish::new();
    let mut state = State::new(2, thread_rng());
    state.deal();
    while state.winner().is_none() {
        if state.current_player() == 0 {
            state = search(state, &evaluation, 4);
        } else {
            state = random_move(&state, &mut rng);
        }
    }
    for (index, player) in state.players.iter().enumerate() {
        println!("player #{}, {}", index, player.points);
    }
}
