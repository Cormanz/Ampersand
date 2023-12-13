use std::{os::unix::thread, collections::HashMap};

use lazy_static::lazy_static;
use monster_chess::{board::Board, games::chess::pieces::{PAWN, BISHOP, KNIGHT, QUEEN, ROOK, KING}, bitboard::BitBoard};
use rand::{random, thread_rng, Rng, rngs::StdRng, SeedableRng};

use crate::search::SearchInfo;

lazy_static! {
    pub static ref MATERIAL: HashMap<usize, i32> = [
        (PAWN, 82),
        (KNIGHT, 337),
        (BISHOP, 365),
        (ROOK, 477),
        (QUEEN, 1025),
        (KING, 0)
    ].into();

    // yoinked from https://www.chessprogramming.org/Simplified_Evaluation_Function
    pub static ref PIECE_SQUARE_TABLES: HashMap<usize, [ i32; 64 ]> = [
        (PAWN, [
            0,   0,   0,   0,   0,   0,  0,   0,
            98, 134,  61,  95,  68, 126, 34, -11,
            -6,   7,  26,  31,  65,  56, 25, -20,
           -14,  13,   6,  21,  23,  12, 17, -23,
           -27,  -2,  -5,  12,  17,   6, 10, -25,
           -26,  -4,  -4, -10,   3,   3, 33, -12,
           -35,  -1, -20, -23, -15,  24, 38, -22,
             0,   0,   0,   0,   0,   0,  0,   0,
        ]),
        (KNIGHT, [
            -167, -89, -34, -49,  61, -97, -15, -107,
            -73, -41,  72,  36,  23,  62,   7,  -17,
            -47,  60,  37,  65,  84, 129,  73,   44,
             -9,  17,  19,  53,  37,  69,  18,   22,
            -13,   4,  16,  13,  28,  19,  21,   -8,
            -23,  -9,  12,  10,  19,  17,  25,  -16,
            -29, -53, -12,  -3,  -1,  18, -14,  -19,
           -105, -21, -58, -33, -17, -28, -19,  -23,
        ]),
        (BISHOP, [
            -29,   4, -82, -37, -25, -42,   7,  -8,
            -26,  16, -18, -13,  30,  59,  18, -47,
            -16,  37,  43,  40,  35,  50,  37,  -2,
             -4,   5,  19,  50,  37,  37,   7,  -2,
             -6,  13,  13,  26,  34,  12,  10,   4,
              0,  15,  15,  15,  14,  27,  18,  10,
              4,  15,  16,   0,   7,  21,  33,   1,
            -33,  -3, -14, -21, -13, -12, -39, -21,
        ]),
        (ROOK, [
            32,  42,  32,  51, 63,  9,  31,  43,
            27,  32,  58,  62, 80, 67,  26,  44,
            -5,  19,  26,  36, 17, 45,  61,  16,
           -24, -11,   7,  26, 24, 35,  -8, -20,
           -36, -26, -12,  -1,  9, -7,   6, -23,
           -45, -25, -16, -17,  3,  0,  -5, -33,
           -44, -16, -20,  -9, -1, 11,  -6, -71,
           -19, -13,   1,  17, 16,  7, -37, -26,
        ]),
        (QUEEN, [
            -28,   0,  29,  12,  59,  44,  43,  45,
            -24, -39,  -5,   1, -16,  57,  28,  54,
            -13, -17,   7,   8,  29,  56,  47,  57,
            -27, -27, -16, -16,  -1,  17,  -2,   1,
             -9, -26,  -9, -10,  -2,  -4,   3,  -3,
            -14,   2, -11,  -2,  -5,   2,  14,   5,
            -35,  -8,  11,   2,   8,  15,  -3,   1,
             -1, -18,  -9,  10, -15, -25, -31, -50,
        ]),
        (KING, [
            -65,  23,  16, -15, -56, -34,   2,  13,
            29,  -1, -20,  -7,  -8,  -4, -38, -29,
            -9,  24,   2, -16, -20,   6,  22, -22,
           -17, -20, -12, -27, -30, -25, -14, -36,
           -49,  -1, -27, -39, -46, -44, -33, -51,
           -14, -14, -22, -46, -44, -30, -15, -27,
             1,   7,  -8, -64, -43, -16,   9,   8,
           -15,  36,  12, -54,   8, -28,  24,  14,
        ])
    ].into();

    pub static ref PIECE_SQUARE_TABLES_REVERSED: HashMap<usize, [ i32; 64 ]> = PIECE_SQUARE_TABLES
        .iter()
        .map(|entry| (*entry.0, reverse_pst(entry.1)))
        .collect();
}

pub fn reverse_pst(pst: &[ i32; 64 ]) -> [ i32; 64 ] {
    pst
        .chunks(8)
        .rev()
        .flat_map(|el| el)
        .map(|&el| el)
        .collect::<Vec<_>>()
        .try_into().expect("Must be convertable")


}

struct Teams<const T: usize> {
    team: BitBoard<T>, 
    opposing_team: BitBoard<T>,
    team_ind: usize
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

    fn piece_square_difference(
        &self,
        pieces: BitBoard<T>, 
        piece_square_table: [ i32; 64 ],
        piece_square_table_reversed: [ i32; 64 ]
    ) -> i32 {
        let (white, black) = if self.team_ind == 0 {
            (self.team, self.opposing_team)
        } else {
            (self.opposing_team, self.team)
        };

        let white_pieces = (pieces & white)
            .iter_set_bits(64)
            .map(|pos| piece_square_table[pos as usize])
            .reduce(|acc, cur| acc + cur)
            .unwrap_or(0);

        let black_pieces = (pieces & black)
            .iter_set_bits(64)
            .map(|pos| piece_square_table_reversed[pos as usize])
            .reduce(|acc, cur| acc + cur)
            .unwrap_or(0);

        let (home_pieces, opposing_pieces) = if self.team_ind == 0 {
            (white_pieces, black_pieces)
        } else {
            (black_pieces, white_pieces)
        };

        home_pieces - opposing_pieces
    }

    fn difference(   
        &self,
        pieces: BitBoard<T>, 
        piece_type: usize
    ) -> i32 {
        self.material_difference(pieces, MATERIAL[&piece_type]) +
        self.piece_square_difference(pieces, PIECE_SQUARE_TABLES[&piece_type], PIECE_SQUARE_TABLES_REVERSED[&piece_type])
    }
}

pub fn evaluate<const T: usize>(
    board: &mut Board<T>
) -> i32 {
    let team_ind = board.state.moving_team as usize;
    let team = board.state.teams[team_ind];

    let opposing_team_ind = board.get_next_team(board.state.moving_team) as usize;
    let opposing_team = board.state.teams[opposing_team_ind];

    let teams = Teams {
        team,
        opposing_team,
        team_ind

    };

    let pawns = board.state.pieces[PAWN];
    let knights = board.state.pieces[KNIGHT];
    let bishops = board.state.pieces[BISHOP];
    let rooks = board.state.pieces[ROOK];
    let queens = board.state.pieces[QUEEN];
    
    teams.difference(pawns, PAWN) +
    teams.difference(knights, KNIGHT) +
    teams.difference(bishops, BISHOP) +
    teams.difference(rooks, ROOK) +
    teams.difference(queens, QUEEN)
}