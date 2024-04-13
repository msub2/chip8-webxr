mod chip8;

use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::KeyCode,
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;
use chip8::Chip8;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(640, 320))
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
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                chip8.run();
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
                window.request_redraw();
            },
            _ => ()
        }

        if input.update(&event) {
            // Close events
            if input.key_pressed(KeyCode::Escape) || input.close_requested() {
                elwt.exit();
            }
        }
    });
}