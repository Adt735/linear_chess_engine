use std::{net::TcpStream, sync::Mutex};

use ndarray::Array1;

use crate::{attacks::{get_bishop_attacks, get_queen_attacks, KING_ATTACKS}, bitboard::{count_bits, get_ls1b_index, print_bitboard, Board}, get_bit, pop_bit, set_bit, Side, Square::{self, *}, _csv_fen_to_bitboard::convert_board_to_csv, _linear_regression::{self, LinearModel}, _neural_network::{communicate, load, predict, Layer}};


// File masks
static mut FILE_MASKS:[u64;64] = [0;64];
static mut RANK_MASKS:[u64;64] = [0;64];
static mut ISOLATED_MASKS:[u64;64] = [0;64];
static mut WHITE_PASSED_MASKS:[u64;64] = [0;64];
static mut BLACK_PASSED_MASKS:[u64;64] = [0;64];
const GET_RANK:[usize;64] = [
    7,7,7,7,7,7,7,7,
    6,6,6,6,6,6,6,6,
    5,5,5,5,5,5,5,5,
    4,4,4,4,4,4,4,4,
    3,3,3,3,3,3,3,3,
    2,2,2,2,2,2,2,2,
    1,1,1,1,1,1,1,1,
    0,0,0,0,0,0,0,0,
];

fn set_file_rank_mask(file_number:usize, rank_number:usize) -> u64 {
    let mut mask:u64 = 0;

    for rank in 0..8 {
        for file in 0..8 {
            let square = rank*8 + file;

            if file_number == file {
                set_bit!(mask, square);
            } else if rank_number == rank {
                set_bit!(mask, square);
            }
        }
    }

    mask
}
pub unsafe fn init_evaluation_masks() {
    for rank in 0..8 {
        for file in 0..8 {
            let square = rank*8 + file;

            FILE_MASKS[square] |= set_file_rank_mask(file, 8);
            RANK_MASKS[square] |= set_file_rank_mask(8, rank);

            if file==0 {
                ISOLATED_MASKS[square] |= set_file_rank_mask(file+1, 8);
            } else {
                ISOLATED_MASKS[square] |= set_file_rank_mask(file-1, 8);
                ISOLATED_MASKS[square] |= set_file_rank_mask(file+1, 8);
            }

            WHITE_PASSED_MASKS[square] |= FILE_MASKS[square];
            WHITE_PASSED_MASKS[square] |= ISOLATED_MASKS[square];
            let mut mask:u64 = 0;
            for i in 0..rank {
                mask |= 0xFF << (i*8);
            }
            WHITE_PASSED_MASKS[square] &= mask;

            BLACK_PASSED_MASKS[square] |= FILE_MASKS[square];
            BLACK_PASSED_MASKS[square] |= ISOLATED_MASKS[square];
            let mut mask:u64 = 0;
            for i in (rank+1)..8 {
                mask |= 0xFF << (i*8);
            }
            BLACK_PASSED_MASKS[square] &= mask;
        }
    }
}


pub const MATERIAL_SCORE:[i32;12] = [
    100, 300, 350, 500, 1000, 10000,
    -100, -300, -350, -500, -1000, -10000
];

pub const PAWN_SCORE:[i32;64] = [
    90,  90,  90,  90,  90,  90,  90,  90,
    30,  30,  30,  40,  40,  30,  30,  30,
    20,  20,  20,  30,  30,  30,  20,  20,
    10,  10,  10,  20,  20,  10,  10,  10,
     5,   5,  10,  20,  20,   5,   5,   5,
     0,   0,   0,   5,   5,   0,   0,   0,
     0,   0,   0, -10, -10,   0,   0,   0,
     0,   0,   0,   0,   0,   0,   0,   0
];

pub const KNIGHT_SCORE:[i32;64] = [
    -5,   0,   0,   0,   0,   0,   0,  -5,
    -5,   0,   0,  10,  10,   0,   0,  -5,
    -5,   5,  20,  20,  20,  20,   5,  -5,
    -5,  10,  20,  30,  30,  20,  10,  -5,
    -5,  10,  20,  30,  30,  20,  10,  -5,
    -5,   5,  20,  10,  10,  20,   5,  -5,
    -5,   0,   0,   0,   0,   0,   0,  -5,
    -5, -10,   0,   0,   0,   0, -10,  -5
];

