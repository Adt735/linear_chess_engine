use crate::{bitboard::{Board, get_ls1b_index, print_bitboard, count_bits}, Side, Square::{*, self}, pop_bit, get_bit, set_bit, attacks::{get_bishop_attacks, get_queen_attacks, KING_ATTACKS}, move_gen::generate_moves};

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

// Material
pub static mut MATERIAL_SCORE:[i32;12] = [0;12];

// PIECES
const MOBILITY_BONUS:i32 = 10;

// PAWNS
const DOUBLE_PAWN_PENALTY:i32 = 0;
const BLOCKED_PAWN_PENALTY:i32 = 0;
const ISOLATED_PAWN_PENALTY:i32 = 0;

// SLIDING PIECES
const SEMI_OPEN_FILE_SCORE:i32 = 0;
const OPEN_FILE_SCORE:i32 = 0;

// King
const KING_SHIELD_BONUS:i32 = 0;


pub unsafe fn evaluate(board:&Board) -> i32 {
    let mut score = 0;
    let mut bb:u64;
    let mut piece:usize;
    let mut square:usize;

    let mut board_copy = board.clone();
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
                    // Doubled, Isolated pawns
                    let doubled_pawns = count_bits(current_bb & FILE_MASKS[square]);
                    if doubled_pawns != 1 {
                        score -= DOUBLE_PAWN_PENALTY;
                    }

                    if current_bb & ISOLATED_MASKS[square] == 0 {
                        score -= ISOLATED_PAWN_PENALTY;
                    }
                },
                1 => {}
                2 => {},
                3 => {},
                4 => {}
                5 => {},

                6 => {
                    // Doubled, Isolated pawns
                    let doubled_pawns = count_bits(current_bb & FILE_MASKS[square]);
                    if doubled_pawns != 1 {
                        score += DOUBLE_PAWN_PENALTY;
                    }

                    if current_bb & ISOLATED_MASKS[square] == 0 {
                        score += ISOLATED_PAWN_PENALTY;
                    }
                },
                7 => {},
                8 => {},
                9 => {},
                10 => {}
                11 => {},

                _ => ()
            }

            pop_bit!(bb, square);
        }
    }

    // PAWNS Blocked
    let blocked = (board.bitboards[0] >> 8) & board.occupancies[2];
    if blocked != 0 {
        score -= count_bits(blocked) as i32 * BLOCKED_PAWN_PENALTY;
    }
    let blocked = (board.bitboards[6] << 8) & board.occupancies[2];
    if blocked != 0 {
        score += count_bits(blocked) as i32 * BLOCKED_PAWN_PENALTY;
    }

    // Mobility
    if board.side == Side::White {
        score += generate_moves(&board_copy).count as i32 * MOBILITY_BONUS;
        board_copy.side = Side::Black;
        score -= generate_moves(&board_copy).count as i32 * MOBILITY_BONUS;
    } else {
        score -= generate_moves(&board_copy).count as i32 * MOBILITY_BONUS;
        board_copy.side = Side::White;
        score += generate_moves(&board_copy).count as i32 * MOBILITY_BONUS;
    }

    if board.side==Side::White {score} else {-score}
}


