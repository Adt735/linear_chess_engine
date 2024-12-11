

/*
          binary move bits                               hexidecimal constants
    
    0000 0000 0000 0000 0011 1111    source square       0x3f
    0000 0000 0000 1111 1100 0000    target square       0xfc0
    0000 0000 1111 0000 0000 0000    piece               0xf000
    0000 1111 0000 0000 0000 0000    promoted piece      0xf0000
    0001 0000 0000 0000 0000 0000    capture flag        0x100000
    0010 0000 0000 0000 0000 0000    double push flag    0x200000
    0100 0000 0000 0000 0000 0000    enpassant flag      0x400000
    1000 0000 0000 0000 0000 0000    castling flag       0x800000
*/

use crate::{SQUARE_TO_COORDINATES, bitboard::ASCII_PIECES};

#[macro_export]
macro_rules! encode_move {
    ($source:expr, $target:expr, $piece:expr, $promoted:expr, $capture:expr, $double:expr, $enpassant:expr, $castling:expr) => {
        $source | $target<<6 | $piece<<12 | $promoted<<16 | $capture<<20 | $double<<21 | $enpassant<<22 | $castling<<23
    };
}

#[macro_export]
macro_rules! get_move_source {
    ($move:expr) => {
        $move & 0x3f
    };
}
#[macro_export]
macro_rules! get_move_target {
    ($move:expr) => {
        ($move & 0xfc0) >> 6
    };
}
#[macro_export]
macro_rules! get_move_piece {
    ($move:expr) => {
        ($move & 0xf000) >> 12
    };
}
#[macro_export]
macro_rules! get_move_promoted {
    ($move:expr) => {
        ($move & 0xf0000) >> 16
    };
}
#[macro_export]
macro_rules! get_move_capture {
    ($move:expr) => {
        ($move & 0x100000) != 0
    };
}
#[macro_export]
macro_rules! get_move_double {
    ($move:expr) => {
        ($move & 0x200000) != 0
    };
}
#[macro_export]
macro_rules! get_move_enpassant {
    ($move:expr) => {
        ($move & 0x400000) != 0
    };
}
#[macro_export]
macro_rules! get_move_castling {
    ($move:expr) => {
        ($move & 0x800000) != 0
    };
}


pub struct Moves {
    pub moves:Vec<usize>,
    pub count:usize
}

impl Moves {
    pub fn new() -> Moves {
        Moves {
            moves: Vec::with_capacity(256),
            count: 0,
        }
    }

    pub fn add_move(&mut self, move_:usize) {
        self.moves.push(move_);
        self.count += 1;
    }

    pub fn print(&self) {
        let mut c = self.count;
        println!("  Move      Piece     Capture   Double    Enpassant Castle    \n");
        while c != 0 {
            c -= 1;
            println!("  {}     {}         {}     {}     {}     {}", move_str(self.moves[c]), ASCII_PIECES[get_move_piece!(self.moves[c])], get_move_capture!(self.moves[c]),
                get_move_double!(self.moves[c]), get_move_enpassant!(self.moves[c]), get_move_castling!(self.moves[c])
            );
        }
        println!("Total moves: {}", self.count);
    }
}

pub fn print_move(move_:usize) {
    println!("{}", move_str(move_));
    
}
pub fn move_str(move_:usize) -> String {
    if get_move_promoted!(move_) < 12 {
        format!("{}{}{}", SQUARE_TO_COORDINATES[get_move_source!(move_)],
                        SQUARE_TO_COORDINATES[get_move_target!(move_)],
                        ASCII_PIECES[get_move_promoted!(move_)].to_lowercase()
            )
    } else {
        format!("{}{}", SQUARE_TO_COORDINATES[get_move_source!(move_)],
                        SQUARE_TO_COORDINATES[get_move_target!(move_)]
            )
    }
}


// struct Move {
//     source:usize,
//     target:usize,
//     piece:usize, 
//     promoted:usize, 
//     capture:usize, 
//     double:usize, 
//     enpassant:usize, 
//     castling:usize
// }

// impl Move {
//     pub fn new(source:usize, target:usize, piece:usize, promoted:usize, capture:usize, double:usize, enpassant:usize, castling:usize) -> Move {
//         Move {
//             source,
//             target,
//             piece,
//             promoted,
//             capture,
//             double,
//             enpassant,
//             castling
//         }
//     }
// }

