use std::{os::unix::thread, collections::HashMap};

use lazy_static::lazy_static;
use monster_chess::{board::Board, games::chess::pieces::{PAWN, BISHOP, KNIGHT, QUEEN, ROOK, KING}, bitboard::BitBoard};
use rand::{random, thread_rng, Rng, rngs::StdRng, SeedableRng};

use crate::search::SearchInfo;

lazy_static! {
    pub static ref MATERIAL: HashMap<usize, i32> = [
        (PAWN, 100),
        (KNIGHT, 325),
        (BISHOP, 350),
        (ROOK, 500),
        (QUEEN, 900),
        (KING, 10_000)
    ].into();
}

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

    teams.material_difference(pawns, MATERIAL[&PAWN]) +
    teams.material_difference(knights, MATERIAL[&KNIGHT]) +
    teams.material_difference(bishops, MATERIAL[&BISHOP]) +
    teams.material_difference(rooks, MATERIAL[&ROOK]) +
    teams.material_difference(queens, MATERIAL[&QUEEN]) + rand
}