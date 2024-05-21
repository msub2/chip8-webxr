mod chip8;
mod square_wave;

use std::sync::{
  atomic::{AtomicBool, Ordering},
  Mutex
};

use eframe::egui;
use egui::Key;
use lazy_static::lazy_static;
use rodio::{source::Source, OutputStream, Sink};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use chip8::{Chip8, Variant};
use square_wave::SquareWave;

// I need to allow for ROMs to be loaded when called from a function outside the event loop
// and this was the best way I could think to do it. Also helps to forward keypad state when
// getting it from non-keyboard sources, like the in-world WebXR keypad, which winit can't get.
lazy_static! {
  static ref HAS_ROM: AtomicBool = AtomicBool::new(false);
  static ref ROM_CHANGED: AtomicBool = AtomicBool::new(false);
  static ref ROM_BYTES: Mutex<Vec<u8>> = Mutex::new(vec![]);
  static ref KEYPAD_STATE: Mutex<Vec<bool>> = Mutex::new(vec![false; 16]);
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
fn main() {
  // Redirect `log` message to `console.log` and friends:
  eframe::WebLogger::init(log::LevelFilter::Debug).ok();

  let web_options = eframe::WebOptions::default();

  // Set up rodio
  let (_stream, stream_handle) = OutputStream::try_default().unwrap();
  let sink = Sink::try_new(&stream_handle).unwrap();
  let source = SquareWave::new(440.0).amplify(0.10);
  sink.append(source);
  sink.pause();

  let mut chip8 = Chip8::new(Variant::XOCHIP);
  chip8.load_font();

  let silk8 = SILK8 {
      chip8,
      variant: Variant::XOCHIP,
      rom_loaded: false,
      sink,
  };

  wasm_bindgen_futures::spawn_local(async {
    eframe::WebRunner::new()
        .start(
            "chipCanvas", // hardcode it
            web_options,
            Box::new(|cc| Box::new(silk8)),
        )
        .await
        .expect("failed to start eframe");
});

}

struct SILK8 {
  chip8: Chip8,
  variant: Variant,
  rom_loaded: bool,

  sink: Sink,
}

impl eframe::App for SILK8 {
  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
      egui_extras::install_image_loaders(ctx);
      ctx.request_repaint();

      if !HAS_ROM.load(Ordering::Relaxed) {
        if ROM_CHANGED.load(Ordering::Relaxed) {
          ROM_CHANGED.store(false, Ordering::Relaxed);
          HAS_ROM.store(true, Ordering::Relaxed);
          self.chip8.load_rom_from_bytes(ROM_BYTES.lock().unwrap().to_owned());
          self.rom_loaded = true;
        } else {
          return;
        }
      }

      if self.rom_loaded {
          // Run the interpreter
          for _ in 0..10 {
              self.chip8.run();
              if matches!(self.variant, Variant::CHIP8 | Variant::SCHIP_LEGACY) && self.chip8.displayed_this_frame() {
                  break;
              }
          }
          self.chip8.decrement_timers();

          // Handle audio playback
          let sound_timer = self.chip8.get_sound_timer();
          if sound_timer > 0 && self.sink.is_paused() {
              self.sink.play();
          } else if sound_timer == 0 && !self.sink.is_paused() {
              self.sink.pause();
          }
      }

      // Render the display to a texture for egui
      let display_height = if self.chip8.hires_mode() { 64 } else { 32 };
      let display_width = if self.chip8.hires_mode() { 128 } else { 64 };
      let pixels = self.chip8.get_display()
          .iter()
          .map(|b| if *b == 1 { [255_u8, 255_u8, 255_u8] } else { [0_u8, 0_u8, 0_u8] })
          .flatten()
          .collect::<Vec<u8>>();
      let pixels_range = &pixels[0..display_height * display_width * 3];
      let color_image = egui::ColorImage::from_rgb([display_width, display_height], pixels_range);
      let handle = ctx.load_texture("Display", color_image, egui::TextureOptions::NEAREST);

      // Draw main window
      egui::CentralPanel::default().frame(egui::Frame::none()).show(ctx, |ui| {
          let sized_image = egui::load::SizedTexture::new(handle.id(), egui::vec2(640.0, 320.0));
          let image = egui::Image::from_texture(sized_image);
          ui.add(image);
      });

      // Handle input
      let state = KEYPAD_STATE.lock().unwrap().clone();
      for (key, value) in [
          (Key::Num1, 0),
          (Key::Num2, 1),
          (Key::Num3, 2),
          (Key::Num4, 3),
          (Key::Q, 4),
          (Key::W, 5),
          (Key::E, 6),
          (Key::R, 7),
          (Key::A, 8),
          (Key::S, 9),
          (Key::D, 10),
          (Key::F, 11),
          (Key::Z, 12),
          (Key::X, 13),
          (Key::C, 14),
          (Key::V, 15),
      ] {
          if ctx.input(|i| i.key_down(key)) || state[value as usize] {
              self.chip8.set_keypad_state(value, true);
          } else if ctx.input(|i| i.key_released(key)) || !state[value as usize] {
              self.chip8.set_keypad_state(value, false);
          }
      }
  }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn load_rom(bytes: Vec<u8>) {
  ROM_BYTES.lock().unwrap().clear();
  ROM_BYTES.lock().unwrap().extend_from_slice(&bytes);
  ROM_CHANGED.store(true, Ordering::Relaxed);
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn set_keypad_state(keypad: u8, state: bool) {
  KEYPAD_STATE.lock().unwrap()[keypad as usize] = state;
}