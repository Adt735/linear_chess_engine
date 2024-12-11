use crate::{bitboard::Board, search::{MATE_SCORE, PLY}};



pub const HASH_SIZE:usize = 0x400000;
pub const NO_HASH_ENTRY:i32 = 100000;

#[derive(Copy, Clone)]
pub enum hash_flag {
    Exact, Alpha, Beta,
}

// Transposition table struct
#[derive(Copy, Clone)]
pub struct tt {
    pub hash_key:u64,
    depth:i32,
    flag:hash_flag,
    score:i32,
}

impl tt {
    pub const fn new() -> tt {
        tt {
            hash_key:0,
            depth:0,
            flag:hash_flag::Exact,
            score:0,
        }
    }
}


pub static mut HASH_TABLE:[tt;HASH_SIZE] = [tt::new();HASH_SIZE];


pub unsafe fn read_hash_entry(board:&Board, depth:i32, alpha:i32, beta:i32) -> i32 {
    let hash_entry:&tt = &HASH_TABLE[(board.hash_key as usize) % HASH_SIZE];

    // Make sure we're dealing with the same position we need
    if hash_entry.hash_key == board.hash_key {
        if !(hash_entry.depth < depth) {
            let mut score = hash_entry.score;

            // Mate score shall be independent from the actual path from root
            if score < -MATE_SCORE { score += PLY as i32 }
            else if score > MATE_SCORE { score -= PLY as i32 }

            match hash_entry.flag {
                hash_flag::Exact => return score,
                hash_flag::Alpha => if !(score > alpha) { return alpha },
                hash_flag::Beta => if !(score < beta) { return beta },
            }
        }
    }

    NO_HASH_ENTRY
}

pub unsafe fn write_hash_entry(board:&Board, mut score:i32, depth:i32, flag:hash_flag) {
    let mut hash_entry:&mut tt = &mut HASH_TABLE[(board.hash_key as usize) % HASH_SIZE];

    // Mate score shall be independent from the actual path from root
    if score < -MATE_SCORE { score -= PLY as i32 }
    else if score > MATE_SCORE { score += PLY as i32 }

    hash_entry.hash_key = board.hash_key;
    hash_entry.score = score;
    hash_entry.flag = flag;
    hash_entry.depth = depth;
}
