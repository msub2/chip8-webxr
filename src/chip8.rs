use std::fs;
use std::path::Path;
use wasm_bindgen::prelude::*;

const STANDARD_LAYOUT: [u8; 16] = [
  0x1, 0x2, 0x3, 0xC,
  0x4, 0x5, 0x6, 0xD,
  0x7, 0x8, 0x9, 0xE,
  0xA, 0x0, 0xB, 0xF
];

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
  display: [u8; 128 * 64],
  pc: u16,
  i: u16,
  stack: Vec<u16>,
  delay_timer: u8,
  sound_timer: u8,
  registers: [u8; 16],
  keypad: [bool; 16],
  keypad_prev: [bool; 16],
  last_pressed_key: Option<usize>,
  displayed: bool,
  variant: Variant,
  hires_mode: bool,
  flags: [u8; 8],
}

#[wasm_bindgen]
impl Chip8 {
  /// Create a new Chip8 instance
  #[wasm_bindgen(constructor)]
  pub fn new(variant: Variant) -> Chip8 {
    Self {
      memory: [0; 4096],
      display: [0; 128 * 64],
      pc: 0x200,
      i: 0,
      stack: Vec::new(),
      delay_timer: 0,
      sound_timer: 0,
      registers: [0; 16],
      keypad: [false; 16],
      keypad_prev: [false; 16],
      last_pressed_key: None,
      displayed: false,
      variant,
      // SCHIP
      hires_mode: false,
      flags: [0; 8],
      // XOCHIP
    }
  }

