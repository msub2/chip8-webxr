use std::fs;
use std::path::Path;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Chip8 {
  memory: [u8; 4096],
  display: [u8; 64 * 32],
  pc: u16,
  i: u16,
  stack: Vec<u16>,
  delay_timer: u8,
  sound_timer: u8,
  registers: [u8; 16],
}

#[wasm_bindgen]
impl Chip8 {
  /// Create a new Chip8 instance
  #[wasm_bindgen(constructor)]
  pub fn new() -> Chip8 {
    Self {
      memory: [0; 4096],
      display: [0; 64 * 32],
      pc: 0x200,
      i: 0,
      stack: Vec::new(),
      delay_timer: 0,
      sound_timer: 0,
      registers: [0; 16],
    }
  }

  /// Load the default font into memory at 0x0050
  pub fn load_font(&mut self) {
    let fontset: [u8; 80] = [
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
    let memory_slice = &mut self.memory[0x050..0x050 + fontset.len()];
    memory_slice.copy_from_slice(fontset.as_slice());
  }

  /// Load a ROM into memory at 0x0200 from a file
  pub fn load_rom_from_file(&mut self, rom: &str) {
    let bytes = fs::read(Path::new(rom)).expect("Failed to load ROM");
    let memory_slice = &mut self.memory[0x200..0x200 + bytes.len()];
    memory_slice.copy_from_slice(bytes.as_slice());
  }

  /// Load a ROM into memory at 0x0200 from a sequence of Uint8s
  pub fn load_rom_from_bytes(&mut self, bytes: Vec<u8>) {
    let memory_slice = &mut self.memory[0x200..0x200 + bytes.len()];
    memory_slice.copy_from_slice(bytes.as_slice());
  }

  /// Get screen pixel data as a sequence of Uint8s
  pub fn get_display(&self) -> Vec<u8> {
    Vec::from(&self.display)
  }

  /// Execute the next instruction at the program counter
  pub fn run(&mut self) {
    // Fetch the next 16-bit instruction
    let op1 = self.memory[self.pc as usize];
    let op2 = self.memory[(self.pc + 1) as usize];
    let op = ((op1 as u16) << 8) | (op2 as u16);
    self.pc += 2;

    // Decode and execute the instruction
    let leading_bit = op & 0xF000;
    match leading_bit {
      0x0000 => {
        if op == 0x00E0 {
          // Clear the display
          self.display.fill(0);
        } else if op == 0x00EE {
          // Return from subroutine
          self.pc = self.stack.pop().unwrap();
        } else {
          println!("Unknown opcode: 0x{:04X}", op);
        }
      },
      0x1000 => {
        // Jump to address NNN
        self.pc = op & 0x0FFF;
      },
      0x2000 => {
        // Call subroutine at NNN
        self.stack.push(self.pc);
        self.pc = op & 0x0FFF;
      },
      0x3000 => {
        // Skip next instruction if VX == NN
        let register = ((op & 0x0F00) >> 8) as usize;
        let value = (op & 0x00FF) as u8;
        if self.registers[register] == value {
          self.pc += 2;
        }
      },
      0x4000 => {
        // Skip next instruction if VX != NN
        let register = ((op & 0x0F00) >> 8) as usize;
        let value = (op & 0x00FF) as u8;
        if self.registers[register] != value {
          self.pc += 2;
        }
      },
      0x5000 => {
        // Skip next instruction if VX == VY
        let register1 = ((op & 0x0F00) >> 8) as usize;
        let register2 = ((op & 0x00F0) >> 4) as usize;
        println!("{}:{} == {}:{}", register1, self.registers[register1], register2, self.registers[register2]);
        if self.registers[register1] == self.registers[register2] {
          self.pc += 2;
        }
      },
      0x6000 => {
        // Store number NN in register VX
        let register = ((op & 0x0F00) >> 8) as usize;
        let value = (op & 0x00FF) as u8;
        self.registers[register] = value;
      },
      0x7000 => {
        // Add number NN to register VX
        let register = ((op & 0x0F00) >> 8) as usize;
        let value = (op & 0x00FF) as u8;
        self.registers[register] = self.registers[register].wrapping_add(value);
      },
      0x8000 => {
        let register1 = ((op & 0x0F00) >> 8) as usize;
        let register2 = ((op & 0x00F0) >> 4) as usize;
        let lsb = op & 0x000F;
        match lsb {
          0 => {
            // Store value of register VY in register VX
            self.registers[register1] = self.registers[register2];
          },
          1 => {
            // Set register VX to VX | VY
            self.registers[register1] = self.registers[register1] | self.registers[register2];
          },
          2 => {
            // Set register VX to VX & VY
            self.registers[register1] = self.registers[register1] & self.registers[register2];
          },
          3 => {
            // Set register VX to VX ^ VY
            self.registers[register1] = self.registers[register1] ^ self.registers[register2];
          },
          4 => {
            // Add the value of register VY to register VX
            // Set VF to 01 if a carry occurs
            // Set VF to 00 if a carry does not occur
            let (result, overflow) = self.registers[register1].overflowing_add(self.registers[register2]);
            self.registers[register1] = result;
            self.registers[0xF] = if overflow { 1 } else { 0 };
          },
          5 => {
            // Subtract the value of register VY from register VX
            // Set VF to 00 if a borrow occurs
            // Set VF to 01 if a borrow does not occur
            let (result, overflow) = self.registers[register1].overflowing_sub(self.registers[register2]);
            self.registers[register1] = result;
            self.registers[0xF] = if overflow { 0 } else { 1 };
          },
          6 => {
            // Set register VX to VY >> 1
            // Set register VF to the least significant bit prior to the shift
            let lsb = self.registers[register2] & 0x01;
            self.registers[register1] = self.registers[register2] >> 1;
            self.registers[0xF] = lsb;
          },
          7 => {
            // Set register VX to VY - VX
            // Set VF to 00 if a borrow occurs
            // Set VF to 01 if a borrow does not occur
            let (result, overflow) = self.registers[register2].overflowing_sub(self.registers[register1]);
            self.registers[register1] = result;
            self.registers[0xF] = if overflow { 0 } else { 1 };
          },
          0xE => {
            // Set register VX to VY << 1
            // Set register VF to the most significant bit prior to the shift
            let msb = (self.registers[register2] & 0x80) >> 7;
            self.registers[register1] = self.registers[register2] << 1;
            self.registers[0xF] = msb;
          }
          _ => {
            println!("Unknown opcode: 0x{:04X}", op);
          }
        }
      },
      0x9000 => {
        // Skip next instruction if VX != VY
        let register1 = ((op & 0x0F00) >> 8) as usize;
        let register2 = ((op & 0x00F0) >> 4) as usize;
        if self.registers[register1] != self.registers[register2] {
          self.pc += 2;
        }
      },
      0xA000 => {
        // Store address NNN in register I
        self.i = op & 0x0FFF;
      },
      0xB000 => {
        // Jump to address NNN + V0
        self.pc = (op & 0x0FFF) + self.registers[0] as u16;
      },
      0xC000 => {
        // Set VX to a random number with a mask of NN
        let register = ((op & 0x0F00) >> 8) as usize;
        let mask = (op & 0x00FF) as u8;
        self.registers[register] = rand::random::<u8>() & mask;
      },
      0xD000 => {
        // Draw sprite
        let x = self.registers[((op & 0x0F00) >> 8) as usize] as u16 % 64;
        let y = self.registers[((op & 0x00F0) >> 4) as usize] as u16 % 32;
        let height = op & 0x000F;

        self.registers[0xF] = 0;

        for row in 0..height {
          let sprite = self.memory[(self.i + row) as usize];
          for column in 0..8 {
            // 0x80 is 0b10000000, this iterates through each bit
            if (sprite & (0x80 >> column)) != 0 {
              if self.display[(x + column + (y + row) * 64) as usize] == 1 {
                self.registers[0xF] = 1;
              }
              self.display[(x + column + (y + row) * 64) as usize] ^= 1;
            }
          }
        }
      },
      0xE000 => {
        let lsb2 = op & 0x00FF;
        match lsb2 {
          0x9E => {
            // TODO: Skip next instruction if key stored in VX is pressed
          },
          0xA1 => {
            // TODO: Skip next instruction if key stored in VX is not pressed
          },
          _ => {
            println!("Unknown opcode: 0x{:04X}", op);
          }
        }
      },
      0xF000 => {
        let lsb2 = op & 0x00FF;
        match lsb2 {
          0x07 => {
            // Set VX to value of delay timer
            let register = ((op & 0x0F00) >> 8) as usize;
            self.registers[register] = self.delay_timer;
          },
          0x0A => {
            // TODO: Wait for key press and store in VX
          },
          0x15 => {
            // Set delay timer to VX
            let register = ((op & 0x0F00) >> 8) as usize;
            self.delay_timer = self.registers[register];
          },
          0x18 => {
            // Set sound timer to VX
            let register = ((op & 0x0F00) >> 8) as usize;
            self.sound_timer = self.registers[register];
          },
          0x1E => {
            // Add VX to I
            let register = ((op & 0x0F00) >> 8) as usize;
            self.i = self.i.wrapping_add(self.registers[register] as u16);
          },
          0x29 => {
            // Set I to the memory address of the sprite data corresponding to the hex digit stored in register VX
            let register = ((op & 0x0F00) >> 8) as usize;
            let digit = self.registers[register];
            self.i = 0x050 + (digit * 5) as u16;
          },
          0x33 => {
            // Store BCD representation of VX in memory locations I, I+1, and I+2
            let register = ((op & 0x0F00) >> 8) as usize;
            self.memory[self.i as usize] = self.registers[register] / 100;
            self.memory[self.i as usize + 1] = (self.registers[register] / 10) % 10;
            self.memory[self.i as usize + 2] = (self.registers[register] % 100) % 10;
          },
          0x55 => {
            // Store the values of registers V0 to VX inclusive in memory starting at address I
            // I is set to I + X + 1 after operation
            let register = ((op & 0x0F00) >> 8) as usize;
            for i in 0..(register + 1) {
              self.memory[self.i as usize + i] = self.registers[i];
            }
            self.i = self.i.wrapping_add(register as u16 + 1);
          },
          0x65 => {
            // Fill registers V0 to VX inclusive with the values stored in memory starting at address I
            // I is set to I + X + 1 after operation
            let register = ((op & 0x0F00) >> 8) as usize;
            for i in 0..(register + 1) {
              self.registers[i] = self.memory[self.i as usize + i];
            }
            self.i = self.i.wrapping_add(register as u16 + 1);
          },
          _ => {
            println!("Unknown opcode: 0x{:04X}", op);
          }
        }
      },
      _ => {
        println!("Unknown opcode: 0x{:04X}", op);
      }
    }
  }
}
