use crate::{eval::evaluate, bitboard::{Board, get_ls1b_index}, move_gen::{generate_moves, is_square_attacked}, Side, move_scoring::{sort_moves, KILLER_MOVES, HISTORY_MOVES, enbale_pv_scoring}, get_move_piece, get_move_target, get_move_capture, get_move_promoted, uci::{communicate, STOPPED}, transposition::{hash_flag, read_hash_entry, NO_HASH_ENTRY, write_hash_entry}, hashing::{ENPASSANT_KEYS, SIDE_KEY}};

pub static mut NODES:u32 = 0;
pub static mut PLY:usize = 0;
pub const MAX_PLY:usize = 64;

pub const INFINITY:i32 = 50000;
pub const MATE_VALUE:i32 = 49000;
pub const MATE_SCORE:i32 = 48000;

pub static mut PV_LENGTH:[usize;MAX_PLY] = [0;MAX_PLY];
pub static mut PV_TABLE:[[usize;MAX_PLY];MAX_PLY] = [[0;MAX_PLY];MAX_PLY];

pub static mut FOLLOW_PV:bool = false;
pub static mut SCORE_PV:bool = false;

// LMR
const FULL_DEPTH_MOVES:u32 = 4;
const REUCTION_LIMIT:i32 = 3;

pub unsafe fn negamax(board:&mut Board, mut depth:i32, mut alpha:i32, beta:i32) -> i32 {
    if NODES & 65535 == 0 {
        // Listen to GUI
        communicate();
    }

    let mut score:i32;

    if PLY!=0 && is_repetition(board) {
        return 0
    }

    let pv_node = beta - alpha > 1;

    // Read hash entry if not in root and move is not a PV node
    if { score = read_hash_entry(board, depth, alpha, beta); score!=NO_HASH_ENTRY && PLY!=0 && !pv_node} {
        return score
    }

    // Hash flag
    let mut hashf = hash_flag::Alpha;

    // Init PV
    PV_LENGTH[PLY] = PLY;

    if depth == 0 {
        // Run Quiescence search
        return quiescence(board, alpha, beta)
    }

    if PLY > MAX_PLY-1 {
        return evaluate(board)
    }

    NODES += 1;

    let in_check = is_square_attacked(
        board, 
        if board.side==Side::White {get_ls1b_index(board.bitboards[5])} else {get_ls1b_index(board.bitboards[11])}, 
        board.side!=Side::White
    );
    if in_check { depth += 1 }

    // LMR
    let mut moves_searched:u32 = 0;
    // Temp vars
    let mut n_legal_moves:usize = 0;

    // NULL move prunning
    if !(depth < 3) && !in_check && PLY!=0 {
        let current_board = board.clone();
        PLY += 1;

        // board.repetition_index += 1;
        // board.repetition_table[board.repetition_index] = board.hash_key;

        // Hashing null move
        if let Some(square) = board.en_passant {
            board.hash_key ^= ENPASSANT_KEYS[square];
        }
        board.hash_key ^= SIDE_KEY;

        // Swicth sides, giving opponent an extra move
        board.en_passant = None;
        board.side = if board.side==Side::White {Side::Black} else {Side::White};

        // Search moves with reduce depth
        score = -negamax(board, depth-3, -beta, -beta + 1);

        board.take_back(&current_board);
        PLY -= 1;

        // Stopped by GUI
        if STOPPED {
            return 0
        }

        if !(score < beta) {
            // Node fails high
            return beta
        }
    }

    let mut moves = generate_moves(board);

    if FOLLOW_PV {
        enbale_pv_scoring(&mut moves);
    }

    sort_moves(&mut moves, board);
    
    for c in 0..moves.count {
        let previous_board = board.clone();
        PLY += 1;

        // board.repetition_index += 1;
        // board.repetition_table[board.repetition_index] = board.hash_key;
        
        if !board.make_move(moves.moves[c], false) {
            PLY -= 1;
            board.repetition_index -= 1;
            continue;
        }

        // Current move score (static evaluation)
        let mut score: i32;

        // Normal alpha-beta search
        if moves_searched == 0 {
            score = -negamax(board, depth-1, -beta, -alpha);
        } else {
            // Condition to consider LMR
            if !(moves_searched < FULL_DEPTH_MOVES) && !(depth < REUCTION_LIMIT) 
                    && !in_check && !get_move_capture!(moves.moves[c]) && get_move_promoted!(moves.moves[c]) > 11 {
                score = -negamax(board, depth-2, -alpha - 1, -alpha);
            } else {
                // Hack to ensure full-depth search is done
                score = alpha + 1;
            }

            // Research at normal depth (PV search)
            if score > alpha {
                /* 
                    When a move with a score between alpha and beta, the rest are searched with de goal
                    of proving they are all bad
                */ 
                score = -negamax(board, depth-1, -alpha - 1, -alpha);

                /*
                    If the algorism finds out it was wrong (one of the subsequent is better than the first PV move),
                    it has to search again in the normal alpha-beta manner
                */
                if score > alpha && score < beta {
                    score = -negamax(board, depth-1, -beta, -alpha);
                }
            }
        }
            


        // Update vars
        n_legal_moves += 1;
        PLY -= 1;
        board.repetition_index -= 1;
        board.take_back(&previous_board);
        
        moves_searched += 1;

        // Stopped by GUI
        if STOPPED {
            return 0
        }

        // Found a better move (PV node or move)
        if score > alpha {
            // Switch flags
            hashf = hash_flag::Exact;

            if !get_move_capture!(moves.moves[c]) {
                HISTORY_MOVES[get_move_piece!(moves.moves[c])][get_move_target!(moves.moves[c])] += depth; //OVERFLOW??????????????????
            }

            alpha = score;

            // Write PV move
            PV_TABLE[PLY][PLY] = moves.moves[c];
            for i in (PLY+1)..(PV_LENGTH[PLY+1]) {
                PV_TABLE[PLY][i] = PV_TABLE[PLY+1][i];
            }
            PV_LENGTH[PLY] = PV_LENGTH[PLY + 1];

            if !(score < beta) {
                // Store hash entry
                write_hash_entry(board, beta, depth, hash_flag::Beta);
    
                if !get_move_capture!(moves.moves[c]) {
                    //Store killer moves
                    KILLER_MOVES[1][PLY] = KILLER_MOVES[0][PLY];
                    KILLER_MOVES[0][PLY] = moves.moves[c];
                }
    
                // Node fails high
                return beta
            }
        }
    }

    // Checking for checkmate
    if n_legal_moves == 0 {
        if in_check {
            return -MATE_VALUE + (PLY as i32) // Finding the
        } else {
            return 0
        }
    }

    // Store hash entry
    write_hash_entry(board, alpha, depth, hashf);

    // Node fails low
    alpha
}

