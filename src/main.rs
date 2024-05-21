mod chip8;
mod square_wave;

use std::collections::HashMap;

use eframe::egui;
use egui::Key;
use muda::{accelerator::{Accelerator, Code, Modifiers}, Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem, Submenu};
use rfd::FileDialog;
use rodio::{source::Source, OutputStream, Sink};
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

use chip8::{Chip8, Variant};
use square_wave::SquareWave;

fn main() -> Result<(), eframe::Error> {
    // Set window options, main important one here is min_inner_size so our window accounts for menubar insertion
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([640.0, 320.0]).with_min_inner_size([640.0, 320.0]),
        ..Default::default()
    };

    let mut chip8 = Chip8::new(Variant::XOCHIP);
    chip8.load_font();

    // Set up rodio
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    let source = SquareWave::new(440.0).amplify(0.10);
    sink.append(source);
    sink.pause();

    let silk8 = SILK8 {
        show_about_window: false,
        menubar: None,
        menubar_items: HashMap::new(),
        menubar_interaction: "".to_string(),
        chip8,
        variant: Variant::XOCHIP,
        rom_loaded: false,
        sink,
    };
    eframe::run_native(
        "SILK-8",
        options,
        Box::new(|_cc| Box::<SILK8>::new(silk8)),
    )
}

struct SILK8 {
    /// Immediate viewports are show immediately, so passing state to/from them is easy.
    /// The downside is that their painting is linked with the parent viewport:
    /// if either needs repainting, they are both repainted.
    show_about_window: bool,

    menubar: Option<Menu>,
    menubar_items: HashMap<MenuId, String>,
    menubar_interaction: String,

    chip8: Chip8,
    variant: Variant,
    rom_loaded: bool,

    sink: Sink,
}

impl eframe::App for SILK8 {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui_extras::install_image_loaders(ctx);
        ctx.request_repaint();

        // Check for interactions on the menubar
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            let item_string = self.menubar_items.get(event.id()).unwrap();
            match item_string.as_str() {
                "Load ROM" => {
                    let file = FileDialog::new()
                        .add_filter("ROMs", &["ch8"])
                        .set_directory("./roms")
                        .pick_file();
                    if let Some(path) = file {
                        self.chip8.reset();
                        self.chip8.load_rom_from_file(path.to_str().unwrap());
                        self.rom_loaded = true;
                    }
                },
                "Quit" => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                },
                "About" => {
                    self.show_about_window = true;
                }
                _ => {}
            }
        } else if self.menubar_interaction != "" {
            // I don't love this but it's conceptually easier than messing around
            // with the Windows API I'd have to interact with for accelerators
            match self.menubar_interaction.to_owned().as_str() {
                "Load ROM" => {
                    let file = FileDialog::new()
                        .add_filter("ROMs", &["ch8"])
                        .set_directory("./roms")
                        .pick_file();
                    if let Some(path) = file {
                        self.chip8.reset();
                        self.chip8.load_rom_from_file(path.to_str().unwrap());
                        self.rom_loaded = true;
                    }
                },
                _ => {}
            }
            self.menubar_interaction = "".to_string();
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
            if self.menubar.is_none() {
                let handle = _frame.window_handle().unwrap().as_raw();
                let hwnd = match handle {
                    RawWindowHandle::Win32(handle) => handle.hwnd.get(),
                    _ => panic!("Cannot handle other platform window handles yet!"),
                };
                let (menubar, menubar_items) = create_menubar();
                menubar.init_for_hwnd(hwnd).unwrap();
                self.menubar = Some(menubar);
                self.menubar_items = menubar_items;
            }

            let sized_image = egui::load::SizedTexture::new(handle.id(), egui::vec2(640.0, 320.0));
            let image = egui::Image::from_texture(sized_image);
            ui.add(image);
        });

        // Draw about window, if activve
        if self.show_about_window {
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("about_window"),
                egui::ViewportBuilder::default()
                    .with_title("About")
                    .with_inner_size([256.0, 128.0]),
                |ctx, class| {
                    assert!(
                        class == egui::ViewportClass::Immediate,
                        "This egui backend doesn't support multiple viewports"
                    );

                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label("Created by Daniel Adams");
                        })
                    });

                    if ctx.input(|i| i.viewport().close_requested()) {
                        // Tell parent viewport that we should not show next frame:
                        self.show_about_window = false;
                    }
                },
            );
        }

        // Handle input
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
            if ctx.input(|i| i.key_down(key)) {
                self.chip8.set_keypad_state(value, true);
            } else if ctx.input(|i| i.key_released(key)) {
                self.chip8.set_keypad_state(value, false);
            }
        }

        if ctx.input(|i| i.modifiers.ctrl) && ctx.input(|i| i.key_pressed(Key::O)) {
            self.menubar_interaction = "Load ROM".to_string();
        }
    }
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