use crate::{Color, get_bit, pop_bit};
use crate::bitboard::{get_ls1b_index, count_bits};
use crate::set_bit;

/******************************************\
 ==========================================
                Attacks
 ==========================================
\******************************************/
const NOT_A_FILE:u64 = !0b0000000100000001000000010000000100000001000000010000000100000001;
const NOT_B_FILE:u64 = !0b0000001000000010000000100000001000000010000000100000001000000010;
const NOT_G_FILE:u64 = !0b0100000001000000010000000100000001000000010000000100000001000000;
const NOT_H_FILE:u64 = !0b1000000010000000100000001000000010000000100000001000000010000000;

const NOT_AB_FILE:u64 = NOT_A_FILE & NOT_B_FILE;
const NOT_GH_FILE:u64 = NOT_G_FILE & NOT_H_FILE;


pub const BISHOP_RELEVANT_BITS:[usize;64] = [
    6, 5, 5, 5, 5, 5, 5, 6,
    5, 5, 5, 5, 5, 5, 5, 5,
    5, 5, 7, 7, 7, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 7, 7, 7, 5, 5,
    5, 5, 5, 5, 5, 5, 5, 5,
    6, 5, 5, 5, 5, 5, 5, 6
];
pub const ROOK_RELEVANT_BITS:[usize;64] = [
    12, 11, 11, 11, 11, 11, 11, 12,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    12, 11, 11, 11, 11, 11, 11, 12
];

pub const ROOK_MAGIC_NUMBERS:[u64;64] = [
    0x8a80104000800020,
    0x140002000100040,
    0x2801880a0017001,
    0x100081001000420,
    0x200020010080420,
    0x3001c0002010008,
    0x8480008002000100,
    0x2080088004402900,
    0x800098204000,
    0x2024401000200040,
    0x100802000801000,
    0x120800800801000,
    0x208808088000400,
    0x2802200800400,
    0x2200800100020080,
    0x801000060821100,
    0x80044006422000,
    0x100808020004000,
    0x12108a0010204200,
    0x140848010000802,
    0x481828014002800,
    0x8094004002004100,
    0x4010040010010802,
    0x20008806104,
    0x100400080208000,
    0x2040002120081000,
    0x21200680100081,
    0x20100080080080,
    0x2000a00200410,
    0x20080800400,
    0x80088400100102,
    0x80004600042881,
    0x4040008040800020,
    0x440003000200801,
    0x4200011004500,
    0x188020010100100,
    0x14800401802800,
    0x2080040080800200,
    0x124080204001001,
    0x200046502000484,
    0x480400080088020,
    0x1000422010034000,
    0x30200100110040,
    0x100021010009,
    0x2002080100110004,
    0x202008004008002,
    0x20020004010100,
    0x2048440040820001,
    0x101002200408200,
    0x40802000401080,
    0x4008142004410100,
    0x2060820c0120200,
    0x1001004080100,
    0x20c020080040080,
    0x2935610830022400,
    0x44440041009200,
    0x280001040802101,
    0x2100190040002085,
    0x80c0084100102001,
    0x4024081001000421,
    0x20030a0244872,
    0x12001008414402,
    0x2006104900a0804,
    0x1004081002402
];
pub const BISHOP_MAGIC_NUMBERS:[u64;64] = [
    0x40040844404084,
    0x2004208a004208,
    0x10190041080202,
    0x108060845042010,
    0x581104180800210,
    0x2112080446200010,
    0x1080820820060210,
    0x3c0808410220200,
    0x4050404440404,
    0x21001420088,
    0x24d0080801082102,
    0x1020a0a020400,
    0x40308200402,
    0x4011002100800,
    0x401484104104005,
    0x801010402020200,
    0x400210c3880100,
    0x404022024108200,
    0x810018200204102,
    0x4002801a02003,
    0x85040820080400,
    0x810102c808880400,
    0xe900410884800,
    0x8002020480840102,
    0x220200865090201,
    0x2010100a02021202,
    0x152048408022401,
    0x20080002081110,
    0x4001001021004000,
    0x800040400a011002,
    0xe4004081011002,
    0x1c004001012080,
    0x8004200962a00220,
    0x8422100208500202,
    0x2000402200300c08,
    0x8646020080080080,
    0x80020a0200100808,
    0x2010004880111000,
    0x623000a080011400,
    0x42008c0340209202,
    0x209188240001000,
    0x400408a884001800,
    0x110400a6080400,
    0x1840060a44020800,
    0x90080104000041,
    0x201011000808101,
    0x1a2208080504f080,
    0x8012020600211212,
    0x500861011240000,
    0x180806108200800,
    0x4000020e01040044,
    0x300000261044000a,
    0x802241102020002,
    0x20906061210001,
    0x5a84841004010310,
    0x4010801011c04,
    0xa010109502200,
    0x4a02012000,
    0x500201010098b028,
    0x8040002811040900,
    0x28000010020204,
    0x6000020202d0240,
    0x8918844842082200,
    0x4010011029020020
];


