use rand::{seq::SliceRandom, Rng};
use std::hash::Hash;

#[derive(Clone)]
pub enum GameState<D, S> {
    Deterministic(D),
    Stochastic(S),
}

pub trait DeterministicGameState: Sized + Clone + Hash + Eq {
    type Stochastic: StochasticGameState<Deterministic = Self>;

    fn current_player(&self) -> usize;
    fn children(&self) -> Vec<GameState<Self, Self::Stochastic>>;
    fn winner(&self) -> Option<usize>;

    fn select<R: Rng>(&self, _rng: &mut R) -> GameState<Self, Self::Stochastic> {
        GameState::Deterministic(self.clone())
    }
}

pub trait StochasticGameState: Sized + Clone + Hash + Eq {
    type Deterministic: DeterministicGameState<Stochastic = Self>;

    // TODO: outcomes should be able to be stochastic themselves
    fn outcomes(&self) -> Vec<(f32, Self::Deterministic)>;
}

pub trait Evaluation<S: DeterministicGameState> {
    fn evaulate(&self, state: &S, player: usize) -> i32;

    // TODO: Move heuristic into separate trait

    // may re-order (but not modify) states
    fn update(&mut self, _state: &S, _value: i32) {}
    fn heuristic(&self, _states: &mut Vec<S>) {}
}

// search code. returns child index and evaluation
pub fn minmax<S: DeterministicGameState, E: Evaluation<S>>(
    state: GameState<S, S::Stochastic>,
    evaluation: &mut E,
    player: usize,
    depth: usize,
    alpha: i32,
    beta: i32,
) -> (Option<usize>, i32) {
    match state {
        GameState::Deterministic(state) => {
            if depth == 0 {
                let e = evaluation.evaulate(&state, state.current_player());
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
                let mut children: Vec<_> = state
                    .children()
                    .into_iter()
                    .filter_map(|child| match child {
                        GameState::Deterministic(child) => Some(child),
                        GameState::Stochastic(_) => None,
                    })
                    .collect();
                evaluation.heuristic(&mut children);
                for (index, child) in children.iter().enumerate() {
                    let new_value = minmax(
                        GameState::Deterministic(child.clone()),
                        evaluation,
                        player,
                        depth - 1,
                        alpha,
                        beta,
                    )
                    .1;
                    if new_value >= best_value {
                        best_value = new_value;
                        best_index = Some(index);
                    }
                    if best_value > beta {
                        break; // β cutoff
                    }
                    alpha = alpha.max(best_value);
                }
                (best_index, best_value)
            } else {
                let mut best_value = i32::MAX;
                let mut best_index = None;
                let mut beta = beta;
                let mut children: Vec<_> = state
                    .children()
                    .into_iter()
                    .filter_map(|child| match child {
                        GameState::Deterministic(child) => Some(child),
                        GameState::Stochastic(_) => None,
                    })
                    .collect();
                evaluation.heuristic(&mut children);
                for (index, child) in children.iter().enumerate() {
                    let new_value = minmax(
                        GameState::Deterministic(child.clone()),
                        evaluation,
                        player,
                        depth - 1,
                        alpha,
                        beta,
                    )
                    .1;
                    if new_value <= best_value {
                        best_value = new_value;
                        best_index = Some(index);
                    }
                    if best_value < alpha {
                        break; // α cutoff
                    }
                    beta = beta.min(best_value);
                }
                (best_index, best_value)
            }
        }
        GameState::Stochastic(chance) => {
            let value =
                chance_value(&chance, evaluation, player, depth, alpha, beta).round() as i32;
            (None, value)
        }
    }
}

pub fn search<S: DeterministicGameState, E: Evaluation<S>>(
    state: &S,
    evaluation: &mut E,
    depth: usize,
) -> Option<S> {
    let player = state.current_player();
    if let Some(index) = minmax(
        GameState::Deterministic(state.clone()),
        evaluation,
        player,
        depth,
        i32::MIN,
        i32::MAX,
    )
    .0
    {
        // TODO: children called twice - once in minmax and once here...
        // They might get different bags due to rng
        let children: Vec<_> = state
            .children()
            .into_iter()
            .filter_map(|child| match child {
                GameState::Deterministic(child) => Some(child),
                GameState::Stochastic(_) => None,
            })
            .collect();
        children.get(index).cloned()
    } else {
        None
    }
}

pub fn random_move<S: DeterministicGameState, R: Rng>(state: &S, rng: &mut R) -> S {
    let children: Vec<_> = state
        .children()
        .into_iter()
        .filter_map(|child| match child {
            GameState::Deterministic(child) => Some(child),
            GameState::Stochastic(_) => None,
        })
        .collect();
    children.choose(rng).unwrap().clone()
}

fn chance_value<S: DeterministicGameState, E: Evaluation<S>>(
    chance: &S::Stochastic,
    evaluation: &mut E,
    player: usize,
    depth: usize,
    alpha: i32,
    beta: i32,
) -> f32 {
    chance
        .outcomes()
        .into_iter()
        .map(|(probability, outcome)| {
            let value = minmax(
                GameState::Deterministic(outcome),
                evaluation,
                player,
                depth,
                alpha,
                beta,
            )
            .1 as f32;
            probability * value
        })
        .sum()
}
