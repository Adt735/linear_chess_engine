/******************************************\
 ==========================================
            Bit Manipulations
 ==========================================
\******************************************/

#[allow(dead_code)]
pub const MAXUSIZE: usize = usize::MAX;

#[macro_export]
macro_rules! get_bit {
    ($bitboard:expr, $square:expr) => {
        $bitboard & (1u64 << $square)
    };
}

#[macro_export]
macro_rules! set_bit {
    ($bitboard:expr, $square:expr) => {
        $bitboard |= 1u64 << $square
    };
}

#[macro_export]
macro_rules! pop_bit {
    ($bitboard:expr, $square:expr) => {
        if get_bit!($bitboard, $square)!=0 {$bitboard ^= 1u64 << $square}
    };
}

#[inline(always)]
pub fn count_bits(mut bitboard: u64) -> usize {
    let mut count = 0;
    
    while bitboard != 0 {
        count += 1;
        bitboard &= bitboard - 1;
    }
    
    count
} 

#[inline(always)]
pub fn get_ls1b_index(bitboard: u64) -> usize {
    if bitboard != 0 {
        return count_bits((bitboard & (!bitboard + 1)) - 1);
    } else {
        MAXUSIZE
    }
}

/******************************************\
 ==========================================
                Bit Board
 ==========================================
\******************************************/

use std::collections::HashMap;
use std::slice::Iter;

use crate::{Side, Color, Square, SQUARE_TO_COORDINATES, get_move_capture, get_move_source, get_move_target, get_move_piece, get_move_promoted, get_move_double, get_move_enpassant, get_move_castling, move_gen::is_square_attacked, hashing::{generate_hash_key, SIDE_KEY, PIECE_KEYS, ENPASSANT_KEYS, CASTLE_KEYS}};

#[derive(Clone)]
pub struct Board {
    pub bitboards:[u64;12],
    pub occupancies:[u64;3],
    pub side:Side,

    pub en_passant:Option<usize>,
    pub castle:u8,

    pub hash_key:u64,
    pub repetition_table:[u64;1000], // Number of PLY in the intire game
    pub repetition_index:usize,
}

impl Board {
    pub fn new() -> Board {
        Board {
            bitboards:[0;12],
            occupancies:[0;3],
            side:Side::None,
            en_passant:None,
            castle:0,
            hash_key:0,
            repetition_table:[0;1000],
            repetition_index:0,
        }
    }

