use std::sync::Mutex;

use egui_extras::Column;

use crate::{decoding::decode_instruction, gameboy::Gameboy};

pub struct EmulatorApp {
    console: Gameboy,
}

impl EmulatorApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>, console: Gameboy) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        return EmulatorApp { console };
    }
}

impl eframe::App for EmulatorApp {
    /// Called by the frame work to save state before shutdown.
    /*  fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    } */

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            // assembly view
            {
                let pc = self.console.cpu().read_program_counter();
                let mut pos = pc;
                let mut to_list = 5;
                while to_list > 0 {
                    let instr = decode_instruction(&self.console, pos);
                    ui.label(format!("{:#06X} | {}", pos, instr));
                    pos += instr.size;
                    to_list -= 1;
                }
            }

            // registers
            {
                let table = egui_extras::TableBuilder::new(ui)
                    .column(Column::auto())
                    .column(Column::auto())
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.heading("Register");
                        });
                        header.col(|ui| {
                            ui.heading("Value");
                        });
                    })
                    .body(|mut body| {
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.label("test1");
                            });
                            row.col(|ui| {
                                ui.label("test2");
                            });
                        })
                    });
            }
        });

        egui::TopBottomPanel::bottom(egui::Id::new("bottom pannel")).show(ctx, |ui| {
            if ui.button("Next").clicked() {
                self.console.step();
            }
        });
    }
}