pub fn is_repetition(board:&Board) -> bool {
    for index in 0..(board.repetition_index+100) {
        if board.repetition_table[index] == board.hash_key { return true }
    }
    false
}

// Search for next captures
pub unsafe fn quiescence(board:&mut Board, mut alpha:i32, beta:i32) -> i32 {
    if NODES & 65535 == 0 {
        // Listen to GUI
        communicate();
    }

    NODES += 1;

    if PLY > MAX_PLY-1 {
        return evaluate(board)
    }

    let evaluation = evaluate(board);
    if !(evaluation < beta) {
        // Node fails high
        return beta
    }

    // Found a better move (PV node or move)
    if evaluation > alpha {
        alpha = evaluation;
    }

    let mut moves = generate_moves(board);
    sort_moves(&mut moves, board);
    
    for c in 0..moves.count {
        let previous_board = board.clone();
        PLY += 1;

        // board.repetition_index += 1;
        // board.repetition_table[board.repetition_index] = board.hash_key;

        if !board.make_move(moves.moves[c], true) {
            PLY -= 1;
            board.repetition_index -= 1;
            continue;
        }

        let score = -quiescence(board, -beta, -alpha);

        // Update vars
        PLY -= 1;
        board.repetition_index -= 1;
        board.take_back(&previous_board);
        

        // Stopped by GUI
        if STOPPED {
            return 0
        }

        // Found a better move (PV node or move)
        if score > alpha {
            alpha = score;

            if !(score < beta) {
                // Node fails high
                return beta
            }
        }
    }

    alpha
}
