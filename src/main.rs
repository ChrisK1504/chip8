use minifb;
use minifb::Key;
use minifb::Scale;
use minifb::Window;
use minifb::WindowOptions;
use rand;
use std::env;
use std::fs;

// CHIP-8 SPECIFICS
struct CHIP8 {
    registers: [u8; 16], // 16 8-bit Registers
    memory: [u8; 4096],  // 4K Bytes of Memory
    IR: u16, // 16-bit Index Register (16 bits are needed to hold the maximum memory adress 0xFFF)
    PC: u16, // 16-bit Program Counter
    stack: [u16; 16], // 16 level Execution Stack
    st_pointer: u8, // 8-bit Stack Pointer
    // delayTimer: u8, // 8-bit Delay Timer
    // soundTimer: u8, // 8-bit Sound Timer
    // keypad: [u8; 16], // 16 input keys
    video: [u8; 64 * 32], // 64 by 32 pixels video screen
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
            video: [0; 64 * 32],
            IR: 0,
            stack: [0; 16],
            st_pointer: 0,
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

    // 00E0 - CLS
    // Clear the video display
    fn op_00e0(&mut self) {
        eprintln!("In OP_00E0");
        // Set all pixels in the screen to 0 (black)
        self.video.fill(0);
    }

    // 00EE - RET
    // Return from a subroutine
    fn op_00ee(&mut self) {
        // The top of the stack has the address of one instruction past the one that called the subroutine
        // So we can put that back into the PC.
        eprintln!("In OP_00EE");
        self.st_pointer -= 1;
        self.PC = self.stack[self.st_pointer as usize];
    }

    // 1nnn - JP addr
    // Jump to location at 'nnn'
    fn op_1nnn(&mut self, opcode: u16) {
        eprintln!("In OP_1NNN");
        // Mask the opcode to retrieve the address
        let address: u16 = opcode & 0x0FFF;

        // Set PC to address
        self.PC = address;
    }

    // 2nnn - CALL addr
    // Call subroutine at 'nnn'
    fn op_2nnn(&mut self, opcode: u16) {
        eprintln!("In OP_2NNN");

        // Mask the opcode to retrieve the address
        let address: u16 = opcode & 0x0FFF;

        // Push the current PC on top of the stack
        self.stack[self.st_pointer as usize] = self.PC;
        // Increment the stack pointer
        self.st_pointer += 1;
        // Set the PC to the address
        self.PC = address;
    }

    // 3xkk - SE Vx, byte
    // Skip next instruction if Vx = kk
    //The interpreter compares register Vx to kk, and if they are equal, increments the program counter by 2.
    fn op_3xkk(&mut self, opcode: u16) {
        eprintln!("In OP_3XKK");

        // Mask the opcode to get the first 8 bits, which represent 'kk'
        let value: u16 = opcode & 0x00FF;
        // Bitshift to the right by 8 bits, then mask the first 4 bits, which represent 'x'
        let r_address: u16 = (opcode >> 8) & 0x0F;

        // Compare if Vx and kk are equal
        if self.registers[r_address as usize] == value as u8 {
            self.PC += 2;
        }
    }

    // 4xkk - SNE Vx, byte
    // Skip next instruction if Vx != kk
    // The interpreter compares register Vx to kk, and if they are not equal, increments the program counter by 2.
    fn op_4xkk(&mut self, opcode: u16) {
        eprintln!("In OP_4XKK");

        // Mask the opcode to get the first 8 bits, which represent 'kk'
        let value: u16 = opcode & 0x0FF;
        // Bitshift to the right by 8 bits, then mask the first 4 bits, which represent 'x'
        let r_address: u16 = (opcode >> 8) & 0x0F;

        // Check Vx and kk are not equal
        if self.registers[r_address as usize] != value as u8 {
            self.PC += 2;
        }
    }

    // 5xy0 - SE Vx, Vy
    // Skip next instruction if Vx = Vy.
    // The interpreter compares register Vx to register Vy, and if they are equal, increments the program counter by 2.
    fn op_5xy0(&mut self, opcode: u16) {
        eprintln!("In OP_5XY0");

        // Bitshift the opcode 4 bits to the right to remove the '0', then mask to get 0x00y
        let x: u16 = (opcode >> 4) & 0x00F;
        // Bitshift the opcode 8 bits to the right to remove the 'y0', then mask to get 0x0x
        let y: u16 = (opcode >> 8) & 0x0F;

        // Compare if Vx and Vy are equal
        if self.registers[x as usize] == self.registers[y as usize] {
            self.PC += 2;
        }
    }

