use engine::AmpersandEngine;
use monster_chess::games::chess::Chess;
use monster_ugi::{engine::Engine, ugi::run_ugi};

mod evaluate;
mod engine;
mod search;

fn main() {
    run_ugi(Engine {
        game: Chess::create(),
        behavior: Box::new(AmpersandEngine(rand::thread_rng()))
    });
}
