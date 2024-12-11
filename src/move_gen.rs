use crate::moves::Moves;
use crate::{Side, Square, get_bit, pop_bit, SQUARE_TO_COORDINATES, encode_move};
use crate::bitboard::{Board, Pieces, get_ls1b_index, CastlingSide};
use crate::attacks::{PAWN_ATTACKS, KNIGHT_ATTACKS, KING_ATTACKS, get_bishop_attacks, get_queen_attacks, get_rook_attacks};

#[inline(always)]
pub unsafe fn is_square_attacked(board:&Board, square:usize, is_white_turn:bool) -> bool {
    let offset = if is_white_turn {0} else {6};

    // Attacked by pawns
    if (is_white_turn) && ((PAWN_ATTACKS[1][square] & board.bitboards[0]) != 0) { return true }
    if (!is_white_turn) && ((PAWN_ATTACKS[0][square] & board.bitboards[6]) != 0) { return true }

    // Attacked by leapers pieces
    if (KNIGHT_ATTACKS[square] & board.bitboards[1+offset]) != 0 {return true }
    if (KING_ATTACKS[square] & board.bitboards[5+offset]) != 0 {return true }

    // Attacked by sliders pieces
    if (get_bishop_attacks(square, board.occupancies[2]) & board.bitboards[2+offset]) != 0 { return true }
    if (get_rook_attacks(square, board.occupancies[2]) & board.bitboards[3+offset]) != 0 { return true }
    if (get_queen_attacks(square, board.occupancies[2]) & board.bitboards[4+offset]) != 0 { return true }

    false
}

pub fn print_attacked_squares(board:&Board, is_white_turn:bool) {
    println!("\n");
    for rank in 0..8usize {
        for file in 0..8 {
            let square = rank * 8 + file;

            if file == 0 {
                print!("\t{}   ", 8-rank);
            }

            unsafe { print!(" {}", if is_square_attacked(board, square, is_white_turn) {1} else {0}) };
        }
        println!();
    }

    print!("\n\t     a b c d e f g h \n\n");
    // println!("\t\tBitboard: {}", bitboard);
}