    pub fn new_from_fen(fen:&str) -> Board {
        let mut board = Board::new();
        let mut chars = fen.chars();
        let mut char_ = chars.next();

        // Board position
        let mut rank = 0;
        let mut file = 0;
        'outer_fen: while rank < 8 {
            while file < 8 {
                let square = rank * 8 + file;

                match char_ {
                    Some(c) => {
                        if c.is_alphabetic() {
                            let piece = ASCII_TO_PIECE[&c];
                            set_bit!(board.bitboards[piece], square)
                        } else if c.is_numeric() {
                            let offset = (c as i32) - ('1' as i32);
                            file += offset;
                        } else if c == '/'{
                            file -= 1;
                        }

                        if c == ' ' { break 'outer_fen; }
                    },
                    None => { break 'outer_fen; }
                }

                char_ = chars.next();
                file += 1;
            }
            file = 0;
            rank += 1;
        }
        
        // Side to castle
        char_ = chars.next();
        match char_ {
            Some(c) => board.side = if c=='w' { Side::White } else { Side::Black },
            None => (),
        }

        // Castling Side
        chars.next();
        char_ = chars.next();
        let mut current_char = *char_.get_or_insert(' ');
        while current_char != ' ' {
            match current_char {
                'K' => board.castle |= CastlingSide::WK as u8,
                'Q' => board.castle |= CastlingSide::WQ as u8,
                'k' => board.castle |= CastlingSide::BK as u8,
                'q' => board.castle |= CastlingSide::BQ as u8,
                _ => { chars.next(); break; },
            }
            current_char = *chars.next().get_or_insert(' ');
        }

        // Enpassant square
        current_char = *chars.next().get_or_insert('-');
        if current_char != '-' {
            let next_char = chars.next().expect("Enpassant not correctly formatted");
            board.en_passant = Some(SQUARE_TO_COORDINATES.iter().position(
                |&r| r == format!("{}{}", current_char, next_char)
            ).unwrap());
        } else {
            board.en_passant = None
        }

        // Init occupancies
        for w_piece in (Pieces::P as usize)..(Pieces::K as usize + 1) {
            board.occupancies[Color::White as usize] |= board.bitboards[w_piece];
        }
        for b_piece in (Pieces::p as usize)..(Pieces::k as usize + 1) {
            board.occupancies[Color::Black as usize] |= board.bitboards[b_piece];
        }
        board.occupancies[Color::Both as usize] |= board.occupancies[Color::White as usize];
        board.occupancies[Color::Both as usize] |= board.occupancies[Color::Black as usize];

        unsafe { board.hash_key = generate_hash_key(&board); }

        board
    }

    pub unsafe fn make_move(&mut self, move_:usize, only_captures:bool) -> bool {
        if !only_captures {
            let previous_board = self.clone();

            // Parsing move
            let source_square = get_move_source!(move_);
            let target_square = get_move_target!(move_);
            let piece = get_move_piece!(move_);
            let promoted = get_move_promoted!(move_);
            let capture = get_move_capture!(move_);
            let double = get_move_double!(move_);
            let enpassant = get_move_enpassant!(move_);
            let castle = get_move_castling!(move_);

            // Updating move
            pop_bit!(self.bitboards[piece], source_square);
            set_bit!(self.bitboards[piece], target_square);

            // Hash piece
            self.hash_key ^= PIECE_KEYS[piece][source_square]; // Remove piece
            self.hash_key ^= PIECE_KEYS[piece][target_square]; // Set piece

            // If capture, remove bit from opponents bitboard
            if capture {
                let offset = if self.side == Side::White {6} else {0};
                for bb_piece in (0+offset)..(6+offset) {
                    if get_bit!(self.bitboards[bb_piece], target_square) != 0 {
                        pop_bit!(self.bitboards[bb_piece], target_square);
                        // Remove piece from hash key
                        self.hash_key ^= PIECE_KEYS[bb_piece][target_square];
                        break;
                    }
                }
            }

            // If promotion, update corresponding bitboard
            if promoted < 12 {
                pop_bit!(self.bitboards[piece], target_square);
                self.hash_key ^= PIECE_KEYS[piece][target_square];
                set_bit!(self.bitboards[promoted], target_square);
                self.hash_key ^= PIECE_KEYS[promoted][target_square];
            }

            // Manage enpassant case
            if enpassant {
                if self.side == Side::White {
                    pop_bit!(self.bitboards[6], target_square+8);
                    self.hash_key ^= PIECE_KEYS[6][target_square+8];
                } else {
                    pop_bit!(self.bitboards[0], target_square-8);
                    self.hash_key ^= PIECE_KEYS[0][target_square-8];
                }
            }

            if let Some(square) = self.en_passant {
                self.hash_key ^= ENPASSANT_KEYS[square]
            }

            self.en_passant = None;

            // Enabling enpassant square /if double push
            if double {
                if self.side == Side::White {
                    self.en_passant = Some(target_square+8);
                    self.hash_key ^= ENPASSANT_KEYS[target_square+8];
                } else {
                    self.en_passant = Some(target_square-8);
                    self.hash_key ^= ENPASSANT_KEYS[target_square-8];
                }
            }

            // Case of castling
            if castle && self.castle!=0 {
                match target_square {
                    x if x == Square::g1 as usize => {
                        pop_bit!(self.bitboards[Pieces::R as usize], Square::h1 as usize);
                        set_bit!(self.bitboards[Pieces::R as usize], Square::f1 as usize);
                        self.hash_key ^= PIECE_KEYS[Pieces::R as usize][Square::h1 as usize];
                        self.hash_key ^= PIECE_KEYS[Pieces::R as usize][Square::f1 as usize];
                    },
                    x if x == Square::c1 as usize => {
                        pop_bit!(self.bitboards[Pieces::R as usize], Square::a1 as usize);
                        set_bit!(self.bitboards[Pieces::R as usize], Square::d1 as usize);
                        self.hash_key ^= PIECE_KEYS[Pieces::R as usize][Square::a1 as usize];
                        self.hash_key ^= PIECE_KEYS[Pieces::R as usize][Square::d1 as usize];
                    },
                    x if x == Square::g8 as usize => {
                        pop_bit!(self.bitboards[Pieces::r as usize], Square::h8 as usize);
                        set_bit!(self.bitboards[Pieces::r as usize], Square::f8 as usize);
                        self.hash_key ^= PIECE_KEYS[Pieces::r as usize][Square::h8 as usize];
                        self.hash_key ^= PIECE_KEYS[Pieces::r as usize][Square::f8 as usize];
                    },
                    x if x == Square::c8 as usize => {
                        pop_bit!(self.bitboards[Pieces::r as usize], Square::a8 as usize);
                        set_bit!(self.bitboards[Pieces::r as usize], Square::d8 as usize);
                        self.hash_key ^= PIECE_KEYS[Pieces::r as usize][Square::a8 as usize];
                        self.hash_key ^= PIECE_KEYS[Pieces::r as usize][Square::d8 as usize];
                    },
                    _ => (),
                }
            }

            // Updating castle rights
            if self.castle != 0 {
                self.hash_key ^= CASTLE_KEYS[self.castle as usize];
                self.castle &= CASTLING_RIGHTS[source_square];
                self.castle &= CASTLING_RIGHTS[target_square];
                self.hash_key ^= CASTLE_KEYS[self.castle as usize];
            }

            // Update occupancies
            self.occupancies = [0;3];
            for w_piece in (Pieces::P as usize)..(Pieces::K as usize + 1) {
                self.occupancies[0] |= self.bitboards[w_piece];
            }
            for b_piece in (Pieces::p as usize)..(Pieces::k as usize + 1) {
                self.occupancies[1] |= self.bitboards[b_piece];
            }
            self.occupancies[Color::Both as usize] |= self.occupancies[Color::White as usize];
            self.occupancies[Color::Both as usize] |= self.occupancies[Color::Black as usize];

            // Check if the move is legal
            let other_side = if self.side==Side::White {false} else {true};
            self.side = if other_side {Side::White} else {Side::Black};

            // Hashing key
            self.hash_key ^= SIDE_KEY;

            self.repetition_index = 0;


            if is_square_attacked(
                self, 
                if other_side {get_ls1b_index(self.bitboards[11])} else {get_ls1b_index(self.bitboards[5])}, 
                other_side
            ) {
                *self = previous_board;
                return false
            } else {
                return true
            }
            

        } else {
            if get_move_capture!(move_) {
                return self.make_move(move_, false)
            } else {
                return false
            }
        }
    }

    pub fn take_back(&mut self, other:&Board) {
        self.bitboards = other.bitboards;
        self.occupancies = other.occupancies;
        self.side = other.side;
        self.en_passant = other.en_passant;
        self.castle = other.castle;
        self.hash_key = other.hash_key;
        self.repetition_index = other.repetition_index;
    }

    pub fn print(&self) {
        print!("\n\n");

        for rank in 0..8 {
            for file in 0..8 {
                let square = rank*8 + file;

                if file == 0 {
                    print!("\t{}   ", 8-rank);
                }

                let mut piece = -1;

                for bb_piece in (Pieces::P as usize)..(Pieces::k as usize + 1) {
                    if get_bit!(self.bitboards[bb_piece], square) != 0 {
                        piece = bb_piece as i32;
                        break;
                    }
                }

                print!(" {}", if piece==-1 {'.'} else {ASCII_PIECES[piece as usize]})
            }
            println!();
        }
        print!("\n\t     a b c d e f g h \n\n");

        println!(" Side:\t{}", self.side.to_string());
        println!(" Enpassant:\t{}", match self.en_passant{
            Some(square) => SQUARE_TO_COORDINATES[square],
            None => "None",
        });
        println!(" Castling:\t{}{}{}{}", if self.castle & CastlingSide::WK as u8 != 0 {'K'} else {'-'},
                                         if self.castle & CastlingSide::WQ as u8 != 0 {'Q'} else {'-'},
                                         if self.castle & CastlingSide::BK as u8 != 0 {'k'} else {'-'},
                                         if self.castle & CastlingSide::BQ as u8 != 0 {'q'} else {'-'}
        );
        println!(" Hash key:\t{:x}", self.hash_key);
    }
}

pub enum CastlingSide {WK=1, WQ=2, BK=4, BQ=8}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub enum Pieces {P, N, B, R, Q, K, p, n, b, r, q, k}
impl Pieces {
    pub fn iterator() -> Iter<'static, Pieces> {
        static PIECES:[Pieces;12] = [Pieces::P, Pieces::N, Pieces::B, Pieces::R, Pieces::Q, Pieces::K, 
                                     Pieces::p, Pieces::n, Pieces::b, Pieces::r, Pieces::q, Pieces::k];
        PIECES.iter()
    }
}
static PIECES:[usize;12] = [Pieces::P as usize, Pieces::N as usize, Pieces::B as usize, Pieces::R as usize, Pieces::Q as usize, Pieces::K as usize, 
                                     Pieces::p as usize, Pieces::n as usize, Pieces::b as usize, Pieces::r as usize, Pieces::q as usize, Pieces::k as usize];
