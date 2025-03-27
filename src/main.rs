// CHIP-8 SPECIFICS
struct CHIP8 {
    registers: [u8; 16],   // 16 8-bit Registers
    memory: [u8; 4096],    // 4K Bytes of Memory
    IR: u16, // 16-bit Index Register (16 bits are needed to hold the maximum memory adress 0xFFF)
    PC: u16, // 16-bit Program Counter
    stack: [u16; 16], // 16 level Execution Stack
    st_pointer: u8, // 8-bit Stack Pointer
    delayTimer: u8, // 8-bit Delay Timer
    soundTimer: u8, // 8-bit Sound Timer
    keypad: [u8; 16], // 16 input keys
    video: [u32; 64 * 32], // 64 by 32 pixels video screen
    opcode: u16, // 2 Byte operation code
}

fn main() {}