#[inline(always)]
pub unsafe fn generate_moves(board:&Board) -> Moves {
    let mut moves = Moves::new();
    let mut source_square:usize;
    let mut target_square:usize;

    let mut bb:u64;
    let mut attacks:u64;

    for piece in 0..12 {
        bb = board.bitboards[piece];

        // Generate pawns and king castling
        match board.side {
            Side::White => {
                match piece {
                    // Pawn
                    0 => while bb != 0 {
                        source_square = get_ls1b_index(bb);
                        target_square = source_square - 8; 

                        // No need to check for negative squares (a pawn will never be in the last rank)
                        if get_bit!(board.occupancies[2], target_square) == 0 {
                            // Pawn promotion
                            if source_square < Square::a6 as usize && source_square > Square::h8 as usize {
                                moves.add_move(encode_move!(source_square, target_square, piece, Pieces::Q as usize, 0, 0, 0, 0));
                                moves.add_move(encode_move!(source_square, target_square, piece, Pieces::R as usize, 0, 0, 0, 0));
                                moves.add_move(encode_move!(source_square, target_square, piece, Pieces::B as usize, 0, 0, 0, 0));
                                moves.add_move(encode_move!(source_square, target_square, piece, Pieces::N as usize, 0, 0, 0, 0));
                            } else {
                                // Pawn push
                                moves.add_move(encode_move!(source_square, target_square, piece, 12, 0, 0, 0, 0));
                                if (source_square > Square::h3 as usize && source_square < Square::a1 as usize) && get_bit!(board.occupancies[2], target_square-8) == 0 {
                                    // Double pawn push
                                    moves.add_move(encode_move!(source_square, target_square-8, piece, 12, 0, 1, 0, 0));
                                }
                            }
                        }

                        // Pawn captures
                        unsafe { attacks = PAWN_ATTACKS[0][source_square] & board.occupancies[1]; }
                        while attacks != 0 {
                            target_square = get_ls1b_index(attacks);
                            
                            // Capture + promotion
                            if source_square < Square::a6 as usize && source_square > Square::h8 as usize {
                                moves.add_move(encode_move!(source_square, target_square, piece, Pieces::Q as usize, 1, 0, 0, 0));
                                moves.add_move(encode_move!(source_square, target_square, piece, Pieces::R as usize, 1, 0, 0, 0));
                                moves.add_move(encode_move!(source_square, target_square, piece, Pieces::B as usize, 1, 0, 0, 0));
                                moves.add_move(encode_move!(source_square, target_square, piece, Pieces::N as usize, 1, 0, 0, 0));
                            } else {
                                moves.add_move(encode_move!(source_square, target_square, piece, 12, 1, 0, 0, 0));
                            }

                            pop_bit!(attacks, target_square);
                        }

                        // Enpassant
                        unsafe {
                            match board.en_passant {
                                Some(sq) => {
                                    let en_passant_attacks = PAWN_ATTACKS[0][source_square] & (1u64 << sq);
                                    if en_passant_attacks != 0 {
                                        let target_enpassant = get_ls1b_index(en_passant_attacks);
                                        moves.add_move(encode_move!(source_square, target_enpassant, piece, 12, 1, 0, 1, 0));
                                    }
                                },
                                None => (),
                            }
                        }

                        pop_bit!(bb, source_square);
                    },
                    // King Castling
                    5 => unsafe {
                        if (board.castle & CastlingSide::WK as u8) != 0 {
                            // Squares between are empty
                            if get_bit!(board.occupancies[2], Square::f1 as usize)==0 && get_bit!(board.occupancies[2], Square::g1 as usize)==0 {
                                if !is_square_attacked(board, Square::e1 as usize, false) 
                                // && !is_square_attacked(board, Square::g1 as usize, false)
                                && !is_square_attacked(board, Square::f1 as usize, false) {
                                    moves.add_move(encode_move!(Square::e1 as usize, Square::g1 as usize, piece, 12, 0, 0, 0, 1));
                                }
                            }
                        }

                        if (board.castle & CastlingSide::WQ as u8) != 0 {
                            // Squares between are empty
                            if get_bit!(board.occupancies[2], Square::d1 as usize)==0 && get_bit!(board.occupancies[2], Square::c1 as usize)==0
                                && get_bit!(board.occupancies[2], Square::b1 as usize)==0 {
                                if !is_square_attacked(board, Square::e1 as usize, false) 
                                // && !is_square_attacked(board, Square::c1 as usize, false)
                                && !is_square_attacked(board, Square::d1 as usize, false) {
                                    moves.add_move(encode_move!(Square::e1 as usize, Square::c1 as usize, piece, 12, 0, 0, 0, 1));
                                }
                            }
                        }
                    }
                    _ => (),
                }
            },
            Side::Black => {
                match piece {
                    6 => while bb != 0 {
                        source_square = get_ls1b_index(bb);
                        target_square = source_square + 8; 

                        // Quiet pawns move
                        // No need to check for negative squares (a pawn will never be in the last rank)
                        if get_bit!(board.occupancies[2], target_square) == 0 {
                            // Pawn promotion
                            if source_square > Square::h3 as usize && source_square < Square::a1 as usize {
                                moves.add_move(encode_move!(source_square, target_square, piece, Pieces::q as usize, 0, 0, 0, 0));
                                moves.add_move(encode_move!(source_square, target_square, piece, Pieces::r as usize, 0, 0, 0, 0));
                                moves.add_move(encode_move!(source_square, target_square, piece, Pieces::b as usize, 0, 0, 0, 0));
                                moves.add_move(encode_move!(source_square, target_square, piece, Pieces::n as usize, 0, 0, 0, 0));
                            } else {
                                // Pawn push
                                moves.add_move(encode_move!(source_square, target_square, piece, 12, 0, 0, 0, 0));
                                if (source_square < Square::a6 as usize && source_square > Square::h8 as usize) && get_bit!(board.occupancies[2], target_square+8) == 0 {
                                    // Double pawn push
                                    moves.add_move(encode_move!(source_square, target_square+8, piece, 12, 0, 1, 0, 0));
                                }
                            }
                        }

                        // Pawn captures
                        unsafe { attacks = PAWN_ATTACKS[1][source_square] & board.occupancies[0]; }
                        while attacks != 0 {
                            target_square = get_ls1b_index(attacks);
                            
                            // Capture + promotion
                            if source_square > Square::h3 as usize && source_square < Square::a1 as usize {
                                moves.add_move(encode_move!(source_square, target_square, piece, Pieces::q as usize, 1, 0, 0, 0));
                                moves.add_move(encode_move!(source_square, target_square, piece, Pieces::r as usize, 1, 0, 0, 0));
                                moves.add_move(encode_move!(source_square, target_square, piece, Pieces::b as usize, 1, 0, 0, 0));
                                moves.add_move(encode_move!(source_square, target_square, piece, Pieces::n as usize, 1, 0, 0, 0));
                            } else {
                                moves.add_move(encode_move!(source_square, target_square, piece, 12, 1, 0, 0, 0));
                            }

                            pop_bit!(attacks, target_square);
                        }

                        // Enpassant
                        unsafe {
                            match board.en_passant {
                                Some(sq) => {
                                    let en_passant_attacks = PAWN_ATTACKS[1][source_square] & (1u64 << sq);
                                    if en_passant_attacks != 0 {
                                        let target_enpassant = get_ls1b_index(en_passant_attacks);
                                        moves.add_move(encode_move!(source_square, target_enpassant, piece, 12, 1, 0, 1, 0));
                                    }
                                },
                                None => (),
                            }
                        }

                        pop_bit!(bb, source_square);
                    },
                    // King Castling
                    11 => unsafe {
                        if (board.castle & CastlingSide::BK as u8) != 0 {
                            // Squares between are empty
                            if get_bit!(board.occupancies[2], Square::f8 as usize)==0 && get_bit!(board.occupancies[2], Square::g8 as usize)==0 {
                                if !is_square_attacked(board, Square::e8 as usize, true) 
                                // && !is_square_attacked(board, Square::g8 as usize, true)
                                && !is_square_attacked(board, Square::f8 as usize, true) {
                                    moves.add_move(encode_move!(Square::e8 as usize, Square::g8 as usize, piece, 12, 0, 0, 0, 1));
                                }
                            }
                        }

                        if (board.castle & CastlingSide::BQ as u8) != 0 {
                            // Squares between are empty
                            if get_bit!(board.occupancies[2], Square::d8 as usize)==0 && get_bit!(board.occupancies[2], Square::c8 as usize)==0
                                && get_bit!(board.occupancies[2], Square::b8 as usize)==0 {
                                if !is_square_attacked(board, Square::e8 as usize, true) 
                                // && !is_square_attacked(board, Square::c8 as usize, true)
                                && !is_square_attacked(board, Square::d8 as usize, true) {
                                    moves.add_move(encode_move!(Square::e8 as usize, Square::c8 as usize, piece, 12, 0, 0, 0, 1));
                                }
                            }
                        }
                    }
                    _ => (),
                }
            },
            _ => (),
        }

        let offset = if board.side == Side::White {0} else {6};
        let not_current_occupancies = if offset==0 {!board.occupancies[0]} else {!board.occupancies[1]};
        let other_occupancies = if offset==0 {board.occupancies[1]} else {board.occupancies[0]};

        if piece == 1+offset {
            while bb != 0 {
                source_square = get_ls1b_index(bb);
    
                attacks = KNIGHT_ATTACKS[source_square] & not_current_occupancies;
                while attacks != 0 {
                    target_square = get_ls1b_index(attacks);
    
                    if get_bit!(other_occupancies, target_square) == 0 {
                        moves.add_move(encode_move!(source_square, target_square, piece, 12, 0, 0, 0, 0));
                    } else {
                        moves.add_move(encode_move!(source_square, target_square, piece, 12, 1, 0, 0, 0));
                    }
    
                    pop_bit!(attacks, target_square);
                }
    
                pop_bit!(bb, source_square);
            }
        } else if piece == 2+offset {
            while bb != 0 {
                source_square = get_ls1b_index(bb);
    
                attacks = get_bishop_attacks(source_square, board.occupancies[2]) & not_current_occupancies;
                while attacks != 0 {
                    target_square = get_ls1b_index(attacks);
    
                    if get_bit!(other_occupancies, target_square) == 0 {
                        moves.add_move(encode_move!(source_square, target_square, piece, 12, 0, 0, 0, 0));
                    } else {
                        moves.add_move(encode_move!(source_square, target_square, piece, 12, 1, 0, 0, 0));
                    }
    
                    pop_bit!(attacks, target_square);
                }
    
                pop_bit!(bb, source_square);
            }
        } else if piece == 3+offset {
            while bb != 0 {
                source_square = get_ls1b_index(bb);
    
                attacks = get_rook_attacks(source_square, board.occupancies[2]) & not_current_occupancies;
                while attacks != 0 {
                    target_square = get_ls1b_index(attacks);
    
                    if get_bit!(other_occupancies, target_square) == 0 {
                        moves.add_move(encode_move!(source_square, target_square, piece, 12, 0, 0, 0, 0));
                    } else {
                        moves.add_move(encode_move!(source_square, target_square, piece, 12, 1, 0, 0, 0));
                    }
    
                    pop_bit!(attacks, target_square);
                }
    
                pop_bit!(bb, source_square);
            }
        } else if piece == 4+offset {
            while bb != 0 {
                source_square = get_ls1b_index(bb);
    
                attacks = get_queen_attacks(source_square, board.occupancies[2]) & not_current_occupancies;
                while attacks != 0 {
                    target_square = get_ls1b_index(attacks);
    
                    if get_bit!(other_occupancies, target_square) == 0 {
                        moves.add_move(encode_move!(source_square, target_square, piece, 12, 0, 0, 0, 0));
                    } else {
                        moves.add_move(encode_move!(source_square, target_square, piece, 12, 1, 0, 0, 0));
                    }
    
                    pop_bit!(attacks, target_square);
                }
    
                pop_bit!(bb, source_square);
            }
        } else if piece == 5+offset {
            while bb != 0 {
                source_square = get_ls1b_index(bb);
    
                attacks = KING_ATTACKS[source_square] & not_current_occupancies;
                while attacks != 0 {
                    target_square = get_ls1b_index(attacks);
    
                    if get_bit!(other_occupancies, target_square) == 0 {
                        moves.add_move(encode_move!(source_square, target_square, piece, 12, 0, 0, 0, 0));
                    } else {
                        moves.add_move(encode_move!(source_square, target_square, piece, 12, 1, 0, 0, 0));
                    }
    
                    pop_bit!(attacks, target_square);
                }
    
                pop_bit!(bb, source_square);
            }
        }

    }
    moves
}