pub const BISHOP_SCORE:[i32;64] = [
     0,   0,   0,   0,   0,   0,   0,   0,
     0,   0,   0,   0,   0,   0,   0,   0,
     0,   20,   0,  10,  10,   0,   20,   0,
     0,   0,  10,  20,  20,  10,   0,   0,
     0,   0,  10,  20,  20,  10,   0,   0,
     0,  10,   0,   0,   0,   0,  10,   0,
     0,  30,   0,   0,   0,   0,  30,   0,
     0,   0, -10,   0,   0, -10,   0,   0

];

pub const ROOK_SCORE:[i32;64] = [
    50,  50,  50,  50,  50,  50,  50,  50,
    50,  50,  50,  50,  50,  50,  50,  50,
     0,   0,  10,  20,  20,  10,   0,   0,
     0,   0,  10,  20,  20,  10,   0,   0,
     0,   0,  10,  20,  20,  10,   0,   0,
     0,   0,  10,  20,  20,  10,   0,   0,
     0,   0,  10,  20,  20,  10,   0,   0,
     0,   0,   0,  20,  20,   0,   0,   0

];

pub const KING_SCORE:[i32;64] = [
     0,   0,   0,   0,   0,   0,   0,   0,
     0,   0,   5,   5,   5,   5,   0,   0,
     0,   5,   5,  10,  10,   5,   5,   0,
     0,   5,  10,  20,  20,  10,   5,   0,
     0,   5,  10,  20,  20,  10,   5,   0,
     0,   0,   5,  10,  10,   5,   0,   0,
     0,   5,   5,  -5,  -5,   0,   5,   0,
     0,   0,   5,   0, -15,   0,  10,   0
];

const MIRROR_SCORE:[usize;64] = [
	a1 as usize, b1 as usize, c1 as usize, d1 as usize, e1 as usize, f1 as usize, g1 as usize, h1 as usize,
	a2 as usize, b2 as usize, c2 as usize, d2 as usize, e2 as usize, f2 as usize, g2 as usize, h2 as usize,
	a3 as usize, b3 as usize, c3 as usize, d3 as usize, e3 as usize, f3 as usize, g3 as usize, h3 as usize,
	a4 as usize, b4 as usize, c4 as usize, d4 as usize, e4 as usize, f4 as usize, g4 as usize, h4 as usize,
	a5 as usize, b5 as usize, c5 as usize, d5 as usize, e5 as usize, f5 as usize, g5 as usize, h5 as usize,
	a6 as usize, b6 as usize, c6 as usize, d6 as usize, e6 as usize, f6 as usize, g6 as usize, h6 as usize,
	a7 as usize, b7 as usize, c7 as usize, d7 as usize, e7 as usize, f7 as usize, g7 as usize, h7 as usize,
	a8 as usize, b8 as usize, c8 as usize, d8 as usize, e8 as usize, f8 as usize, g8 as usize, h8 as usize
];

// PAWNS
const DOUBLE_PAWN_PENALTY:i32 = -10;
const ISOLATED_PAWN_PENALTY:i32 = -10;
const PASSED_PAWN_BONUS:[i32;8] = [ 0, 5, 10, 20, 35, 60, 100, 200 ];

// SLIDING PIECES
const SEMI_OPEN_FILE_SCORE:i32 = 10;
const OPEN_FILE_SCORE:i32 = 15;

// King
const KING_SHIELD_BONUS:i32 = 5;


// For nn --------------------------------------------------
// let mut stream = TcpStream::connect(server_address).unwrap();
lazy_static::lazy_static! {
    pub static ref STREAM: Mutex<TcpStream> = Mutex::new(
        TcpStream::connect("127.0.0.1:5000").unwrap()
    );
}

lazy_static::lazy_static! {
    pub static ref NN_MODEL: Vec<Layer> = load();
}

pub static mut LINEAR_COEFF: LinearModel = LinearModel{ coefficients: vec![], intercept: 0.0 };
// lazy_static::lazy_static! {
//     pub static ref LINEAR_COEFF: LinearModel = {
//         let json_data = std::fs::read_to_string("C:/Users/adtro/Uni/MatCAD/3r/APC/kaggle/final/linear_model.json").expect("Failed to read JSON file");
//         serde_json::from_str(&json_data).expect("Failed to parse JSON")
//     };
// }


