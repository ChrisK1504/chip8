use std::env;
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

// Fontset Size
const FONTSET_SIZE: u8 = 80;
// Fontset Address (Fontsets begin to be stored in 0x50, in memory)
const FONTSET_ADDRESS: u8 = 0x50;
// Every 5 bytes represents a 'sprite', for a total of 16 haracters
const FONTSET: [u8; FONTSET_SIZE as usize] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

impl CHIP8 {
    // Constructor to create a new chip8 model
    fn new() -> Self {
        let mut chip8: CHIP8 = CHIP8 {
            registers: [0x00; 16],
            memory: [0x00; 4096],
            PC: START_ADDRESS, // Program Counter set to First Instruction
        };

        // Start loading the font bytes into memory, starting from 0x50
        for (i, font) in FONTSET.iter().enumerate() {
            chip8.memory[FONTSET_ADDRESS as usize + i] = *font;
        }

        // Return the newly constructed chip
        return chip8;
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
