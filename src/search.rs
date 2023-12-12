use monster_chess::{board::{Board, game::{NORMAL_MODE, GameResults}, actions::Move, tests::get_time_ms}, games::chess::{pieces::KING, ATTACKS_MODE}};
use rand::rngs::StdRng;

use crate::evaluate::evaluate;

#[derive(Debug, Clone, Copy)]
pub enum SearchEnd {
    Time(u128),
    Nodes(u64),
    None
}

pub struct SearchInfo {
    pub best_move: Option<Move>,
    pub nodes: u64,
    pub search_end: SearchEnd,  
    pub ended: bool
}

pub fn negamax<const T: usize>(
    board: &mut Board<T>, 
    search_info: &mut SearchInfo,
    depth: u32, 
    ply: u32,
    mut alpha: i32,
    beta: i32
) -> i32 {
    if depth == 0 { 
        let eval = evaluate(board);
        return eval;
    }

    if search_info.ended { return 0; }

    if depth > 1 {
        let end_early = match search_info.search_end {
            SearchEnd::Nodes(nodes) => search_info.nodes >= nodes,
            SearchEnd::Time(time) => get_time_ms() >= time,
            SearchEnd::None => false
        };

        if end_early {
            search_info.ended = true;
            return 0;
        }
    }

    let mut max = -1_000_000;
    let mut best_move: Option<Move> = None;

    let moves = board.generate_legal_moves(NORMAL_MODE);
    
    let resolution = board.game.resolution.resolve(board, &moves);

    match resolution {
        GameResults::Ongoing => {},
        GameResults::Draw => {
            return 0;
        },
        GameResults::Win(team) => {
            return if team == board.state.moving_team { 100_000 - (ply as i32) } else { -100_000 + (ply as i32) };
        }
    }

    for action in moves {

        search_info.nodes += 1;
        let undo = board.make_move(&action);
        let score = -negamax(board, search_info, depth - 1, ply + 1, -beta, -alpha);
        board.undo_move(undo);

        if score > max {
            best_move = Some(action);
            max = score;
        }

        if max > alpha {
            alpha = max;
        }

        if alpha >= beta {
            break; // Beta cutoff
        }
    }

    if ply == 0 && !search_info.ended {
        search_info.best_move = best_move;
    }
    
    return max;
}