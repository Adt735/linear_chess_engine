use crate::{random_numbers::{STATE, get_random_u64_number}, bitboard::{Board, get_ls1b_index}, get_bit, pop_bit, Side};

pub static mut PIECE_KEYS:[[u64;64];12] = [[0;64];12];
pub static mut ENPASSANT_KEYS:[u64;64] = [0;64];
pub static mut CASTLE_KEYS:[u64;16] = [0;16];
pub static mut SIDE_KEY:u64 = 0;

pub unsafe fn init_random_hash_keys() {
    STATE = 1804289383;

    for piece in 0..12 {
        for square in 0..64 {
            // Init random piece keys
            PIECE_KEYS[piece][square] = get_random_u64_number();
        }
    }

    for square in 0..64 {
        ENPASSANT_KEYS[square] = get_random_u64_number();
    }
    
    for index in 0..16 {
        CASTLE_KEYS[index] = get_random_u64_number();
    }

    SIDE_KEY = get_random_u64_number();
}

pub unsafe fn generate_hash_key(board:&Board) -> u64 {
    let mut final_key:u64 = 0;
    let mut bb:u64;

    for piece in 0..12 {
        bb = board.bitboards[piece];

        while bb != 0 {
            let square = get_ls1b_index(bb);

            final_key ^= PIECE_KEYS[piece][square];

            pop_bit!(bb, square);
        }
    }

    if let Some(square) = board.en_passant {
        final_key ^= ENPASSANT_KEYS[square];
    }

    final_key ^= CASTLE_KEYS[board.castle as usize];

    if board.side == Side::Black {
        final_key ^= SIDE_KEY
    }

    final_key
}
