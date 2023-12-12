use std::os::unix::thread;

use monster_chess::{board::Board, games::chess::pieces::{PAWN, BISHOP, KNIGHT, QUEEN, ROOK}, bitboard::BitBoard};
use rand::{random, thread_rng, Rng, rngs::StdRng, SeedableRng};

use crate::search::SearchInfo;

struct Teams<const T: usize> {
    team: BitBoard<T>, 
    opposing_team: BitBoard<T>
}

impl <const T: usize> Teams<T> {
    fn material_difference(
        &self,
        pieces: BitBoard<T>, 
        material: i32
    ) -> i32 {
        let home_pieces = (pieces & self.team).count_ones() as i32;
        let opposing_pieces = (pieces & self.opposing_team).count_ones() as i32;
    
        material * (home_pieces - opposing_pieces)
    }
}

pub fn evaluate<const T: usize>(
    board: &mut Board<T>
) -> i32 {
    let team = board.state.teams[board.state.moving_team as usize];
    let opposing_team = board.state.teams[board.get_next_team(board.state.moving_team) as usize];

    let teams = Teams {
        team,
        opposing_team
    };

    let pawns = board.state.pieces[PAWN];
    let knights = board.state.pieces[KNIGHT];
    let bishops = board.state.pieces[BISHOP];
    let rooks = board.state.pieces[ROOK];
    let queens = board.state.pieces[QUEEN];

    let rand = thread_rng().gen_range(-20..20);

    teams.material_difference(pawns, 100) +
    teams.material_difference(knights, 325) +
    teams.material_difference(bishops, 350) +
    teams.material_difference(rooks, 500) +
    teams.material_difference(queens, 900) + rand
}