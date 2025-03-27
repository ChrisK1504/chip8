use std::env;
use std::fmt::LowerHex;
use std::fs;

// CHIP-8 SPECIFICS
struct CHIP8 {
    registers: [u8; 16], // 16 8-bit Registers
    memory: [u8; 4096],  // 4K Bytes of Memory
    // IR: u16, // 16-bit Index Register (16 bits are needed to hold the maximum memory adress 0xFFF)
    PC: u16, // 16-bit Program Counter
             // stack: [u16; 16], // 16 level Execution Stack
             // st_pointer: u8, // 8-bit Stack Pointer
             // delayTimer: u8, // 8-bit Delay Timer
             // soundTimer: u8, // 8-bit Sound Timer
             // keypad: [u8; 16], // 16 input keys
             // video: [u32; 64 * 32], // 64 by 32 pixels video screen
             // opcode: u16, // 2 Byte operation code
}

// Instructions are stored starting at address 0x200
const START_ADDRESS: u16 = 0x200;

impl CHIP8 {
    // Constructor to create a new chip8 model
    fn new() -> Self {
        Self {
            registers: [0x00; 16],
            memory: [0x00; 4096],
            PC: START_ADDRESS, // Program Counter set to First Instruction
        }
    }
    // Function to load a ROM File using a file name
    fn load_rom(chip8: &mut CHIP8, filename_path: &String) {
        // The ROM file is a binary file
        // The bytes are stored in a Vector-array
        let rom: Vec<u8> = fs::read(filename_path).expect("File not found");

        // Store the instructions from the vector-array in the chip's memory starting from 0x200
        for (i, instruction) in rom.iter().enumerate() {
            chip8.memory[START_ADDRESS as usize + i] = *instruction;
        }
    }
}

fn main() {
    // Create new chip
    let mut chip8: CHIP8 = CHIP8::new();
    // Collect command line arguments
    let args: Vec<String> = env::args().collect();
    // Set the filename as the second argument (first argument is always the program name)
    let filename_path = &args[1];

    // Load ROM Instructions into Memory from the file path
    CHIP8::load_rom(&mut chip8, filename_path);
}
