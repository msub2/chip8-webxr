mod chip8;
mod square_wave;

use std::collections::HashMap;

use pixels::{wgpu::Color, Pixels, SurfaceTexture};
use rfd::FileDialog;
use rodio::{source::Source, OutputStream, Sink};
use muda::{
    accelerator::{Accelerator, Code, Modifiers}, Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem, Submenu
};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::KeyCode,
    raw_window_handle::{HasWindowHandle, RawWindowHandle},
    window::{Window, WindowBuilder}
};
use winit_input_helper::WinitInputHelper;
use chip8::{Chip8, Variant};
use square_wave::SquareWave;

use fltk::{
    app, browser::Browser, prelude::*, window::Window as FLTKWindow
};

fn main() {
    // Set up winit and menubar
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(640, 320))
        .with_title("SILK-8")
        .build(&event_loop)
        .unwrap();
    let hwnd = match window.window_handle().unwrap().as_raw() {
        RawWindowHandle::Win32(handle) => handle.hwnd.get(),
        _ => panic!("Cannot handle other platform window handles yet!"),
    };
    let mut input = WinitInputHelper::new();
    let (menubar, menubar_items) = create_menubar();
    menubar.init_for_hwnd(hwnd).unwrap();
    let mut menubar_interaction = "";
    let mut windows: Vec<Window> = Vec::new();

    // Set up pixels and chip8
    let variant = Variant::XOCHIP;
    let mut chip8 = Chip8::new(variant);
    let mut rom_loaded = false;
    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        let initial_width = if variant == Variant::SCHIP_LEGACY { 128 } else { 64 };
        let initial_height = if variant == Variant::SCHIP_LEGACY { 64 } else { 32 };
        Pixels::new(initial_width, initial_height, surface_texture).unwrap()
    };
    pixels.clear_color(Color::BLACK);
    chip8.load_font();

    // Set up rodio
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    let source = SquareWave::new(440.0).amplify(0.10);
    sink.append(source);
    sink.pause();

    // Run the event loop
    event_loop.set_control_flow(ControlFlow::Poll);
    let _ = event_loop.run(move |event, elwt| {
        // Check for interactions on the menubar
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            let item_string = menubar_items.get(event.id()).unwrap().to_string();
            match item_string.as_str() {
                "Load ROM" => {
                    let file = FileDialog::new()
                        .add_filter("ROMs", &["ch8"])
                        .set_directory("./roms")
                        .pick_file();
                    if let Some(path) = file {
                        chip8.reset();
                        chip8.load_rom_from_file(path.to_str().unwrap());
                        rom_loaded = true;
                    }
                },
                "Quit" => {
                    elwt.exit();
                },
                "About" => {
                    create_about_window();
                }
                _ => {}
            }
        } else if menubar_interaction != "" {
            // I don't love this but it's conceptually easier than messing around
            // with the Windows API I'd have to interact with for accelerators
            match menubar_interaction {
                "Load ROM" => {
                    let file = FileDialog::new()
                        .add_filter("ROMs", &["ch8"])
                        .set_directory("./roms")
                        .pick_file();
                    if let Some(path) = file {
                        chip8.reset();
                        chip8.load_rom_from_file(path.to_str().unwrap());
                        rom_loaded = true;
                    }
                },
                _ => {}
            }
            menubar_interaction = "";
        }
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id: id,
            } => {
                for i in 0..windows.len() {
                    if windows[i].id() == id {
                        windows.remove(i);
                        break;
                    }
                }
                if id == window.id() {
                    elwt.exit();
                }
            },
            Event::AboutToWait => {
                if !rom_loaded {
                    let frame = pixels.frame_mut();
                    frame.fill(0);
                    if let Err(err) = pixels.render() {
                        println!("pixels.render() failed: {}", err);
                        elwt.exit();
                    }
                } else {
                    // Run the interpreter
                    for _ in 0..10 {
                        chip8.run();
                        if matches!(variant, Variant::CHIP8 | Variant::SCHIP_LEGACY) && chip8.displayed_this_frame() {
                            break;
                        }
                    }
                    chip8.decrement_timers();
    
                    // Resize buffer if we're in hi-res mode
                    if chip8.hires_mode() && pixels.frame().len() == 64 * 32 * 4 {
                        let _ = pixels.resize_buffer(128, 64);
                    } else if !chip8.hires_mode() && pixels.frame().len() == 128 * 64 * 4 {
                        let _ = pixels.resize_buffer(64, 32);
                    }
    
                    // Render the display
                    let display = chip8.get_display();
                    let frame = pixels.frame_mut();
                    for (pixel, &value) in frame.chunks_mut(4).zip(display.iter()) {
                        pixel.copy_from_slice(&[value * 255, value * 255, value * 255, 255]);
                    }
                    if let Err(err) = pixels.render() {
                        println!("pixels.render() failed: {}", err);
                        elwt.exit();
                    }
    
                    // Handle audio playback
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

            if input.held_control() && input.key_pressed(KeyCode::KeyO) {
                menubar_interaction = "Load ROM";
            }
        }
    });
}

fn create_menubar() -> (Menu, HashMap<MenuId, String>) {
    let menu = Menu::new();

    // File Tab
    let load_rom = MenuItem::new(
        "Load ROM",
        true,
        Some(Accelerator::new(Some(Modifiers::CONTROL), Code::KeyO)),
    );
    let quit = MenuItem::new(
        "Quit",
        true,
        None,
    );
    let file_tab = Submenu::with_items(
        "File",
        true,
        &[
            &load_rom,
            &PredefinedMenuItem::separator(),
            &quit,
        ],
    ).unwrap();
    menu.append(&file_tab).unwrap();

    // Help Tab
    let about = MenuItem::new(
        "About",
        true,
        None,
    );
    let help_tab = Submenu::with_items(
        "Help",
        true,
        &[
            &about,
        ],
    ).unwrap();
    menu.append(&help_tab).unwrap();

    let mut menu_ids = HashMap::new();
    menu_ids.insert(load_rom.id().clone(), "Load ROM".to_string());
    menu_ids.insert(quit.id().clone(), "Quit".to_string());
    menu_ids.insert(about.id().clone(), "About".to_string());

    (menu, menu_ids)
}

fn create_about_window() {
    let app = app::App::default();
    let mut window = FLTKWindow::new(0, 0, 400, 300, "About");
    let mut browser = Browser::new(0, 0, 400, 300, None);
    browser.set_frame(fltk::enums::FrameType::FlatBox);
    for _ in 0..8 { browser.add(""); }
    browser.add("@c Created by Daniel Adams");
    browser.add("@c Version 0.1.0");
    window.add(&browser);
    window.end();
    window.show();
    app.run().unwrap();
}