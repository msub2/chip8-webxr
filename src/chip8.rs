use std::fs;
use std::path::Path;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Copy, Clone, Debug, PartialEq)]
/// Which particular CHIP-8 interpreter to emulate
pub enum Variant {
  CHIP8,
  SCHIP1_0,
  SCHIP1_1,
  XOCHIP
}

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
  keypad: [bool; 16],
  keypad_prev: [bool; 16],
  displayed: bool,
  variant: Variant,
}

#[wasm_bindgen]
impl Chip8 {
  /// Create a new Chip8 instance
  #[wasm_bindgen(constructor)]
  pub fn new(variant: Variant) -> Chip8 {
    Self {
      memory: [0; 4096],
      display: [0; 64 * 32],
      pc: 0x200,
      i: 0,
      stack: Vec::new(),
      delay_timer: 0,
      sound_timer: 0,
      registers: [0; 16],
      keypad: [false; 16],
      keypad_prev: [false; 16],
      displayed: false,
      variant,
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
    self.displayed = false;

    // Fetch the next 16-bit instruction
    let op1 = self.memory[self.pc as usize];
    let op2 = self.memory[(self.pc + 1) as usize];
    let op = ((op1 as u16) << 8) | (op2 as u16);
    self.pc += 2;

    // Decode and execute the instruction
    let op_1 = op & 0xF000;
    let op_2 = op & 0x0F00;
    let op_3 = op & 0x00F0;
    let op_4 = op & 0x000F;
    let x = ((op & 0x0F00) >> 8) as usize;
    let y = ((op & 0x00F0) >> 4) as usize;
    let nnn = op & 0x0FFF;
    let nn = (op & 0x00FF) as u8;
    let n = (op & 0x000F) as u8;

    match (op_1, op_2, op_3, op_4) {
      (0x0000, 0x0000, 0x00E0, 0x0000) => {
        // Clear the display
        self.display.fill(0);
      },
      (0x0000, 0x0000, 0x00E0, 0x000E) => {
        // Return from subroutine
        self.pc = match self.stack.pop() {
          Some(addr) => addr,
          None => {
            0x0200
          }
        }
      },
      (0x1000, _, _, _) => {
        // Jump to address NNN
        self.pc = nnn;
      },
      (0x2000, _, _, _) => {
        // Call subroutine at NNN
        self.stack.push(self.pc);
        self.pc = nnn;
      },
      (0x3000, _, _, _) => {
        // Skip next instruction if VX == NN
        if self.registers[x] == nn {
          self.pc += 2;
        }
      },
      (0x4000, _, _, _) => {
        // Skip next instruction if VX != NN
        if self.registers[x] != nn {
          self.pc += 2;
        }
      },
      (0x5000, _, _, _) => {
        // Skip next instruction if VX == VY
        if self.registers[x] == self.registers[y] {
          self.pc += 2;
        }
      },
      (0x6000, _, _, _) => {
        // Store number NN in register VX
        self.registers[x] = nn;
      },
      (0x7000, _, _, _) => {
        // Add number NN to register VX
        self.registers[x] = self.registers[x].wrapping_add(nn);
      },
      (0x8000, _, _, 0x0000) => {
        // Store value of register VY in register VX
        self.registers[x] = self.registers[y];
      },
      (0x8000, _, _, 0x0001) => {
        // Set register VX to VX | VY
        self.registers[x] = self.registers[x] | self.registers[y];
        if self.variant == Variant::CHIP8 {
          self.registers[0xF] = 0;
        }
      },
      (0x8000, _, _, 0x0002) => {
        // Set register VX to VX & VY
        self.registers[x] = self.registers[x] & self.registers[y];
        if self.variant == Variant::CHIP8 {
          self.registers[0xF] = 0;
        }
      },
      (0x8000, _, _, 0x0003) => {
        // Set register VX to VX ^ VY
        self.registers[x] = self.registers[x] ^ self.registers[y];
        if self.variant == Variant::CHIP8 {
          self.registers[0xF] = 0;
        }
      },
      (0x8000, _, _, 0x0004) => {
        // Add the value of register VY to register VX
        // Set VF to 01 if a carry occurs
        // Set VF to 00 if a carry does not occur
        let (result, overflow) = self.registers[x].overflowing_add(self.registers[y]);
        self.registers[x] = result;
        self.registers[0xF] = if overflow { 1 } else { 0 };
      },
      (0x8000, _, _, 0x0005) => {
        // Subtract the value of register VY from register VX
        // Set VF to 00 if a borrow occurs
        // Set VF to 01 if a borrow does not occur
        let (result, overflow) = self.registers[x].overflowing_sub(self.registers[y]);
        self.registers[x] = result;
        self.registers[0xF] = if overflow { 0 } else { 1 };
      },
      (0x8000, _, _, 0x0006) => {
        // Set register VX to VY >> 1
        // Set register VF to the least significant bit prior to the shift
        let lsb = self.registers[x] & 0x01;
        self.registers[x] = self.registers[y] >> 1;
        self.registers[0xF] = lsb;
      },
      (0x8000, _, _, 0x0007) => {
        // Set register VX to VY - VX
        // Set VF to 00 if a borrow occurs
        // Set VF to 01 if a borrow does not occur
        let (result, overflow) = self.registers[y].overflowing_sub(self.registers[x]);
        self.registers[x] = result;
        self.registers[0xF] = if overflow { 0 } else { 1 };
      },
      (0x8000, _, _, 0x000E) => {
        // Set register VX to VY << 1
        // Set register VF to the most significant bit prior to the shift
        let msb = (self.registers[x] & 0x80) >> 7;
        self.registers[x] = self.registers[y] << 1;
        self.registers[0xF] = msb;
      },
      (0x9000, _, _, _) => {
        // Skip next instruction if VX != VY
        if self.registers[x] != self.registers[y] {
          self.pc += 2;
        }
      },
      (0xA000, _, _, _) => {
        // Store address NNN in register I
        self.i = nnn;
      },
      (0xB000, _, _, _) => {
        // Jump to address NNN + V0
        self.pc = nnn + self.registers[0] as u16;
      },
      (0xC000, _, _, _) => {
        // Set VX to a random number with a mask of NN
        self.registers[x] = rand::random::<u8>() & nn;
      },
      (0xD000, _, _, _) => {
        // Draw sprite
        let x_val = self.registers[x] as u16;
        let y_val = self.registers[y] as u16;
        let height = n as u16;

        self.registers[0xF] = 0;

        for row in 0..height {
          if self.variant == Variant::CHIP8 && y_val + row == 32 {
            break;
          }
          let sprite = self.memory[(self.i + row) as usize];
          for column in 0..8 {
            if self.variant == Variant::CHIP8 && x_val + column == 64 {
              break;
            }
            // 0x80 is 0b10000000, this iterates through each bit
            if (sprite & (0x80 >> column)) != 0 {
              let pixel = (((x_val + column) % 64) + (((y_val + row) % 32) * 64)) as usize;
              if self.display[pixel] == 1 {
                self.registers[0xF] = 1;
              }
              self.display[pixel] ^= 1;
            }
          }
        }

        self.displayed = true;
      },
      (0xE000, _, 0x0090, 0x000E) => {
        // Skip next instruction if key stored in VX is pressed
        let index = self.get_keypad_index_from_value(self.registers[x]);
        if self.keypad[index] {
          self.pc += 2;
        }
      },
      (0xE000, _, 0x00A0, 0x0001) => {
        // Skip next instruction if key stored in VX is not pressed
        let index = self.get_keypad_index_from_value(self.registers[x]);
        if !self.keypad[index] {
          self.pc += 2;
        }
      },
      (0xF000, _, 0x0000, 0x0007) => {
        // Set VX to value of delay timer
        self.registers[x] = self.delay_timer;
      },
      (0xF000, _, 0x0000, 0x000A) => {
        // Wait for key press and store in VX
        if !self.keypad.iter().any(|key| *key) {
          self.pc -= 2;
        } else {
          // Store the first key in the keypad that is pressed
          let key_index = self.keypad.iter().position(|key| *key).unwrap();
          if !self.keypad[key_index] && self.keypad_prev[key_index] {
            self.registers[x] = self.get_keypad_value_from_index(key_index as u8);
          }
        }
      },
      (0xF000, _, 0x0010, 0x0005) => {
        // Set delay timer to VX
        self.delay_timer = self.registers[x];
      },
      (0xF000, _, 0x0010, 0x0008) => {
        // Set sound timer to VX
        self.sound_timer = self.registers[x];
      },
      (0xF000, _, 0x0010, 0x000E) => {
        // Add VX to I
        self.i = self.i.wrapping_add(self.registers[x] as u16);
      },
      (0xF000, _, 0x0020, 0x0009) => {
        // Set I to the memory address of the sprite data corresponding to the hex digit stored in register VX
        let digit = self.registers[x];
        self.i = 0x050 + (digit * 5) as u16;
      },
      (0xF000, _, 0x0030, 0x0003) => {
        // Store BCD representation of VX in memory locations I, I+1, and I+2
        self.memory[self.i as usize] = self.registers[x] / 100;
        self.memory[self.i as usize + 1] = (self.registers[x] / 10) % 10;
        self.memory[self.i as usize + 2] = (self.registers[x] % 100) % 10;
      },
      (0xF000, _, 0x0050, 0x0005) => {
        // Store the values of registers V0 to VX inclusive in memory starting at address I
        // I is set to I + X + 1 after operation
        for i in 0..(x + 1) {
          self.memory[self.i as usize + i] = self.registers[i];
        }
        self.i = self.i.wrapping_add(x as u16 + 1);
      },
      (0xF000, _, 0x0060, 0x0005) => {
        // Fill registers V0 to VX inclusive with the values stored in memory starting at address I
        // I is set to I + X + 1 after operation
        for i in 0..(x + 1) {
          self.registers[i] = self.memory[self.i as usize + i];
        }
        self.i = self.i.wrapping_add(x as u16 + 1);
      }
      _ => {
        println!("Unknown opcode: 0x{:04X}", op);
      }
    }

    // Update keypad states
    for i in 0..self.keypad.len() {
      self.keypad_prev[i] = self.keypad[i];
    }
  }

  pub fn set_keypad_state(&mut self, key_index: u8, value: bool) {
    self.keypad[key_index as usize] = value;
  }

  pub fn decrement_timers(&mut self) {
    if self.delay_timer > 0 {
      self.delay_timer -= 1;
    }
    if self.sound_timer > 0 {
      self.sound_timer -= 1;
    }
  }

  pub fn get_sound_timer(&self) -> u8 {
    self.sound_timer
  }

  pub fn displayed_this_frame(&self) -> bool {
    self.displayed
  }

  fn get_keypad_value_from_index(&self, key_index: u8) -> u8 {
    // TODO: Account for keypad layout variations
    // Assuming standard for now
    match key_index {
      0 => 0x1,
      1 => 0x2,
      2 => 0x3,
      3 => 0xC,
      4 => 0x4,
      5 => 0x5,
      6 => 0x6,
      7 => 0xD,
      8 => 0x7,
      9 => 0x8,
      10 => 0x9,
      11 => 0xE,
      12 => 0xA,
      13 => 0x0,
      14 => 0xB,
      15 => 0xF,
      _ => 0x0 // Shouldn't reach here
    }
  }

  fn get_keypad_index_from_value(&self, value: u8) -> usize {
    // TODO: Account for keypad layout variations
    // Assuming standard for now
    match value {
      0x1 => 0,
      0x2 => 1,
      0x3 => 2,
      0xC => 3,
      0x4 => 4,
      0x5 => 5,
      0x6 => 6,
      0xD => 7,
      0x7 => 8,
      0x8 => 9,
      0x9 => 10,
      0xE => 11,
      0xA => 12,
      0x0 => 13,
      0xB => 14,
      0xF => 15,
      _ => 0
    }
  }
}