pub static mut PAWN_ATTACKS:[[u64;64];2] = [[0;64];2];
pub static mut KNIGHT_ATTACKS:[u64;64] = [0;64];
pub static mut KING_ATTACKS:[u64;64] = [0;64];

pub static mut BISHOP_MASKS:[u64;64] = [0;64];
pub static mut ROOK_MASKS:[u64;64] = [0;64];
pub static mut BISHOP_ATTACKS:[[u64;512];64] = [[0;512];64]; 
pub static mut ROOK_ATTACKS:[[u64;4096];64] = [[0;4096];64];

#[inline(always)]
pub fn get_bishop_attacks(square:usize, mut occupancy:u64) -> u64 {
    unsafe { occupancy &= BISHOP_MASKS[square]; }
    occupancy = occupancy.wrapping_mul(BISHOP_MAGIC_NUMBERS[square]);
    occupancy >>= 64-BISHOP_RELEVANT_BITS[square];

    return unsafe { BISHOP_ATTACKS[square][occupancy as usize] }
}

#[inline(always)]
pub fn get_rook_attacks(square:usize, mut occupancy:u64) -> u64 {
    unsafe { occupancy &= ROOK_MASKS[square]; }
    occupancy = occupancy.wrapping_mul(ROOK_MAGIC_NUMBERS[square]);
    occupancy >>= 64-ROOK_RELEVANT_BITS[square];

    return unsafe { ROOK_ATTACKS[square][occupancy as usize] }
}

// #[allow(unused_assignments)]
#[inline(always)]
pub fn get_queen_attacks(square:usize, occupancy:u64) -> u64 {
    let mut result:u64 = 0;
    let mut bishop_occupancies = occupancy;
    let mut rook_occupancies = occupancy;

    // Bishop attacks
    unsafe { bishop_occupancies &= BISHOP_MASKS[square]; }
    bishop_occupancies = bishop_occupancies.wrapping_mul(BISHOP_MAGIC_NUMBERS[square]);
    bishop_occupancies >>= 64-BISHOP_RELEVANT_BITS[square];
    unsafe { result |= BISHOP_ATTACKS[square][bishop_occupancies as usize]; };

    unsafe { rook_occupancies &= ROOK_MASKS[square]; }
    rook_occupancies = rook_occupancies.wrapping_mul(ROOK_MAGIC_NUMBERS[square]);
    rook_occupancies >>= 64-ROOK_RELEVANT_BITS[square];
    unsafe { result |= ROOK_ATTACKS[square][rook_occupancies as usize]; };

    return result
}


fn mask_pawn_attacks(side:Color, square:usize) -> u64 {
    let mut attacks:u64 = 0;
    let mut bitboard:u64 = 0;
    let mut mask:u64;

    set_bit!(bitboard, square);

    match side {
        Color::White => {
            if {mask = (bitboard >> 7) & NOT_A_FILE; mask!=0} {attacks |= mask;};
            if {mask = (bitboard >> 9) & NOT_H_FILE; mask!=0} {attacks |= mask;};
        },
        Color::Black => {
            if {mask = (bitboard << 7) & NOT_H_FILE; mask!=0} {attacks |= mask;};
            if {mask = (bitboard << 9) & NOT_A_FILE; mask!=0} {attacks |= mask;};
        },
        _ => (),
    }

    attacks
}

