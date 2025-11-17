mod azul;
mod minmax;

use crate::{
    azul::{Fish, State},
    minmax::{random_move, search, DeterministicGameState},
};
use rand::thread_rng;

fn main() {
    let mut rng = thread_rng();
    let mut evaluation = Fish::new();
    let mut state = State::new(2);
    let names = ["Samuel", "Maria"];
    state.deal(&mut rng);
    while state.winner().is_none() {
        println!("round {}: {}", state.moves, names[state.current_player()]);
        if state.current_player() == 0 {
            state = search(&state, &mut evaluation, &mut rng, 4).unwrap();
        } else {
            state = random_move(&state, &mut rng);
        }
    }
    for (index, player) in state.players.iter().enumerate() {
        println!("player {}, {}", names[index], player.points);
    }
}
