mod azul;
mod azul_fmt;
mod minmax;

use crate::{
    azul::{Fish, State},
    azul_fmt::print_state,
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
        state.resolve_stochastic(&mut rng);
        print_state(&state, &names);
        //println!("round {}: {}", state.moves, names[state.current_player()]);
        if state.current_player() == 0 {
            state = search(&state, &mut evaluation, 4, &mut rng).unwrap();
        } else {
            state = random_move(&state, &mut rng);
        }
        //state.self_check();
    }
    print_state(&state, &names);
    for (index, player) in state.players.iter().enumerate() {
        println!("player {}, {}", names[index], player.points);
    }
}