    // 6xkk - LD Vx, byte
    // Set Vx = kk.
    // The interpreter puts the value kk into register Vx.
    fn op_6xkk(&mut self, opcode: u16) {
        eprintln!("In OP_6XKK");

        // Mask the opcode to get 0x00kk
        let value: u16 = opcode & 0x00FF;
        // Bitwise shift to the right by 8 bits, then mask to get 0x0x
        let r_address: u16 = (opcode >> 8) & 0x0F;
        // Load 'kk' into 'Vx'
        self.registers[r_address as usize] = value as u8;
    }

    // 7xkk - ADD Vx, byte
    // Set Vx = Vx + kk.
    // Adds the value kk to the value of register Vx, then stores the result in Vx.
    fn op_7xkk(&mut self, opcode: u16) {
        eprintln!("In OP_7XKK");

        // Mask the opcode to get 0x00kk
        let value: u16 = opcode & 0x00FF;
        // Bitwise shift to the right by 8 bits, then mask to get 0x0x
        let r_address: u16 = (opcode >> 8) & 0x0F;
        // Add 'kk' into 'Vx'
        self.registers[r_address as usize] += value as u8;
    }

    // 8xy0 - LD Vx, Vy
    // Set Vx = Vy.
    // Stores the value of register Vy in register Vx.
    fn op_8xy0(&mut self, opcode: u16) {
        eprintln!("In OP_8XY0");

        // Bitshift the opcode 4 bits to the right to remove the '0', then mask to get 0x00y
        let x: u16 = (opcode >> 4) & 0x00F;
        // Bitshift the opcode 8 bits to the right to remove the 'y0', then mask to get 0x0x
        let y: u16 = (opcode >> 8) & 0x0F;
        // Load the value inside 'Vy' onto 'Vx'
        self.registers[x as usize] = self.registers[y as usize];
    }

    // 8xy1 - OR Vx, Vy
    // Set Vx = Vx OR Vy.
    // Performs a bitwise OR on the values of Vx and Vy, then stores the result in Vx.
    fn op_8xy1(&mut self, opcode: u16) {
        eprintln!("In OP_8XY1");
        // Bitshift the opcode 4 bits to the right to remove the '0', then mask to get 0x00y
        let x: u16 = (opcode >> 4) & 0x00F;
        // Bitshift the opcode 8 bits to the right to remove the 'y0', then mask to get 0x0x
        let y: u16 = (opcode >> 8) & 0x0F;
        // Perform bitwise OR with the values inside 'Vx' and 'Vy'. Store back into 'Vx'
        self.registers[x as usize] = self.registers[x as usize] | self.registers[y as usize];
    }

    // 8xy2 - AND Vx, Vy
    // Set Vx = Vx AND Vy.
    // Performs a bitwise AND on the values of Vx and Vy, then stores the result in Vx.
    fn op_8xy2(&mut self, opcode: u16) {
        eprintln!("In OP_8XY2");
        // Bitshift the opcode 4 bits to the right to remove the '0', then mask to get 0x00y
        let x: u16 = (opcode >> 4) & 0x00F;
        // Bitshift the opcode 8 bits to the right to remove the 'y0', then mask to get 0x0x
        let y: u16 = (opcode >> 8) & 0x0F;
        // Perform bitwise AND with the values inside 'Vx' and 'Vy'. Store back into 'Vx'
        self.registers[x as usize] = self.registers[x as usize] & self.registers[y as usize];
    }

    // 8xy3 - XOR Vx, Vy
    // Set Vx = Vx XOR Vy.
    // Performs a bitwise exclusive OR on the values of Vx and Vy, then stores the result in Vx.
    fn op_8xy3(&mut self, opcode: u16) {
        eprintln!("In OP_8XY3");
        // Bitshift the opcode 4 bits to the right to remove the '0', then mask to get 0x00y
        let x: u16 = (opcode >> 4) & 0x00F;
        // Bitshift the opcode 8 bits to the right to remove the 'y0', then mask to get 0x0x
        let y: u16 = (opcode >> 8) & 0x0F;
        // Perform bitwise XOR with the values inside 'Vx' and 'Vy'. Store back into 'Vx'
        self.registers[x as usize] = self.registers[x as usize] ^ self.registers[y as usize];
    }

