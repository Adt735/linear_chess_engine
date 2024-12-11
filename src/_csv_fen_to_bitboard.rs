use crate::{bitboard::{count_bits, Board, CastlingSide}, get_bit, move_gen::generate_moves, moves::move_str, uci::parse_move, START_POSITION, TRICKY_POSITION};
use rand::prelude::*;
use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;


#[derive(Debug, Deserialize)]
struct InputRow {
    fen: String,
    evaluation: f64,
}


pub fn convert_board_to_csv(board: &Board) -> Vec<u8> {
    let mut result = Vec::with_capacity(785);
    for bitboard in board.bitboards {
        result.extend(&convert_bitboard_to_csv(bitboard));
    }
    for bitboard in board.bitboards {
        result.push(count_bits(bitboard) as u8);
    }
    // result.push(if board.castle & CastlingSide::WK as u8 != 0 {1} else {0});
    // result.push(if board.castle & CastlingSide::WQ as u8 != 0 {1} else {0});
    // result.push(if board.castle & CastlingSide::BK as u8 != 0 {1} else {0});
    // result.push(if board.castle & CastlingSide::BQ as u8 != 0 {1} else {0});
    // result.push(board.side as u8);
    result
}

fn convert_bitboard_to_csv(bitboard:u64) -> Vec<u8> {
    let mut result = Vec::with_capacity(64);
    for square in 0..64 {
        result.push(if get_bit!(bitboard,square)!=0 {1} else {0})
    }
    result
}


pub fn process_csv(input_file: &str, output_file: &str) -> Result<(), Box<dyn Error>> {
    // Open the input CSV file for reading
    let mut reader = ReaderBuilder::new()
        .has_headers(true) // Expect the first row to have column headers
        .from_path(input_file)?;

    // Open the output CSV file for writing
    let mut writer = WriterBuilder::new()
        .has_headers(false)
        .from_path(output_file)?;

    // Write the header row for the output file
    // writer.write_record(&["FEN", "Evaluation Adjusted"])?;

    // Process each record in the input file
    for result in reader.deserialize() {
        let record: InputRow = result?;
        
        // Generate the 773-length feature vector for the FEN
        let board = Board::new_from_fen(&record.fen);
        let feature_vector = convert_board_to_csv(&board);

        // Combine the vector with the evaluation value
        let mut output_row: Vec<String> = feature_vector.iter().map(|v| v.to_string()).collect();
        output_row.push(record.evaluation.to_string());

        // Write the row to the output file
        writer.write_record(output_row)?;
    }

    println!("CSV processing complete. Output saved to {}", output_file);
    Ok(())
}


pub unsafe fn play_random_game(max_turns: usize) {
    let mut board = Board::new_from_fen(START_POSITION);
    let mut rng = rand::thread_rng();
    let mut command = String::from("position startpos moves ");

    for _ in 0..max_turns {
        let moves = generate_moves(&board);
        if moves.count == 0 { println!("Finished"); return; }
        let idx = rng.gen_range(0..moves.count);
        if !board.make_move(moves.moves[idx], false) {
            continue;
        }
        command += &format!("{} ", move_str(moves.moves[idx]));
        println!("{}", command);
    }
}
