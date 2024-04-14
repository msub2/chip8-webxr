mod chip8;
mod square_wave;

use pixels::{Pixels, SurfaceTexture};
use rodio::{source::Source, OutputStream, Sink};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::KeyCode,
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;
use chip8::Chip8;
use square_wave::SquareWave;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(640, 320))
        .with_title("Chip8 Emulator")
        .build(&event_loop)
        .unwrap();
    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(64, 32, surface_texture).unwrap()
    };
    let mut input = WinitInputHelper::new();
    let mut chip8 = Chip8::new();
    chip8.load_font();
    chip8.load_rom_from_file("./roms/timendus/5-quirks.ch8");

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    // Add a dummy source of the sake of the example.
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
                for _ in 0..10 {
                    chip8.run();
                    if chip8.displayed_this_frame() {
                        break;
                    }
                }
                chip8.decrement_timers();
                let display = chip8.get_display();
                let frame = pixels.frame_mut();
                // The frame data is RGBA
                for i in 0..display.len() {
                    frame[i * 4] = display[i] * 255;
                    frame[i * 4 + 1] = display[i] * 255;
                    frame[i * 4 + 2] = display[i] * 255;
                    frame[i * 4 + 3] = 255;
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
            },
            _ => ()
        }

        if input.update(&event) {
            // Close events
            if input.key_pressed(KeyCode::Escape) || input.close_requested() {
                elwt.exit();
            }

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
                if input.key_pressed(key) {
                    chip8.set_keypad_state(value, true);
                } else if input.key_released(key) {
                    chip8.set_keypad_state(value, false);
                }
            }
        }
    });
}