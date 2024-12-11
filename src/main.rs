
use std::error::Error;
use std::io::Write;
use std::{fs::File, io::BufReader, net::TcpStream, vec};


#[allow(dead_code)]
use _csv_fen_to_bitboard::convert_board_to_csv;
use _csv_fen_to_bitboard::{play_random_game, process_csv};
use _linear_regression::{parse_csv, LinearModel};
use _neural_network::{close_connection, communicate, load, open_program, send_and_receive};
use attacks::{get_bishop_attacks, get_rook_attacks, get_queen_attacks, PAWN_ATTACKS};
use bitboard::Board;
use clap::Parser;
use clap_derive::{Parser, Subcommand};
use eval::*;
use hashing::{init_random_hash_keys, generate_hash_key};
use linfa::traits::Fit;
use linfa::Dataset;
use linfa_linear::LinearRegression;
use linfa::prelude::SingleTargetRegression;
use move_gen::generate_moves;
use move_scoring::{sort_moves, print_move_scores};
use nalgebra::DMatrix;
use nalgebra::DVector;
use ndarray::Array;
use ndarray::Array2;
use ndarray::{Array1, ArrayBase, OwnedRepr};
use perft::{perft_driver, perft_test};
use polars::{io::SerReader, prelude::{DataType, Float64Type, IndexOrder}};
use search::PV_TABLE;
use serde::Deserialize;
use serde::Serialize;
use transposition::{write_hash_entry, read_hash_entry};
// use tweak::{init_eval_constants, EngineValues, save_to_json_file};
use uci::{parse_move, parse_go, parse_position, uci_loop, search_position};

use crate::{bitboard::{Pieces, print_bitboard, CastlingSide, ASCII_PIECES}, moves::print_move, transposition::hash_flag};



mod bitboard;
mod attacks;
mod move_gen;
mod moves;
mod eval;
mod search;
mod move_scoring;
mod transposition;
mod uci;
mod hashing;
mod random_numbers;
mod perft;


#[allow(non_camel_case_types)]
#[allow(dead_code)]
enum Square {
    a8, b8, c8, d8, e8, f8, g8, h8,
    a7, b7, c7, d7, e7, f7, g7, h7,
    a6, b6, c6, d6, e6, f6, g6, h6,
    a5, b5, c5, d5, e5, f5, g5, h5,
    a4, b4, c4, d4, e4, f4, g4, h4,
    a3, b3, c3, d3, e3, f3, g3, h3,
    a2, b2, c2, d2, e2, f2, g2, h2,
    a1, b1, c1, d1, e1, f1, g1, h1
}

pub enum Color { White, Black, Both }

#[derive(PartialEq, Clone, Copy)]
pub enum Side { White, Black, None }

impl Side {
    pub fn to_string(&self) -> &str {
        match self {
            Side::White => "White",
            Side::Black => "Black",
            Side::None => "None"
        }
    }
}

#[allow(dead_code)]
const SQUARE_TO_COORDINATES:[&str;64] = [
    "a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8",
    "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7",
    "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6",
    "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5",
    "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4",
    "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3",
    "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2",
    "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1"
];

const EMPTY_BOARD:&str = "8/8/8/8/8/8/8/8 w - - ";
const START_POSITION:&str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 ";
const TRICKY_POSITION:&str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1 ";
const KILLER_POSITION:&str = "rnbqkb1r/pp1p1pPp/8/2p1pP2/1P1P4/3P3P/P1P1P3/RNBQKBNR w KQkq - 0 1";
const CMK_POSITION:&str = "r2q1rk1/ppp2ppp/2n1bn2/2b1p3/3pP3/3P1NPP/PPP1NPB1/R1BQ1RK1 b - - 0 9 ";
const REPETITIONS:&str = "2r3k1/R7/8/1R6/8/8/P4KPP/8 w - - 0 40 ";


fn init_all_vars() {
    unsafe {
        attacks::init_leapers_attacks();
        attacks::init_sliders_attacks(true);
        attacks::init_sliders_attacks(false);   
        // random_numbers::init_magic_numbers();
        init_random_hash_keys();
        init_evaluation_masks();

        // init_eval_constants("./data.json");
        // let val = EngineValues { material_score: [100,300,300,500,900,20000] };
        // MATERIAL_SCORE[..6].copy_from_slice(&val.material_score);
        // for i in 6..12 {
        //     MATERIAL_SCORE[i] = -val.material_score[i-6];
        // }
        // if let Err(e) = save_to_json_file(&val, "./data.json") {
        //     eprintln!("Error writing to file: {}", e);
        // }
    }
    
}

mod _csv_fen_to_bitboard;
mod _neural_network;
mod _linear_regression;


/// CLI application to process commands
#[derive(Parser)]
#[command(name = "MyApp")]
#[command(about = "A CLI tool with multiple commands", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}