fn mask_knight_attacks(square:usize) -> u64 {
    let mut attacks:u64 = 0;
    let mut bitboard:u64 = 0;
    let mut mask:u64;

    set_bit!(bitboard, square);

    if {mask = (bitboard >> 17) & NOT_H_FILE; mask!=0} {attacks |= mask;}; //^^<
    if {mask = (bitboard >> 15) & NOT_A_FILE; mask!=0} {attacks |= mask;}; //^^>
    if {mask = (bitboard >> 10) & NOT_GH_FILE; mask!=0} {attacks |= mask;}; //^<<
    if {mask = (bitboard >> 6) & NOT_AB_FILE; mask!=0} {attacks |= mask;}; //^>>

    if {mask = (bitboard << 17) & NOT_A_FILE; mask!=0} {attacks |= mask;};
    if {mask = (bitboard << 15) & NOT_H_FILE; mask!=0} {attacks |= mask;};
    if {mask = (bitboard << 10) & NOT_AB_FILE; mask!=0} {attacks |= mask;};
    if {mask = (bitboard << 6) & NOT_GH_FILE; mask!=0} {attacks |= mask;};

    attacks
}

fn mask_king_attacks(square:usize) -> u64 {
    let mut attacks:u64 = 0;
    let mut bitboard:u64 = 0;
    let mut mask:u64;

    set_bit!(bitboard, square);

    attacks |= bitboard >> 8;
    if {mask = (bitboard >> 9) & NOT_H_FILE; mask!=0} {attacks |= mask;};
    if {mask = (bitboard >> 7) & NOT_A_FILE; mask!=0} {attacks |= mask;};
    if {mask = (bitboard >> 1) & NOT_H_FILE; mask!=0} {attacks |= mask;};

    attacks |= bitboard << 8;
    if {mask = (bitboard << 9) & NOT_A_FILE; mask!=0} {attacks |= mask;};
    if {mask = (bitboard << 7) & NOT_H_FILE; mask!=0} {attacks |= mask;};
    if {mask = (bitboard << 1) & NOT_A_FILE; mask!=0} {attacks |= mask;};

    attacks
}


pub fn mask_bishop_attacks(square:usize) -> u64 {
    let mut attacks:u64 = 0;

    // rank, current rank, file, target file
    let mut r:usize;
    let tr:usize = square/8;
    let mut f:usize;
    let tf:usize = square%8;

    r = tr + 1; f = tf + 1;
    while r < 7 && f < 7 { attacks |= 1u64 << (r * 8 + f); r += 1; f += 1; }
    if tr > 0 {r = tr - 1; f = tf + 1} else {r=0; f=tf+1;};
    while r > 0 && f < 7 { attacks |= 1u64 << (r * 8 + f); r -= 1; f += 1; }
    if tf > 0 {r = tr + 1; f = tf - 1} else {r=tr+1; f=0;};
    while r < 7 && f > 0 { attacks |= 1u64 << (r * 8 + f); r += 1; f -= 1; }
    if tr>0 && tf>0 {r = tr - 1; f = tf - 1} else {r=0; f=0};
    while r > 0 && f > 0 { attacks |= 1u64 << (r * 8 + f); r -= 1; f -= 1; }

    attacks
}

pub fn bishop_attacks(square:usize, block:u64) -> u64 {
    let mut attacks:u64 = 0;
    let mut mask:u64;

    // rank, current rank, file, target file
    let mut r:isize;
    let tr:isize = (square/8) as isize;
    let mut f:isize;
    let tf:isize = (square%8) as isize;

    r = tr + 1; f = tf + 1;
    while r < 8 && f < 8 { 
        mask = 1u64 << (r * 8 + f);
        attacks |= mask;
        if (mask & block) != 0 { break; };
        r += 1; f += 1; 
    }
    r = tr - 1; f = tf + 1;
    while r > -1 && f < 8 { 
        mask = 1u64 << (r * 8 + f);
        attacks |= mask;
        if (mask & block) != 0 { break; };
        r -= 1; f += 1; 
    }
    r = tr + 1; f = tf - 1;
    while r < 8 && f > -1 { 
        mask = 1u64 << (r * 8 + f);
        attacks |= mask;
        if (mask & block) != 0 { break; };
        r += 1; f -= 1; 
    }
    r = tr - 1; f = tf - 1;
    while r > -1 && f > -1 { 
        mask = 1u64 << (r * 8 + f);
        attacks |= mask;
        if (mask & block) != 0 { break; };
        r -= 1; f -= 1; 
    }

    attacks
}

