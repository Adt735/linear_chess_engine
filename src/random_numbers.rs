#![allow(dead_code)]

/******************************************\
 ==========================================
            Random Generator
 ==========================================
\******************************************/

use crate::attacks::{mask_bishop_attacks, mask_rook_attacks, set_occupancy, bishop_attacks, rook_attacks, BISHOP_RELEVANT_BITS, ROOK_RELEVANT_BITS};

pub static mut STATE:u32 = 1804289383;

fn get_random_u32_number() -> u32 {
    let mut number:u32;
    unsafe { number = STATE; };

    // XOR shift algorithm
    number ^= number << 13;
    number ^= number >> 17;
    number ^= number << 5;

    unsafe { STATE = number; };

    number
}

pub fn get_random_u64_number() -> u64 {
    let n1:u64; let n2:u64; let n3:u64; let n4:u64;

    n1 = (get_random_u32_number()) as u64 & 0xFFFF;
    n2 = (get_random_u32_number()) as u64 & 0xFFFF;
    n3 = (get_random_u32_number()) as u64 & 0xFFFF;
    n4 = (get_random_u32_number()) as u64 & 0xFFFF;

    n1 | (n2 << 16) | (n3 <<32) | (n4 << 48)
}

fn generate_magic_number() -> u64 {
    get_random_u64_number() & get_random_u64_number() & get_random_u64_number()
}

/******************************************\
 ==========================================
            Magic Numbers
 ==========================================
\******************************************/

pub fn find_magic_number(square:usize, relevant_bits:usize, is_bishop:bool) -> u64 {
    let mut occupancies:[u64;4096] = [0;4096];
    let mut attacks:[u64;4096] = [0;4096];
    let mut used_attacks:[u64;4096];
    let attack_mask = if is_bishop {mask_bishop_attacks(square)} else {mask_rook_attacks(square)};
    let occupancy_indicies:usize = 1usize << relevant_bits;

    for index in 0..occupancy_indicies {
        occupancies[index] = set_occupancy(index, relevant_bits, attack_mask);
        attacks[index] = if is_bishop {bishop_attacks(square, occupancies[index])} else {rook_attacks(square, occupancies[index])};
    }

    // Test magic number loop
    for _ in 0..10000000 {       
        let magic_number = generate_magic_number();

        // Skip inappropiate magic numbers
        if ((u128::from(attack_mask) * u128::from(magic_number)) & u128::from(0xFF00000000000000u64)).count_ones() < 6 { continue; };

        used_attacks = [0;4096];
        let mut index:usize = 0;
        let mut has_failed:bool = false;

        // Test magic index loop
        while !has_failed && (index < occupancy_indicies) {
            let magic_index = (((occupancies[index].wrapping_mul(magic_number)) >> (64 - relevant_bits))) as usize;

            if used_attacks[magic_index] == 0 {
                used_attacks[magic_index] = attacks[index];
            } else if used_attacks[magic_index] != attacks[index] {
                has_failed = true;
            }

            index += 1;
        }

        if !has_failed {
            return magic_number
        }
    }

    println!("Magic number search failed!");
    return 0
} 

pub fn init_magic_numbers() {
    for square in 0..64usize {
        println!(" {:x}", find_magic_number(square, ROOK_RELEVANT_BITS[square], false));
    }
    for square in 0..64usize {
        println!(" {:x}", find_magic_number(square, BISHOP_RELEVANT_BITS[square], true));
    }
}