#[derive(Subcommand)]
enum Commands {
    /// Process a CSV file
    ProcessCsv {
        /// Input file
        #[arg(short, long)]
        input: String,

        /// Output file
        #[arg(short, long)]
        output: String,
    },
    /// Run the UCI command
    Uci {
        #[arg(short, long)]
        input: String,
    },
    /// Train Model
    LinearRegression {
        /// Input file
        #[arg(short, long)]
        input: String,
    }
}

pub const DATASET_PATH: &str = "C:/Users/adtro/Uni/MatCAD/3r/APC/kaggle/final_data.csv";
fn main() {
    unsafe {
        let cli = Cli::parse();

        match cli.command {
            Commands::ProcessCsv { input, output } => {
                process_csv(
                    &input, // "C:/Users/adtro/Uni/MatCAD/3r/APC/kaggle/final/dataset_.csv"
                    &output, // "C:/Users/adtro/Uni/MatCAD/3r/APC/kaggle/final/final_data.csv"
                ).unwrap();
            }
            Commands::Uci { input } => {
                let json_data = std::fs::read_to_string(&input).expect("Failed to read JSON file");
                LINEAR_COEFF = serde_json::from_str(&json_data).expect("Failed to parse JSON");
                init_all_vars();
                uci_loop();
            }
            Commands::LinearRegression { input } => {
                let inputs = vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,1,1,1,0,1,1,1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,1,1,1,1,1,1,1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1,1,1,1];
                let inputs: Vec<f64> = inputs.into_iter().map(|v| v as f64).collect();

                // Load the model from JSON
                let json_data = std::fs::read_to_string(&input).expect("Failed to read JSON file");
                let model: _linear_regression::LinearModel = serde_json::from_str(&json_data).expect("Failed to parse JSON");

                // Example input data
                let prediction = _linear_regression::predict(&model, &inputs);

                println!("Prediction: {}", prediction);
            }
        }

        // -------------------- Subprocess -------------------------------------------
        // let mut inputs = vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,1,1,1,0,1,1,1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,1,1,1,1,1,1,1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1,1,1,1];
        // // println!("{}", inputs.len());
        // let (mut child, mut child_stdin, mut reader) = open_program();
        // send_and_receive(&mut child, &mut child_stdin, &mut reader, inputs);
        // close_connection(child_stdin, child);
        // print!("sgdv\n")

        // -------------------- Flask ------------------------------------------------
        // let server_address = "127.0.0.1:5000";
        // let mut stream = TcpStream::connect(server_address).unwrap();
        // println!("{:?}", communicate(&mut stream, &server_address, &inputs));
        // inputs[0] = 1;
        // let mut stream = TcpStream::connect(server_address).unwrap();
        // println!("{:?}", communicate(&mut stream, &server_address, &inputs));

        // ------------------- Json model -------------------------------------------
        // let mut board = Board::new_from_fen(START_POSITION);
        // board.make_move(parse_move(&board, "d2d4"), false);
        // let inputs: Vec<f64> = convert_board_to_csv(&board).into_iter().map(|v| v as f64).collect();
        // // let layers = load();
        // // Example input
        // let input = Array1::from(inputs); // Adjust based on your input dimensions

        // // // Perform prediction
        // let output: ndarray::ArrayBase<ndarray::OwnedRepr<f64>, ndarray::Dim<[usize; 1]>> = predict(input, &NN_MODEL);
        // println!("Prediction: {:?}", output);

        // let mut board = Board::new_from_fen("r3k2r/pPppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1 ");
        // board.print();
        // let move_ = parse_move(&board, "e1g1");
        // board.make_move(move_, false);
        // board.print();

        // ---------------- Conevrt to linear model ------------------------------------
        // let mut all_scores: Vec<f64> = Vec::with_capacity(780);
        // all_scores.extend(PAWN_SCORE.iter().map(|&x| x as f64 / 100.0));
        // all_scores.extend(KNIGHT_SCORE.iter().map(|&x| x as f64 / 100.0));
        // all_scores.extend(BISHOP_SCORE.iter().map(|&x| x as f64 / 100.0));
        // all_scores.extend(ROOK_SCORE.iter().map(|&x| x as f64 / 100.0));
        // all_scores.extend(KING_SCORE.iter().map(|&x| x as f64 / 100.0));
        // all_scores.extend(MATERIAL_SCORE.iter().map(|&x| x as f64 / 100.0));

        // // Create the JSON object
        // let coefficients = LinearModel {
        //     coefficients: all_scores,
        //     intercept: 0.0,
        // };

        // // Serialize to JSON and write to file
        // let json_data = serde_json::to_string_pretty(&coefficients).unwrap();
        // let mut file = File::create("coefficients.json").expect("Unable to create file");
        // file.write_all(json_data.as_bytes()).expect("Unable to write data");

        // println!("JSON file 'coefficients.json' created successfully.");
    }
}
