use pixels::{Pixels, SurfaceTexture};
use std::{
  rc::Rc,
  sync::{
    atomic::{AtomicBool, Ordering},
    Mutex
  }
};
use winit_input_helper::WinitInputHelper;
use lazy_static::lazy_static;

mod chip8;
mod square_wave;

use rodio::{source::Source, OutputStream, Sink};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::KeyCode,
    window::{Window, WindowBuilder},
};
use chip8::{Chip8, Variant};
use square_wave::SquareWave;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use web_time::{Instant, Duration};

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
fn main() {
  #[cfg(target_arch = "wasm32")]
  {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Warn).expect("error initializing logger");

    wasm_bindgen_futures::spawn_local(run());
  }
}

lazy_static! {
  static ref HAS_ROM: AtomicBool = AtomicBool::new(false);
  static ref ROM_CHANGED: AtomicBool = AtomicBool::new(false);
  static ref ROM_BYTES: Mutex<Vec<u8>> = Mutex::new(vec![]);
  static ref KEYPAD_STATE: Mutex<Vec<bool>> = Mutex::new(vec![false; 16]);
}

async fn run() {
  let event_loop = EventLoop::new().unwrap();
  let window = WindowBuilder::new()
      .with_inner_size(LogicalSize::new(640, 320))
      .with_title("SILK-8")
      .build(&event_loop)
      .unwrap();
  let mut input = WinitInputHelper::new();
  let window = Rc::new(window);
  let mut prev_frame_time = Instant::now();
  #[cfg(target_arch = "wasm32")]
  {
    use winit::platform::web::WindowExtWebSys;

    // Retrieve current width and height dimensions of browser client window
    let get_window_size = || {
      let client_window = web_sys::window().unwrap();
      LogicalSize::new(
        client_window.inner_width().unwrap().as_f64().unwrap(),
        client_window.inner_height().unwrap().as_f64().unwrap(),
      )
    };

    let window = Rc::clone(&window);

    // Initialize winit window with current dimensions of browser client
    window.set_min_inner_size(Some(get_window_size()));

    // Attach winit canvas to body element
    web_sys::window()
      .and_then(|win| win.document())
      .and_then(|doc| doc.body())
      .and_then(|body| {
        body.append_child(&web_sys::Element::from(window.canvas()?))
          .ok()
      })
      .expect("couldn't append canvas to document body");
  }
  let variant = Variant::XOCHIP;
  let mut chip8 = Chip8::new(variant);
  let mut pixels = {
      let surface_texture = SurfaceTexture::new(640, 320, &window);
      let initial_width = if variant == Variant::SCHIP_LEGACY { 128 } else { 64 };
      let initial_height = if variant == Variant::SCHIP_LEGACY { 64 } else { 32 };
      Pixels::new_async(initial_width, initial_height, surface_texture).await.unwrap()
  };
  chip8.load_font();

  let (_stream, stream_handle) = OutputStream::try_default().unwrap();
  let sink = Sink::try_new(&stream_handle).unwrap();
  let source = SquareWave::new(440.0).amplify(0.10);
  sink.append(source);

  event_loop.set_control_flow(ControlFlow::Poll);

  let _ = event_loop.run(move |event, elwt| {
    match event {
      Event::WindowEvent {
        event: WindowEvent::CloseRequested,
        ..
      } => {
        println!("The close button was pressed; stopping");
        elwt.exit();
      },
      Event::AboutToWait => {
        if !HAS_ROM.load(Ordering::Relaxed) {
          if ROM_CHANGED.load(Ordering::Relaxed) {
            chip8.load_rom_from_bytes(ROM_BYTES.lock().unwrap().to_owned());
            HAS_ROM.store(true, Ordering::Relaxed);
            ROM_CHANGED.store(false, Ordering::Relaxed);
          } else {
            return;
          }
        }

        if Instant::now() - prev_frame_time > Duration::from_millis(17) {
          prev_frame_time = Instant::now();
          for _ in 0..10 {
            chip8.run();
            if matches!(variant, Variant::CHIP8 | Variant::SCHIP_LEGACY) && chip8.displayed_this_frame() {
              break;
            }
          }
          chip8.decrement_timers();
          if chip8.hires_mode() && pixels.frame().len() == 64 * 32 * 4 {
            let _ = pixels.resize_buffer(128, 64);
          } else if !chip8.hires_mode() && pixels.frame().len() == 128 * 64 * 4 {
            let _ = pixels.resize_buffer(64, 32);
          }
          let display = chip8.get_display();
          let frame = pixels.frame_mut();
  
          for (pixel, &value) in frame.chunks_mut(4).zip(display.iter()) {
            pixel.copy_from_slice(&[value * 255, value * 255, value * 255, 255]);
          }
          if let Err(err) = pixels.render() {
            println!("pixels.render() failed: {}", err);
            elwt.exit();
          }
  
          let sound_timer = chip8.get_sound_timer();
          if sound_timer > 0 && sink.is_paused() {
            sink.play();
          } else if sound_timer == 0 && !sink.is_paused() {
            sink.pause();
          }
        }
      },
      _ => ()
    }

    if input.update(&event) {
        // Close events
        if input.key_pressed(KeyCode::Escape) || input.close_requested() {
          elwt.exit();
        }

        let state = KEYPAD_STATE.lock().unwrap().clone();

        for (key, value) in [
            (KeyCode::Digit1, 0),
            (KeyCode::Digit2, 1),
            (KeyCode::Digit3, 2),
            (KeyCode::Digit4, 3),
            (KeyCode::KeyQ, 4),
            (KeyCode::KeyW, 5),
            (KeyCode::KeyE, 6),
            (KeyCode::KeyR, 7),
            (KeyCode::KeyA, 8),
            (KeyCode::KeyS, 9),
            (KeyCode::KeyD, 10),
            (KeyCode::KeyF, 11),
            (KeyCode::KeyZ, 12),
            (KeyCode::KeyX, 13),
            (KeyCode::KeyC, 14),
            (KeyCode::KeyV, 15),
        ] {
            if input.key_pressed(key) || state[value as usize] {
              chip8.set_keypad_state(value, true);
            } else if input.key_released(key) || !state[value as usize] {
              chip8.set_keypad_state(value, false);
            }
        }
    }
  });
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