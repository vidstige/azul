use rand::{seq::SliceRandom, Rng};
use std::hash::Hash;

pub trait GameState: Sized + Clone + Hash + Eq {
    fn current_player(&self) -> usize;
    fn children<R: Rng>(&self, rng: &mut R) -> Vec<Self>;
    fn winner(&self) -> Option<usize>;
}

pub trait Evaluation<S: GameState> {
    fn evaulate(&self, state: &S, player: usize) -> i32;

    // TODO: Move heuristic into separate trait

    // may re-order (but not modify) states
    fn update(&mut self, _state: &S, _value: i32) {}
    fn heuristic(&self, _states: &mut Vec<S>) {}
}

// search code. returns child index and evaluation
pub fn minmax<S: GameState, E: Evaluation<S>, R: Rng>(
    state: &S,
    evaluation: &mut E,
    rng: &mut R,
    player: usize,
    depth: usize,
    alpha: i32,
    beta: i32,
) -> (Option<usize>, i32) {
    if depth == 0 {
        let e = evaluation.evaulate(&state, state.current_player());
        //evaluation.update(&state, e);
        return (None, e);
    }
    if let Some(winner) = state.winner() {
        return if winner == player {
            (None, i32::MAX)
        } else {
            (None, i32::MIN)
        };
    }

    if state.current_player() == player {
        let mut best_value = i32::MIN;
        let mut best_index = None;
        let mut alpha = alpha;
        let mut children = state.children(rng);
        evaluation.heuristic(&mut children);
        for (index, child) in children.iter().enumerate() {
            let new_value = minmax(child, evaluation, rng, player, depth - 1, alpha, beta).1;
            if new_value >= best_value {
                best_value = new_value;
                best_index = Some(index);
            }
            if best_value > beta {
                break; // β cutoff
            }
            alpha = alpha.max(best_value);
        }
        //evaluation.update(best_state.as_ref().unwrap(), best_value);
        (best_index, best_value)
    } else {
        let mut best_value = i32::MAX;
        let mut best_index = None;
        let mut beta = beta;
        let mut children = state.children(rng);
        evaluation.heuristic(&mut children);
        for (index, child) in children.iter().enumerate() {
            let new_value = minmax(child, evaluation, rng, player, depth - 1, alpha, beta).1;
            if new_value <= best_value {
                best_value = new_value;
                best_index = Some(index);
            }
            if best_value < alpha {
                break; // α cutoff
            }
            beta = beta.min(best_value);
        }
        //evaluation.update(best_state.as_ref().unwrap(), best_value);
        (best_index, best_value)
    }
}

pub fn search<S: GameState, E: Evaluation<S>, R: Rng>(
    state: &S,
    evaluation: &mut E,
    rng: &mut R,
    depth: usize,
) -> Option<S> {
    let player = state.current_player();
    if let Some(index) = minmax(state, evaluation, rng, player, depth, i32::MIN, i32::MAX).0 {
        // TODO: children called twice - once in minmax and once here...
        // The might get different bags due to rng
        let children = state.children(rng);
        Some(children[index].clone())
    } else {
        None
    }
}

pub fn random_move<S: GameState, R: Rng>(state: &S, rng: &mut R) -> S {
    let children = state.children(rng);
    children.choose(rng).unwrap().clone()
}
