use monster_chess::{board::{Board, game::{NORMAL_MODE, GameResults, MoveLegalResponse}, actions::{Move, SimpleMove, HistoryMove}, tests::get_time_ms}, games::chess::{pieces::{KING, PAWN, QUEEN, KNIGHT}, ATTACKS_MODE}, bitboard::BitBoard};
use rand::rngs::StdRng;

use crate::evaluate::{evaluate, MATERIAL};

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

pub fn move_score<const T: usize>(
    board: &mut Board<T>, 
    action: &Move
) -> i32 {
    match action {
        SimpleMove::Pass => {
            0
        },
        SimpleMove::Action(action) => {
            let opposing_team = board.state.teams[board.get_next_team(board.state.moving_team) as usize];
    
            let dest = BitBoard::from_lsb(action.to);
            if !(dest & opposing_team).is_set() {
                return 0;
            }

            let mut captured_piece_type: usize = PAWN;
            for piece_type in PAWN..KING {
                if (board.state.pieces[piece_type] & dest).is_set() {
                    captured_piece_type = piece_type;
                    break;
                }
            }

            let moved = MATERIAL[&(action.piece_type as usize)];
            let captured = MATERIAL[&captured_piece_type];            

            1000 + (captured - moved)
        }
    }
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

    let moves = board.generate_moves(NORMAL_MODE);

    let mut scored_moves = moves.into_iter().map(|action| {
        let score = move_score(board, &action);
        (action, score)
    }).collect::<Vec<_>>();

    scored_moves.sort_by(|&a, &b| {
        b.1.cmp(&a.1)
    });

    let mut legal_moves: Vec<Move> = vec![];

    for (action, _score) in scored_moves {
        let mut undo: Option<HistoryMove<T>> = None;
        let MoveLegalResponse { is_legal, made_move } = board.game.controller.is_legal(board, &action, false);
        if !is_legal {
            if let Some(made_move) = made_move {
                board.undo_move(made_move);
                continue;
            }
        }

        if let Some(made_move) = made_move {
            undo = made_move;
        }
        
        if undo.is_none() {
            undo = board.make_move(&action);
        }

        search_info.nodes += 1;
        let score = -negamax(board, search_info, depth - 1, ply + 1, -beta, -alpha);
        board.undo_move(undo);

        if score > max {
            best_move = Some(action);
            max = score;
        }

        if max > alpha {
            alpha = max;
        }

        legal_moves.push(action);

        if alpha >= beta {
            break; // Beta cutoff
        }
    }

    if ply == 0 && !search_info.ended {
        search_info.best_move = best_move;
    }

    let resolution = board.game.resolution.resolve(board, &legal_moves);

    match resolution {
        GameResults::Ongoing => {},
        GameResults::Draw => {
            return 0;
        },
        GameResults::Win(team) => {
            return if team == board.state.moving_team { 100_000 - (ply as i32) } else { -100_000 + (ply as i32) };
        }
    }
    
    return max;
}