pub const ASCII_PIECES:[char;12] = ['P','N','B','R','Q','K','p','n','b','r','q','k'];
lazy_static::lazy_static! {
    pub static ref ASCII_TO_PIECE: HashMap<char, usize> = {
        let mut map = HashMap::new();
        for (&ascii, piece) in ASCII_PIECES.iter().zip(&PIECES) {
            map.insert(ascii, *piece);
        }
        map
    };
}
pub const UNICODE_PIECES:[&str;12] = [
    "&#9817",
    "&#9816",
    "&#9815",
    "&#9814",
    "&#9813",
    "&#9812",
    "&#9823",
    "&#9822",
    "&#9821",
    "&#9820",
    "&#9819",
    "&#9818"
];

const CASTLING_RIGHTS:[u8;64] = [
    7, 15, 15, 15,  3, 15, 15, 11,
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15,
    13, 15, 15, 15, 12, 15, 15, 14
];



/******************************************\
 ==========================================
            Input & Output
 ==========================================
\******************************************/

pub fn print_bitboard(bitboard:u64) {
    print!("\n\n");

    for rank in 0..8 {
        for file in 0..8 {
            let square = rank*8+file;

            if file == 0 {
                print!("\t{}   ", 8-rank);
            }

            print!(" {}", if get_bit!(bitboard,square)!=0 {1} else {0});
        }
        println!();
    }

    print!("\n\t     a b c d e f g h \n\n");
    println!("\t\tBitboard: {}", bitboard);
}
