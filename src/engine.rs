use monster_chess::board::{Board, tests::get_time_ms};
use rand::{rngs::ThreadRng, SeedableRng};

use monster_ugi::engine::{EngineBehavior, Info, TimeControl, MoveSelectionResults, EngineInfo};

use crate::search::{SearchInfo, negamax, SearchEnd};

pub struct AmpersandEngine<const T: usize>(pub ThreadRng);

impl<const T: usize> EngineBehavior<T> for AmpersandEngine<T> {
    fn select_move(&mut self, board: &mut Board<T>, time_control: TimeControl, _hashes: &Vec<u64>) -> MoveSelectionResults {
        let mut max_depth = 100_000;
        let search_end = match time_control {
            TimeControl::Depth(depth) => {
                max_depth = depth;
                SearchEnd::None
            },
            TimeControl::MoveTime(movetime) => SearchEnd::Time(get_time_ms() + movetime),
            TimeControl::Nodes(nodes) => SearchEnd::Nodes(nodes),
            TimeControl::Timed(players) => {
                let player = &players[board.state.moving_team as usize];
                let time = (player.time_ms / 20) + ((player.inc_ms * 3) / 4);

                SearchEnd::Time(get_time_ms() + time)
            },
            TimeControl::Infinite => SearchEnd::None
        };

        let mut depth: u32 = 0;

        let mut search_info = SearchInfo {
            best_move: None,
            search_end,
            nodes: 0,
            ended: false
        };
        let mut score: i32 = 0;
        let mut total_time: u128 = 0;
        let mut total_nodes: u64 = 0;

        loop {
            depth += 1;
            let start = get_time_ms();

            search_info.nodes = 0;

            let new_score = negamax(
                board, 
                &mut search_info, 
                depth, 
                0, 
                -1_000_000,
                1_000_000
            );
            if search_info.ended {
                break;
            }
            
            score = new_score;

            let end = get_time_ms();
            let mut time = end - start;
            
            if time == 0 { time = 1; }

            total_time += time;
            total_nodes += search_info.nodes;

            self.info(Info {
                depth: Some(depth),
                pv: Some(&board.encode_action(&search_info.best_move.expect("We need a best move!!!"))),
                score: Some(score),
                nodes: Some(search_info.nodes),
                nps: Some((search_info.nodes / (time as u64)) * 1000),
                time: Some(time)
            });

            if depth >= max_depth {
                break;
            }
        }

        MoveSelectionResults {
            best_move: search_info.best_move.expect("Must have best move"),
            evaluation: score
        }
    }

    fn get_engine_info(&mut self) -> EngineInfo {
        EngineInfo {
            name: "Ampersand v0.0.3",
            author: "Corman"
        }
    }

    fn is_ready(&mut self) -> bool {
        true
    }

    fn stop_search(&mut self) {}
}