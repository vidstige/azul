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
}

pub trait Outcomes<D, S> {
    fn sample<R: Rng>(&self, rng: &mut R, samples: usize) -> Vec<(f32, GameState<D, S>)>;
}

pub trait StochasticGameState: Sized + Clone + Hash + Eq {
    type Deterministic: DeterministicGameState<Stochastic = Self>;
    type Outcomes: Outcomes<Self::Deterministic, Self>;

    fn outcomes(&self) -> Self::Outcomes;
}

pub trait Evaluation<S: DeterministicGameState> {
    fn evaulate(&self, state: &S, player: usize) -> i32;

    // TODO: Move heuristic into separate trait

    // may re-order (but not modify) states
    fn update(&mut self, _state: &S, _value: i32) {}
    fn heuristic(&self, _states: &mut Vec<GameState<S, S::Stochastic>>) {}
}

// search code. returns child index and evaluation
pub fn minmax<S: DeterministicGameState, E: Evaluation<S>, R: Rng>(
    state: GameState<S, S::Stochastic>,
    evaluation: &mut E,
    rng: &mut R,
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
                let mut children = state.children();
                evaluation.heuristic(&mut children);
                for (index, child) in children.into_iter().enumerate() {
                    let new_value = minmax(
                        child.clone(),
                        evaluation,
                        rng,
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
                let mut children = state.children();
                evaluation.heuristic(&mut children);
                for (index, child) in children.into_iter().enumerate() {
                    let new_value = minmax(
                        child.clone(),
                        evaluation,
                        rng,
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
            let value = chance_value(&chance, evaluation, rng, player, depth, alpha, beta).round()
                as i32;
            (None, value)
        }
    }
}

pub fn search<S: DeterministicGameState, E: Evaluation<S>, R: Rng>(
    state: &S,
    evaluation: &mut E,
    depth: usize,
    rng: &mut R,
) -> Option<S> {
    let player = state.current_player();
    if let Some(index) = minmax(
        GameState::Deterministic(state.clone()),
        evaluation,
        rng,
        player,
        depth,
        i32::MIN,
        i32::MAX,
    )
    .0
    {
        // TODO: children called twice - once in minmax and once here...
        // They might get different bags due to rng
        let children: Vec<_> = state.children();
        children.get(index).cloned().and_then(|child| match child {
            GameState::Deterministic(child) => Some(child),
            GameState::Stochastic(_) => None,
        })
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

fn chance_value<S: DeterministicGameState, E: Evaluation<S>, R: Rng>(
    chance: &S::Stochastic,
    evaluation: &mut E,
    rng: &mut R,
    player: usize,
    depth: usize,
    alpha: i32,
    beta: i32,
) -> f32 {
    const SAMPLE_COUNT: usize = 32;
    let samples = chance.outcomes().sample(rng, SAMPLE_COUNT.max(1));
    let total_weight: f32 = samples.iter().map(|(probability, _)| *probability).sum();
    assert!(
        total_weight > 0.0,
        "chance node sampling produced zero total probability"
    );
    samples
        .into_iter()
        .map(|(probability, outcome)| {
            let normalized = probability / total_weight;
            let value = minmax(outcome, evaluation, rng, player, depth, alpha, beta).1 as f32;
            normalized * value
        })
        .sum()
}