  /// Load the default font into memory at 0x0050
  pub fn load_font(&mut self) {
    let lores_fontset: [u8; 5 * 16] = [
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
    let hires_fontset: [u8; 10 * 16] = [
      0xff, 0xff, 0xc3, 0xc3, 0xc3, 0xc3, 0xc3, 0xc3, 0xff, 0xff, // 0
      0x18, 0x78, 0x78, 0x18, 0x18, 0x18, 0x18, 0x18, 0xff, 0xff, // 1
      0xff, 0xff, 0x03, 0x03, 0xff, 0xff, 0xc0, 0xc0, 0xff, 0xff, // 2
      0xff, 0xff, 0x03, 0x03, 0xff, 0xff, 0x03, 0x03, 0xff, 0xff, // 3
      0xc3, 0xc3, 0xc3, 0xc3, 0xff, 0xff, 0x03, 0x03, 0x03, 0x03, // 4
      0xff, 0xff, 0xc0, 0xc0, 0xff, 0xff, 0x03, 0x03, 0xff, 0xff, // 5
      0xff, 0xff, 0xc0, 0xc0, 0xff, 0xff, 0xc3, 0xc3, 0xff, 0xff, // 6
      0xff, 0xff, 0x03, 0x03, 0x06, 0x0c, 0x18, 0x18, 0x18, 0x18, // 7
      0xff, 0xff, 0xc3, 0xc3, 0xff, 0xff, 0xc3, 0xc3, 0xff, 0xff, // 8
      0xff, 0xff, 0xc3, 0xc3, 0xff, 0xff, 0x03, 0x03, 0xff, 0xff, // 9
      0x7e, 0xff, 0xc3, 0xc3, 0xc3, 0xff, 0xff, 0xc3, 0xc3, 0xc3, // A
      0xfc, 0xfc, 0xc3, 0xc3, 0xfc, 0xfc, 0xc3, 0xc3, 0xfc, 0xfc, // B
      0x3c, 0xff, 0xc3, 0xc0, 0xc0, 0xc0, 0xc0, 0xc3, 0xff, 0x3c, // C
      0xfc, 0xfe, 0xc3, 0xc3, 0xc3, 0xc3, 0xc3, 0xc3, 0xfe, 0xfc, // D
      0xff, 0xff, 0xc0, 0xc0, 0xff, 0xff, 0xc0, 0xc0, 0xff, 0xff, // E
      0xff, 0xff, 0xc0, 0xc0, 0xff, 0xff, 0xc0, 0xc0, 0xc0, 0xc0  // F
    ];

    let memory_slice = &mut self.memory[0x00..0xf0];
    for i in 0..lores_fontset.len() {
      memory_slice[i] = lores_fontset[i];
    }
    for i in 0..hires_fontset.len() {
      memory_slice[i + 0x50] = hires_fontset[i];
    }
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
    if self.pc as usize >= self.memory.len() {
      // Probably not accurate to real life but just set back to start of program
      self.pc = 0x0200;
    }
    let op1 = self.memory[self.pc as usize];
    let op2 = self.memory[(self.pc + 1) as usize];
    let op = ((op1 as u16) << 8) | (op2 as u16);
    self.pc = self.pc.wrapping_add(2);

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
      (0x0000, 0x0000, 0x00C0, _) => {
        // SCHIP: Scroll the display down by N pixels
        let max_cols = self.max_cols();

        self.display.rotate_right(n as usize * max_cols as usize);
        self.display[..n as usize * max_cols as usize].fill(0);
      },
      (0x0000, 0x0000, 0x00E0, 0x0000) => {
        // Clear the display
        self.display.fill(0);
      },
      (0x0000, 0x0000, 0x00E0, 0x000E) => {
        // Return from subroutine
        self.pc = match self.stack.pop() {
          Some(addr) => addr,
          None => {
            // Probably not accurate to real life but just set back to start of program
            0x0200
          }
        }
      },
      (0x0000, 0x0000, 0x00F0, 0x000B) => {
        // SCHIP: Scroll the display right by 4 pixels
        let max_rows = self.max_rows();
        let max_cols = self.max_cols();

        for row in 0..max_rows {
          for col in (0..max_cols).rev() {
            if col > 3 {
              self.display[(row * max_cols) + col] = self.display[(row * max_cols) + col - 4];
            } else {
              self.display[(row * max_cols) + col] = 0;
            }
          }
        }
      },
      (0x0000, 0x0000, 0x00F0, 0x000C) => {
        // SCHIP: Scroll the display left by 4 pixels
        let max_rows = self.max_rows();
        let max_cols = self.max_cols();

        // Similar logic as above step, but for left instead of right
        for row in 0..max_rows {
          for col in 0..max_cols {
            if col < max_cols - 4 {
              self.display[(row * max_cols) + col] = self.display[(row * max_cols) + col + 4];
            } else {
              self.display[(row * max_cols) + col] = 0;
            }
          }
        }
      },
      (0x0000, 0x0000, 0x00F0, 0x000E) => {
        // SCHIP: Use lores mode
        self.hires_mode = false;
      },
      (0x0000, 0x0000, 0x00F0, 0x000F) => {
        // SCHIP: Use hires mode
        self.hires_mode = true;
      }
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
        let lsb = self.registers[x] & 0x01;
        if self.variant == Variant::CHIP8 {
          // Set register VX to VY >> 1
          // Set register VF to the least significant bit prior to the shift
          self.registers[x] = self.registers[y] >> 1;
        } else if self.variant == Variant::SCHIP1_0 {
          self.registers[x] >>= 1;
        }
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
        let msb = (self.registers[x] & 0x80) >> 7;
        if self.variant == Variant::CHIP8 {
          // Set register VX to VY << 1
          // Set register VF to the most significant bit prior to the shift
          self.registers[x] = self.registers[y] << 1;
        } else if self.variant == Variant::SCHIP1_0 {
          self.registers[x] <<= 1;
        }

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
        if self.variant == Variant::CHIP8 {
          // Jump to address NNN + V0
          self.pc = nnn + self.registers[0] as u16;
        } else if self.variant == Variant::SCHIP1_0 {
          // Jump to address XNN + VX
          self.pc = nnn + self.registers[x] as u16;
        }
      },
      (0xC000, _, _, _) => {
        // Set VX to a random number with a mask of NN
        self.registers[x] = rand::random::<u8>() & nn;
      },
      (0xD000, _, _, _) => {
        // Draw sprite
        // The x coordinate to begin drawing at
        let x_val = self.registers[x] as u16;
        // The y coordinate to begin drawing at
        let y_val = self.registers[y] as u16;
        // The width of the sprite (16 if SCHIP and N = 0, otherwise 8)
        let width = if n == 0 && self.variant == Variant::SCHIP1_0 { 16_u16 } else { 8_u16 };
        // The height of the sprite (16 if SCHIP and N = 0, otherwise N)
        let height = if n == 0 && self.variant == Variant::SCHIP1_0 { 16_u16 } else { n as u16 };
        // The maximum width od the display
        let max_width = if self.hires_mode { 128_u16 } else { 64_u16 };
        // The maximum height of the display
        let max_height = if self.hires_mode { 64_u16 } else { 32_u16 };

        self.registers[0xF] = 0;

        // Start iterating through the rows of the sprite
        for row in 0..height {
          if matches!(self.variant, Variant::CHIP8 | Variant::SCHIP1_0) && y_val + row == max_height {
            break;
          }
          // Start iterating through the bytes in the row
          for column in 0..width {
            if matches!(self.variant, Variant::CHIP8 | Variant::SCHIP1_0) && x_val + column == max_width {
              break;
            }
            let scale_factor = if self.hires_mode && n == 0 { 2 } else { 1 };
            // An offset to the next byte to apply in case we are drawing a 16x16 sprite
            let offset = if column > 7 { row * scale_factor + 1 } else { row * scale_factor };
            // The location of the sprite in memory.
            let sprite = self.memory[(self.i + offset) as usize % self.memory.len()];
            let pixel_x = (x_val + column) % max_width;
            let pixel_y = (y_val + row) % max_height;
            // 0x80 is 0b10000000, this iterates through each bit
            if (sprite & (0x80 >> (column % 8))) != 0 {
              let pixel = (pixel_x + pixel_y * max_width) as usize;
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
        let index = self.get_keypad_index_from_value(self.registers[x] & 0xF);
        if self.keypad[index] {
          self.pc = self.pc.wrapping_add(2);
        }
      },
      (0xE000, _, 0x00A0, 0x0001) => {
        // Skip next instruction if key stored in VX is not pressed
        let index = self.get_keypad_index_from_value(self.registers[x] & 0xF);
        if !self.keypad[index] {
          self.pc = self.pc.wrapping_add(2);
        }
      },
      (0xF000, _, 0x0000, 0x0007) => {
        // Set VX to value of delay timer
        self.registers[x] = self.delay_timer;
      },
      (0xF000, _, 0x0000, 0x000A) => {
        // Wait for key press and store in VX
        if !self.keypad_prev.iter().any(|key| *key) {
          self.pc = self.pc.wrapping_sub(2);
        } else {
          // Store the first key in the keypad that is pressed and wait for it to be released
          let key_index = if self.last_pressed_key.is_some() { self.last_pressed_key.unwrap() } else {
            let index = self.keypad_prev.iter().position(|key| *key).unwrap();
            self.last_pressed_key = Some(index);
            index
          };

          if !self.keypad[key_index] && self.keypad_prev[key_index] {
            self.registers[x] = self.get_keypad_value_from_index(key_index as u8);
          } else {
            self.pc = self.pc.wrapping_sub(2);
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
        let digit = self.registers[x] & 0x0F;
        self.i = 0x000 + (digit * 5) as u16;
      },
      (0xF000, _, 0x0030, 0x0000) => {
        // Set I to the memory address of the sprite data corresponding to the big hex digit stored in register VX
        let digit = self.registers[x] & 0x0F;
        self.i = 0x050 + (digit * 5) as u16;
      }
      (0xF000, _, 0x0030, 0x0003) => {
        // Store BCD representation of VX in memory locations I, I+1, and I+2
        self.memory[self.i as usize % self.memory.len()] = self.registers[x] / 100;
        self.memory[(self.i as usize + 1) % self.memory.len()] = (self.registers[x] / 10) % 10;
        self.memory[(self.i as usize + 2) % self.memory.len()] = (self.registers[x] % 100) % 10;
      },
      (0xF000, _, 0x0050, 0x0005) => {
        // Store the values of registers V0 to VX inclusive in memory starting at address I
        // I is set to I + X + 1 after operation
        for i in 0..(x + 1) {
          self.memory[(self.i as usize + i) % self.memory.len()] = self.registers[i];
        }
        if self.variant == Variant::CHIP8 {
          self.i = self.i.wrapping_add(x as u16 + 1);
        }
      },
      (0xF000, _, 0x0060, 0x0005) => {
        // Fill registers V0 to VX inclusive with the values stored in memory starting at address I
        // I is set to I + X + 1 after operation
        for i in 0..(x + 1) {
          self.registers[i] = self.memory[(self.i as usize + i) % self.memory.len()];
        }
        if self.variant == Variant::CHIP8 {
          self.i = self.i.wrapping_add(x as u16 + 1);
        }
      },
      (0xF000, _, 0x0070, 0x0005) => {
        // Store the values of registers V0 to VX inclusive in user flags
        for i in 0..(x + 1) {
          self.flags[i] = self.registers[i];
        }
      },
      (0xF000, _, 0x0080, 0x0005) => {
        // Fill registers V0 to VX inclusive with the values stored in user flags
        for i in 0..(x + 1) {
          self.registers[i] = self.flags[i];
        }
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

  pub fn hires_mode(&self) -> bool {
    self.hires_mode
  }

  fn get_keypad_value_from_index(&self, key_index: u8) -> u8 {
    // TODO: Account for keypad layout variations
    // Assuming standard for now
    STANDARD_LAYOUT.get(key_index as usize).unwrap_or(&0).clone()
  }

  fn get_keypad_index_from_value(&self, value: u8) -> usize {
    // TODO: Account for keypad layout variations
    // Assuming standard for now
    STANDARD_LAYOUT.iter().position(|x| *x == value).unwrap_or(0)
  }

  fn max_rows(&self) -> usize {
    if self.hires_mode { 64 } else { 32 }
  }

  fn max_cols(&self) -> usize {
    if self.hires_mode { 128 } else { 64 }
  }
}