    // 8xy4 - ADD Vx, Vy
    // Set Vx = Vx + Vy, set VF = carry.
    // The values of Vx and Vy are added together.
    fn op_8xy4(&mut self, opcode: u16) {
        eprintln!("In OP_8XY4");
        // Bitshift the opcode 4 bits to the right to remove the '0', then mask to get 0x00y
        let x: u16 = (opcode >> 4) & 0x00F;
        // Bitshift the opcode 8 bits to the right to remove the 'y0', then mask to get 0x0x
        let y: u16 = (opcode >> 8) & 0x0F;
        // Receive the values from the respective registers
        let vx: u8 = self.registers[x as usize];
        let vy: u8 = self.registers[y as usize];

        // If the result is greater than 8 bits (i.e., > 255,) VF is set to 1, otherwise 0. Only the lowest 8 bits of the result are kept, and stored in Vx.
        let add_result = u8::checked_add(vx, vy);
        match add_result {
            Some(value) => self.registers[x as usize] = value,
            None => {
                self.registers[x as usize] = vx + vy;
                self.registers[15] = 1;
            }
        }
    }
    // 8xy5 - SUB Vx, Vy
    // Set Vx = Vx - Vy, set VF = NOT borrow.
    // The value of Vy is subtracted from Vx.
    fn op_8xy5(&mut self, opcode: u16) {
        eprintln!("In OP_8XY5");
        // Bitshift the opcode 4 bits to the right to remove the '0', then mask to get 0x00y
        let x: u16 = (opcode >> 4) & 0x00F;
        // Bitshift the opcode 8 bits to the right to remove the 'y0', then mask to get 0x0x
        let y: u16 = (opcode >> 8) & 0x0F;
        // Receive the values from the respective registers
        let vx: u8 = self.registers[x as usize];
        let vy: u8 = self.registers[y as usize];

        // If Vx > Vy, then VF is set to 1, otherwise 0. Then Vy is subtracted from Vx, and the results stored in Vx.
        self.registers[x as usize] = vx - vy;
        if vx > vy {
            self.registers[15] = 1;
        } else {
            self.registers[15] = 0;
        }
    }

    // 8xy6 - SHR Vx {, Vy}
    // Set Vx = Vx SHR 1.
    // fn op_8xy6(&mut self, opcode: u16) {
    //     // Bitshift the opcode 4 bits to the right to remove the '0', then mask to get 0x00y
    //     let x: u16 = (opcode >> 4) & 0x00F;
    //     // Bitshift the opcode 8 bits to the right to remove the 'y0', then mask to get 0x0x
    //     let y: u16 = (opcode >> 8) & 0x0F;
    // }

    // 8xy7 - SUBN Vx, Vy
    // Set Vx = Vy - Vx, set VF = NOT borrow.
    // The value of Vx is substracted from Vy.
    fn op_8xy7(&mut self, opcode: u16) {
        eprintln!("In OP_8XY7");
        // Bitshift the opcode 4 bits to the right to remove the '0', then mask to get 0x00y
        let x: u16 = (opcode >> 4) & 0x00F;
        // Bitshift the opcode 8 bits to the right to remove the 'y0', then mask to get 0x0x
        let y: u16 = (opcode >> 8) & 0x0F;
        // Receive the values from the respective registers
        let vx: u8 = self.registers[x as usize];
        let vy: u8 = self.registers[y as usize];

        // If Vy > Vx, then VF is set to 1, otherwise 0. Then Vx is subtracted from Vy, and the results stored in Vx.
        self.registers[x as usize] = vy - vx;
        if vy > vx {
            self.registers[15] = 1;
        } else {
            self.registers[15] = 0;
        }
    }

    // 8xyE - SHL Vx {, Vy}
    // Set Vx = Vx SHL 1.

    // 9xy0 - SNE Vx, Vy
    // Skip next instruction if Vx != Vy.
    fn op_9xy0(&mut self, opcode: u16) {
        eprintln!("In OP_9XY0");
        // Bitshift the opcode 4 bits to the right to remove the '0', then mask to get 0x00y
        let x: u16 = (opcode >> 4) & 0x00F;
        // Bitshift the opcode 8 bits to the right to remove the 'y0', then mask to get 0x0x
        let y: u16 = (opcode >> 8) & 0x0F;

        // Compare if Vx and Vy are not equal
        if self.registers[x as usize] != self.registers[y as usize] {
            self.PC += 2;
        }
    }

    // Annn - LD I, addr
    // Set I = nnn.
    fn op_annn(&mut self, opcode: u16) {
        eprintln!("In OP_ANNN");
        // The value of register I is set to nnn.
        self.IR = opcode & 0x0FFF;
    }

    // Bnnn - JP V0, addr
    // Jump to location nnn + V0.
    fn op_bnnn(&mut self, opcode: u16) {
        eprintln!("In OP_BNNN");
        // The program counter is set to nnn plus the value of V0.
        self.PC = self.registers[0] as u16 + (opcode & 0x0FFF);
    }