pub fn mask_rook_attacks(square:usize) -> u64 {
    let mut attacks:u64 = 0;

    // rank, current rank, file, target file
    let mut r:usize;
    let tr:usize = square/8;
    let mut f:usize;
    let tf:usize = square%8;

    r = tr + 1;
    while r < 7 { attacks |= 1u64 << (r * 8 + tf); r += 1 }
    if tr > 0 {r = tr - 1;} else {r = 0;};
    while r > 0 { attacks |= 1u64 << (r * 8 + tf); r -= 1; }
    f = tf + 1;
    while f < 7 { attacks |= 1u64 << (tr * 8 + f); f += 1; }
    if tf > 0 {f = tf - 1} else {f = 0;};
    while f > 0 { attacks |= 1u64 << (tr * 8 + f); f -= 1; }

    attacks
}

pub fn rook_attacks(square:usize, block:u64) -> u64 {
    let mut attacks:u64 = 0;
    let mut mask:u64;

    // rank, current rank, file, target file
    let mut r:isize;
    let tr:isize = (square/8) as isize;
    let mut f:isize;
    let tf:isize = (square%8) as isize;

    r = tr + 1;
    while r < 8 { 
        mask = 1u64 << (r * 8 + tf); 
        attacks |= mask;
        if (mask & block) != 0 { break; };
        r += 1;
    }
    r = tr - 1;
    while r > -1 { 
        mask = 1u64 << (r * 8 + tf); 
        attacks |= mask;
        if (mask & block) != 0 { break; };
        r -= 1; 
    }
    f = tf + 1;
    while f < 8 { 
        mask = 1u64 << (tr * 8 + f); 
        attacks |= mask;
        if (mask & block) != 0 { break; };
        f += 1; 
    }
    f = tf - 1;
    while f > -1 { 
        mask = 1u64 << (tr * 8 + f); 
        attacks |= mask;
        if (mask & block) != 0 { break; };
        f -= 1; 
    }

    attacks
}


pub unsafe fn init_leapers_attacks() {
    for square in 0..64 {
        PAWN_ATTACKS[0][square] = mask_pawn_attacks(Color::White, square);
        PAWN_ATTACKS[1][square] = mask_pawn_attacks(Color::Black, square);

        KNIGHT_ATTACKS[square] = mask_knight_attacks(square);
        
        KING_ATTACKS[square] = mask_king_attacks(square);
    }
}

pub unsafe fn init_sliders_attacks(is_bishop:bool) {
    let mut attack_mask:u64;
    let mut relevant_bits_count:usize;
    let mut occupancy_indicies:usize;
    for square in 0..64usize {
        if is_bishop {
            BISHOP_MASKS[square] = mask_bishop_attacks(square);
            attack_mask = BISHOP_MASKS[square];
        } else {
            ROOK_MASKS[square] = mask_rook_attacks(square);
            attack_mask = ROOK_MASKS[square];
        }

        relevant_bits_count = count_bits(attack_mask);
        occupancy_indicies = 1 << relevant_bits_count;

        for index in 0..occupancy_indicies {
            let occupancy = set_occupancy(index, relevant_bits_count, attack_mask);
            if is_bishop {
                let magic_index = (occupancy.wrapping_mul(BISHOP_MAGIC_NUMBERS[square]) >> (64-BISHOP_RELEVANT_BITS[square])) as usize;
                BISHOP_ATTACKS[square][magic_index] = bishop_attacks(square, occupancy);
            } else {
                let magic_index = (occupancy.wrapping_mul(ROOK_MAGIC_NUMBERS[square]) >> (64-ROOK_RELEVANT_BITS[square])) as usize;
                ROOK_ATTACKS[square][magic_index] = rook_attacks(square, occupancy);
            }
            
        }
    }
}


pub fn set_occupancy(index:usize, bits_in_mask:usize, mut attack_mask:u64) -> u64 {
    // Counting using the attack mask bits
    let mut occupancy:u64 = 0;

    for count in 0..bits_in_mask {
        let square:usize = get_ls1b_index(attack_mask);
        pop_bit!(attack_mask, square);

        if index & (1 << count) != 0 {
            occupancy |= 1u64 << square;
        }
    }

    occupancy
}

