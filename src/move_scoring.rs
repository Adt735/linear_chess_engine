

// most valuable victim & less valuable attacker

/*
                          
    (Victims) Pawn Knight Bishop   Rook  Queen   King
  (Attackers)
        Pawn   105    205    305    405    505    605
      Knight   104    204    304    404    504    604
      Bishop   103    203    303    403    503    603
        Rook   102    202    302    402    502    602
       Queen   101    201    301    401    501    601
        King   100    200    300    400    500    600

*/

use crate::{Board, Side, get_bit, pop_bit, get_move_capture, get_move_piece, get_move_target, moves::{Moves, move_str}, search::{PLY, MAX_PLY, FOLLOW_PV, PV_TABLE, SCORE_PV}};

// MVV LVA [attacker][victim]
static MVV_LVA:[[i32;12];12] = [
   [105, 205, 305, 405, 505, 605,  105, 205, 305, 405, 505, 605],
   [104, 204, 304, 404, 504, 604,  104, 204, 304, 404, 504, 604],
   [103, 203, 303, 403, 503, 603,  103, 203, 303, 403, 503, 603],
   [102, 202, 302, 402, 502, 602,  102, 202, 302, 402, 502, 602],
   [101, 201, 301, 401, 501, 601,  101, 201, 301, 401, 501, 601],
   [100, 200, 300, 400, 500, 600,  100, 200, 300, 400, 500, 600],

   [105, 205, 305, 405, 505, 605,  105, 205, 305, 405, 505, 605],
   [104, 204, 304, 404, 504, 604,  104, 204, 304, 404, 504, 604],
   [103, 203, 303, 403, 503, 603,  103, 203, 303, 403, 503, 603],
   [102, 202, 302, 402, 502, 602,  102, 202, 302, 402, 502, 602],
   [101, 201, 301, 401, 501, 601,  101, 201, 301, 401, 501, 601],
   [100, 200, 300, 400, 500, 600,  100, 200, 300, 400, 500, 600]
];

pub static mut KILLER_MOVES:[[usize;MAX_PLY];2] = [[0;MAX_PLY];2];
pub static mut HISTORY_MOVES:[[i32;64];12] = [[0;64];12];


pub fn sort_moves(moves:&mut Moves, board:&Board) {
    // let mut move_scores:Vec<i32> = Vec::with_capacity(moves.count);
    

    // for c in 0..moves.count {
    //     move_scores.push(score_move(moves.moves[c], board));
    // }

    // move_scores.sort();
    unsafe {
        moves.moves.sort_by_key(|move_| score_move(*move_, board));
        moves.moves.reverse();
        SCORE_PV = false;
    }
}
/*
    1. PV move
    2. Captures in MVV/LVA
    3. 1st killer move
    4. 2st killer move
    5. History moves
    6. Unsorted moves
*/
pub unsafe fn score_move(move_:usize, board:&Board) -> i32 {
    if SCORE_PV {
        // Make sure following the principal variation
        if PV_TABLE[0][PLY] == move_ {
            // SCORE_PV = false;
            return 20000
        }
    }

    if get_move_capture!(move_) {
        let mut target_piece = 0;
        let target_square = get_move_target!(move_);
        let offset = if board.side == Side::White {6} else {0};
        for bb_piece in (0+offset)..(6+offset) {
            if get_bit!(board.bitboards[bb_piece], target_square) != 0 {
                target_piece = bb_piece;
                break;
            }
        }

        return MVV_LVA[get_move_piece!(move_)][target_piece] + 10000
    } else {
        //Score 1st killer move
        if KILLER_MOVES[0][PLY] == move_ {
            return 9000
        }

        //Score 2nd killer move
        else if KILLER_MOVES[1][PLY] == move_ {
            return 8000
        }

        //Score histoy move
        else {
            return HISTORY_MOVES[get_move_piece!(move_)][get_move_target!(move_)]
        }
    }
}

pub unsafe fn enbale_pv_scoring(moves:&mut Moves) {
    FOLLOW_PV = false;

    for c in 0..moves.count {
        if PV_TABLE[0][PLY] == moves.moves[c] {
            SCORE_PV = true;
            FOLLOW_PV = true;
            return;
        }
    }
}

#[allow(dead_code)]
pub unsafe fn print_move_scores(moves:Moves, board:&Board) {
    for c in 0..moves.count {
        println!("\tMove: {}   score: {}", move_str(moves.moves[c]), score_move(moves.moves[c], board));
    }
}
