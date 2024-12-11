use std::io::{self, Write};
use std::time::Duration;
use std::time::Instant;

use crate::eval::evaluate;
use crate::move_scoring::{HISTORY_MOVES, KILLER_MOVES};
use crate::moves::move_str;
use crate::search::{negamax, NODES, PV_TABLE, PV_LENGTH, MAX_PLY, FOLLOW_PV, SCORE_PV, INFINITY, MATE_SCORE, MATE_VALUE};
use crate::transposition::{HASH_TABLE, HASH_SIZE, tt};
use crate::{get_move_source, get_move_target, get_move_promoted, START_POSITION, Side};
use crate::bitboard::{Board, ASCII_PIECES};
use crate::move_gen::generate_moves;


// TIME CONTROL VARS
static mut QUIT:bool = false; // Exit from engine
static mut TIMESET:bool = false; // Control availability
pub static mut STOPPED:bool = false; // Time's up
static mut MOVES_TO_GO:i32 = 30;
static mut MOVE_TIME:i32 = -1;
static mut TIME:i32 = -1;
static mut INC:i32 = 0;
static mut START_TIME:Option<Instant> = None;
static mut STOP_TIME:Option<Duration> = None;


// Parse move string input from the GUI (e7e8q)
pub unsafe fn parse_move(board:&Board, move_string:&str) -> usize {
    let moves = generate_moves(board);
    
    let source_square = move_string.chars().nth(0).unwrap() as usize - 'a' as usize
                                + (8 - (move_string.chars().nth(1).unwrap() as usize - '0' as usize)) * 8; 
    let target_square = move_string.chars().nth(2).unwrap() as usize - 'a' as usize
                                + (8 - (move_string.chars().nth(3).unwrap() as usize - '0' as usize)) * 8; 

    for c in 0..moves.count {
        let move_ = moves.moves[c];

        if get_move_source!(move_) == source_square && get_move_target!(move_) == target_square {
            let promoted = get_move_promoted!(move_);
            if promoted < 12 {
                let promoted = if promoted<6 {promoted+6} else {promoted};
                if move_string.contains(ASCII_PIECES[promoted]) {
                    return move_
                }
            } else {
                return move_
            }
        }
    }

    0
}

pub fn parse_position(command:&str) -> Board {
    let complete_info:Vec<&str> = command.splitn(2, "moves").collect();
    let info:Vec<&str> = complete_info[0].splitn(3, ' ').collect();
    
    let mut board = Board::new();

    match info[..] {
        ["position", "startpos", ..] => board=Board::new_from_fen(START_POSITION),
        ["position", "fen", ""] => board=Board::new_from_fen(START_POSITION),
        ["position", "fen", fen] => board=Board::new_from_fen(fen),
        _ => (),
    }

    let mut index = 0;
    unsafe {
        if complete_info.len() == 2 {
            let moves:Vec<&str> = complete_info[1].split_whitespace().collect();
            for move_ in moves {
                board.make_move(parse_move(&board, move_), false);
                board.repetition_table[index] = board.hash_key;
                index += 1;
                // println!("{}", index);
            }
        }
    }
    
    board.repetition_index = index;
    // println!("rEPETITION_INDEX: {}", board.repetition_index);
    board
}

// Parse go command (go depth 6)
pub unsafe fn parse_go(board:&mut Board, command: &str) {
    let info:Vec<&str> = command.split_whitespace().collect();
    let mut depth:i32 = -1;
    let mut iter = info.iter();

    //Flags
    QUIT = false;
    TIMESET = false;
    MOVES_TO_GO = 30;
    MOVE_TIME = -1;
    TIME = -1;
    INC = 0;
    START_TIME = None;
    STOP_TIME = None;

    while let Some(&token) = iter.next() {
        match token {
            "depth" => {
                if let Some(&value) = iter.next() {
                    if let Ok(depth_) = value.parse::<i32>() {
                        depth = depth_;
                    }
                }
            }
            "binc" => if board.side==Side::Black {
                if let Some(&value) = iter.next() {
                    if let Ok(inc) = value.parse::<i32>() {
                        INC = inc;
                    }
                }
            } 
            "winc" => if board.side==Side::White {
                if let Some(&value) = iter.next() {
                    if let Ok(inc) = value.parse::<i32>() {
                        INC = inc;
                    }
                }
            } 
            "btime" => if board.side==Side::Black {
                if let Some(&value) = iter.next() {
                    if let Ok(inc) = value.parse::<i32>() {
                        TIME = inc;
                    }
                }
            } 
            "wtime" => if board.side==Side::White {
                if let Some(&value) = iter.next() {
                    if let Ok(inc) = value.parse::<i32>() {
                        TIME = inc;
                    }
                }
            } 
            "movestogo" => {
                if let Some(&value) = iter.next() {
                    if let Ok(inc) = value.parse::<i32>() {
                        MOVES_TO_GO = inc;
                    }
                }
            }
            "movetime" => {
                if let Some(&value) = iter.next() {
                    if let Ok(inc) = value.parse::<i32>() {
                        MOVE_TIME = inc;
                    }
                }
            }
            "infinite" => {
                TIMESET = false;
            }
            _ => {}
        }
    }

    if MOVE_TIME != -1 {
        TIME = MOVE_TIME;
        MOVES_TO_GO = 1;
    }

    START_TIME = Some(Instant::now());

    if TIME != -1 {
        TIMESET = true;
        TIME /= MOVES_TO_GO;
        TIME -= 100;
        STOP_TIME = Some(Duration::from_millis((TIME + INC) as u64));
    }

    if depth == -1 {
        depth = 6;
    }

    search_position(board, depth);
}