    // Cxkk - RND Vx, byte
    // Set Vx = random byte AND kk.
    fn op_cxkk(&mut self, opcode: u16) {
        eprintln!("In OP_CXKK");
        let value: u16 = opcode & 0x00FF;
        let r_address: u16 = (opcode >> 8) & 0x0F;
        // The interpreter generates a random number from 0 to 255, which is then ANDed with the value kk. The results are stored in Vx.
        self.registers[r_address as usize] = (rand::random::<u16>() & value) as u8;
    }

    //     Dxyn - DRW Vx, Vy, nibble
    // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.

    // The interpreter reads n bytes from memory, starting at the address stored in I. These bytes are then displayed as sprites on screen at coordinates (Vx, Vy). Sprites are XORed onto the existing screen. If this causes any pixels to be erased, VF is set to 1, otherwise it is set to 0. If the sprite is positioned so part of it is outside the coordinates of the display, it wraps around to the opposite side of the screen.
    fn op_dxyn(&mut self, opcode: u16) {
        eprintln!("In OP_DXYN");
        let height: u8 = (opcode & 0x000F) as u8;
        let vy: u16 = (opcode & 0x00F0) >> 4;
        let vx: u16 = (opcode & 0x0F00) >> 8;

        let x_pos: u8 = self.registers[vx as usize] % 64;
        let y_pos: u8 = self.registers[vy as usize] % 32;

        self.registers[0xF] = 0;

        for row in 0..height {
            let sprite_byte: u8 = self.memory[(self.IR + row as u16) as usize];

            for col in 0..8 {
                let sprite_pixel: u8 = sprite_byte & (0x80 >> col);
                // TODO Fix if it does not work
                if sprite_pixel == 1 {
                    if self.video[((y_pos + row) * 32 + (x_pos + col)) as usize] == 0xFF {
                        self.registers[0xF] = 1;
                    }
                    self.video[((y_pos + row) * 32 + (x_pos + col)) as usize] ^= 0xFF;
                }
            }
        }
    }

    fn op_null(&self) {
        return;
    }

    // TODO Finish all instructions
    fn exec(&mut self, opcode: u16) {
        eprintln!("In OPCODE EXECUTE STAGE; OPCODE: {:#x}", opcode);
        eprintln!("MATCHIN: {:#x}", (opcode & 0xF000) >> 12);
        match (opcode & 0xF000) >> 12 {
            0x0 => match opcode & 0x000F {
                0x0 => self.op_00e0(),
                0xE => self.op_00ee(),
                _ => self.op_null(),
            },
            0x1 => self.op_1nnn(opcode),
            0x2 => self.op_2nnn(opcode),
            0x3 => self.op_3xkk(opcode),
            0x4 => self.op_4xkk(opcode),
            0x5 => self.op_5xy0(opcode),
            0x6 => self.op_6xkk(opcode),
            0x7 => self.op_7xkk(opcode),
            0x8 => match opcode & 0x000F {
                0x0 => self.op_8xy0(opcode),
                0x1 => self.op_8xy1(opcode),
                0x2 => self.op_8xy2(opcode),
                0x3 => self.op_8xy3(opcode),
                0x4 => self.op_8xy4(opcode),
                0x5 => self.op_8xy5(opcode),
                // 0x6 => self.op_8xy6(opcode),
                0x7 => self.op_8xy7(opcode),
                // 0xE => self.op_8xyE(opcode),
                _ => self.op_null(),
            },
            0x9 => self.op_9xy0(opcode),
            0xA => self.op_annn(opcode),
            0xB => self.op_bnnn(opcode),
            0xC => self.op_cxkk(opcode),
            0xD => self.op_dxyn(opcode),
            // 0xE => match
            // 0xF => match
            _ => self.op_null(),
        }
    }

    fn cycle(&mut self) {
        let opcode: u16 = ((self.memory[self.PC as usize] as u16 | 0xFF00) << 8)
            | self.memory[(self.PC + 1) as usize] as u16;
        eprintln!("IN CYCLE STAGE; PC: {:#x} OPCODE: {:#x}", self.PC, opcode);

        self.PC += 2;

        self.exec(opcode);

        // if self.delayTimer > 0
        // {
        //     self.delayTimer -= 1;
        // }

        // if self.soundTimer > 0
        // {
        //     self.soundTimer -= 1;
        // }
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

    let mut window = Window::new(
        "CHIP8",
        640,
        320,
        WindowOptions {
            borderless: false,
            resize: true,
            scale: Scale::X8,
            topmost: true,
            ..WindowOptions::default()
        },
    )
    .unwrap();
    while window.is_open() && !window.is_key_down(Key::Escape) {
        chip8.cycle();
        window.update();
    }

    // for (i, byte) in chip8.memory.iter().enumerate() {
    //     if (*byte != 0) {
    //         println!("{:#x}: {:#x}", i, byte);
    //     }
    // }
}