pub unsafe fn evaluate(board:&Board) -> i32 {
    // let inputs = convert_board_to_csv(board);
    // let mut stream = STREAM.lock().unwrap();
    // let score = (communicate("127.0.0.1:5000", &inputs) * 100.0).round() as i32;
    // return if board.side==Side::White {score} else {-score};
    //-----------------------------------------------------
    let raw_input: Vec<f64> = convert_board_to_csv(board).into_iter().map(|v| v as f64).collect();
    // let input = Array1::from(raw_input);
    // let score = (predict(input, &NN_MODEL)[0] * 100.0 )as i32;
    // return if board.side==Side::White {score} else {-score};
    //-----------------------------------------------------
    let score = (_linear_regression::predict(&LINEAR_COEFF, &raw_input) * 100.0) as i32;
    return if board.side==Side::White {score} else {-score};
    //-----------------------------------------------------
    let mut score = 0;
    let mut bb:u64;
    let mut piece:usize;
    let mut square:usize;

    for bb_piece in 0..12 {
        let current_bb = board.bitboards[bb_piece];
        bb = current_bb;

        piece = bb_piece;
        while bb != 0 {
            square = get_ls1b_index(bb);

            // Material score
            score += MATERIAL_SCORE[piece];

            // Positional score
            match bb_piece {
                0 => {
                    score += PAWN_SCORE[square];

                    let doubled_pawns = count_bits(current_bb & FILE_MASKS[square]) - 1;
                    if doubled_pawns != 0 {
                        score += (doubled_pawns as i32) * DOUBLE_PAWN_PENALTY;
                    }

                    if current_bb & ISOLATED_MASKS[square] == 0 {
                        score += ISOLATED_PAWN_PENALTY;
                    }

                    if board.bitboards[6] & WHITE_PASSED_MASKS[square] == 0 {
                        score += PASSED_PAWN_BONUS[GET_RANK[square]];
                    }
                },
                1 => score += KNIGHT_SCORE[square],
                2 => {
                    score += BISHOP_SCORE[square];

                    score += count_bits(get_bishop_attacks(square, board.occupancies[2])) as i32;
                },
                3 => {
                    score += ROOK_SCORE[square];

                    if board.bitboards[0] & FILE_MASKS[square] == 0 {
                        score += SEMI_OPEN_FILE_SCORE;

                        if board.bitboards[6] & FILE_MASKS[square] == 0 {
                            score += OPEN_FILE_SCORE;
                        }
                    }
                },
                4 => {
                    score += count_bits(get_queen_attacks(square, board.occupancies[2])) as i32;
                }
                5 => {
                    score += KING_SCORE[square];

                    if board.bitboards[0] & FILE_MASKS[square] == 0 {
                        score -= SEMI_OPEN_FILE_SCORE;

                        if board.bitboards[6] & FILE_MASKS[square] == 0 {
                            score -= OPEN_FILE_SCORE;
                        }
                    }

                    score += count_bits(KING_ATTACKS[square] & board.occupancies[2]) as i32 * KING_SHIELD_BONUS;
                },

                6 => {
                    score -= PAWN_SCORE[MIRROR_SCORE[square]];

                    let doubled_pawns = count_bits(current_bb & FILE_MASKS[square]);
                    if doubled_pawns != 1 {
                        score -= (doubled_pawns as i32) * DOUBLE_PAWN_PENALTY;
                    }

                    if current_bb & ISOLATED_MASKS[square] == 0 {
                        score -= ISOLATED_PAWN_PENALTY;
                    }

                    if board.bitboards[0] & BLACK_PASSED_MASKS[square] == 0 {
                        score -= PASSED_PAWN_BONUS[7-GET_RANK[square]];
                    }
                },
                7 => score -= KNIGHT_SCORE[MIRROR_SCORE[square]],
                8 => {
                    score -= BISHOP_SCORE[MIRROR_SCORE[square]];

                    score -= count_bits(get_bishop_attacks(square, board.occupancies[2])) as i32;
                },
                9 => {
                    score -= ROOK_SCORE[MIRROR_SCORE[square]];

                    if board.bitboards[6] & FILE_MASKS[square] == 0 {
                        score -= SEMI_OPEN_FILE_SCORE;

                        if board.bitboards[0] & FILE_MASKS[square] == 0 {
                            score -= OPEN_FILE_SCORE;
                        }
                    }
                },
                10 => {
                    score -= count_bits(get_queen_attacks(square, board.occupancies[2])) as i32;
                }
                11 => {
                    score -= KING_SCORE[MIRROR_SCORE[square]];

                    if board.bitboards[6] & FILE_MASKS[square] == 0 {
                        score += SEMI_OPEN_FILE_SCORE;

                        if board.bitboards[0] & FILE_MASKS[square] == 0 {
                            score += OPEN_FILE_SCORE;
                        }
                    }

                    score -= count_bits(KING_ATTACKS[square] & board.occupancies[2]) as i32 * KING_SHIELD_BONUS;
                },

                _ => ()
            }

            pop_bit!(bb, square);
        }
    }

    if board.side==Side::White {score} else {-score}
}