pub unsafe fn parse_go_(board:&mut Board, command:&str) {
    // Global vars
    TIMESET = false;


    let info:Vec<&str> = command.split_whitespace().collect();
    let depth:i32;

    match info[..] {
        ["go", "depth", depth_] => depth = depth_.parse().unwrap(),
        _ => depth=4,
    }

    search_position(board, depth);
}

pub fn search_position(board:&mut Board, depth:i32) {
    unsafe {
        // Flags
        STOPPED = false;
        FOLLOW_PV = false;
        SCORE_PV = false;

        // Clear helper data
        NODES = 0;
        KILLER_MOVES = [[0;MAX_PLY];2];
        HISTORY_MOVES = [[0;64];12];
        PV_TABLE = [[0;MAX_PLY];MAX_PLY];
        PV_LENGTH = [0;MAX_PLY];

        let mut alpha = -50000;
        let mut beta = 50000;
        let mut current_depth = 1;
        // Iterativa deepening
        while current_depth < depth+1 {
            // Stopped by GUI
            if STOPPED {
                break;
            }

            FOLLOW_PV = true;

            let score = negamax(board, current_depth, alpha, beta);

            // // Aspiration window
            // // If we fall outside the window, try again with full-width window (same depth)
            if !(score > alpha) || !(score<beta) {
                alpha = -INFINITY;
                beta = INFINITY;
                continue;
            }

            alpha = score - 50;
            beta = score + 50;

            // Print info for UCI
            if score > -MATE_VALUE && score < -MATE_SCORE {
                print!("info score mate {} depth {} nodes {} time {} ", -(score+MATE_VALUE)/2-1, current_depth, NODES, duration_as_ms(START_TIME.unwrap().elapsed()))
            } else if score > MATE_SCORE && score < MATE_VALUE {
                print!("info score mate {} depth {} nodes {} time {} ", (MATE_VALUE-score)/2-1, current_depth, NODES, duration_as_ms(START_TIME.unwrap().elapsed()))
            } else {
                match START_TIME {
                    Some(value) => print!("info score cp {} depth {} nodes {} time {} ", score, current_depth, NODES, duration_as_ms(START_TIME.unwrap().elapsed())),
                    None => print!("info score cp {} depth {} nodes {} ", score, current_depth, NODES),
                }
                
            }

            print!("pv ");
            for i in 0..PV_LENGTH[0] {
                print!("{} ", move_str(PV_TABLE[0][i]));
            }
            println!();

            current_depth += 1;
        }

        print!("bestmove ");
        println!("{}", move_str(PV_TABLE[0][0]));
    }
}


pub unsafe fn uci_loop() {
    let mut input:String = String::new();
    let mut board = Board::new();

    // Enginge info
    println!("id name Optimus");
    println!("id author Simply's Adt");
    println!("uciok");

    loop {
        io::stdin().read_line(&mut input).unwrap();
        let mut input_str = input.trim();
        let input_separated:Vec<&str> = input.split_whitespace().collect();

        match input_str {
            x if x.contains("isready") => {
                STOPPED=true;
                println!("readyok");
            },
            x if x.contains("position") => { 
                HASH_TABLE = [tt::new();HASH_SIZE];
                board = parse_position(input_str)
            },
            x if x.contains("ucinewgame") => { 
                STOPPED = true;
                HASH_TABLE = [tt::new();HASH_SIZE];
                board = parse_position("position startpos"); 
            },
            x if x.contains("go") => parse_go(&mut board, input_str),
            x if x.contains("quit") => {QUIT=true; STOPPED=true; break;},
            x if x.contains("new") => {
                STOPPED = true;
                HASH_TABLE = [tt::new();HASH_SIZE];
                board = parse_position("position startpos"); 
            },
            x if x.contains("uci") => {
                // Enginge info
                println!("id name Optimus");
                println!("id author Simply's Adt");
                println!("uciok");
            },
            x if x.contains("eval") => {
                println!("Eval: {}", evaluate(&board));
            }
            _ => (),
        }

        io::stdout().flush().unwrap();
        input.clear();
    }
}


// Bridge function to interact between search and GUI input
pub unsafe fn communicate() {
    if TIMESET && START_TIME.unwrap().elapsed() > STOP_TIME.unwrap() {
        STOPPED = true;
    }
}





pub fn duration_as_ms(elapsed:Duration) -> u64 {
    elapsed.as_secs() * 1000 +
    elapsed.subsec_nanos() as u64 / 1_000_000
}
