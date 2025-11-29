mod azul;
mod azul_fmt;
mod minmax;

use crate::{
    azul::{describe_move, Fish, State},
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
        let next_state = if state.current_player() == 0 {
            search(&state, &mut evaluation, 4, &mut rng).unwrap()
        } else {
            random_move(&state, &mut rng)
        };
        match describe_move(&state, &next_state) {
            Ok(description) => println!("{}", description),
            Err(err) => println!("Unable to describe move: {}", err),
        }
        state = next_state;
        //state.self_check();
    }
    print_state(&state, &names);
    for (index, player) in state.players.iter().enumerate() {
        println!("player {}, {}", names[index], player.points);
    }
}
