use std::time::Instant;

use crate::bitboard::Board;
use crate::move_gen::generate_moves;
use crate::moves;
use crate::uci::duration_as_ms;

static mut NODES:u64 = 0;

pub unsafe fn perft_driver(board:&mut Board, depth:isize) {
    if depth == 0 {
        NODES += 1;
        return
    }

    let moves = generate_moves(&board);

    for c in 0..moves.count {
        let new_board = board.clone();
        if !board.make_move(moves.moves[c], false) {
            continue;
        }
        perft_driver(board, depth-1);
        *board = new_board;
    }
}

pub unsafe fn perft_test(board:&mut Board, depth:isize) {
    println!("\tPerformance test");

    let start = Instant::now();
    let moves = generate_moves(&board);
    let mut commulative_nodes:u64;
    let mut old_nodes:u64;

    for c in 0..moves.count {
        let new_board = board.clone();
        if !board.make_move(moves.moves[c], false) {
            continue;
        }
        commulative_nodes = NODES;
        perft_driver(board, depth-1);
        old_nodes = NODES - commulative_nodes;
        *board = new_board;

        println!("\tMove: {}   Nodes: {}", moves::move_str(moves.moves[c]), old_nodes);
    }

    let since_the_epoch = start.elapsed();
    println!("\n\n\tDepth: {}", depth);
    println!("\tNodes: {}", NODES);
    println!("\tTime: {}", duration_as_ms(since_the_epoch));
}