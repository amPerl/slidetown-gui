use std::path::PathBuf;

use eframe::egui;
use slidetown::parsers::agt;
use slidetown::parsers::levelmodifier;

struct LevelModifierPlotCurve {
    pub curve: egui::plot::Curve,
    pub visible: bool,
    pub name: String,
}

struct LevelModifierPlot {
    pub curves: Vec<LevelModifierPlotCurve>,
    pub name: String,
    pub visible: bool,
}

impl std::fmt::Debug for LevelModifierPlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LevelModifierPlot {{ .. }}")
    }
}

#[derive(Debug)]
pub struct LevelModifierWindow {
    open: bool,
    title: String,
    path: PathBuf,
    agt_entry: agt::Entry,
    data: Option<levelmodifier::LevelModifier>,
    plots: Vec<LevelModifierPlot>,
}

impl LevelModifierWindow {
    pub fn from_agt_entry(path: PathBuf, agt_entry: agt::Entry) -> Self {
        Self {
            open: false,
            title: agt_entry.path.clone(),
            path,
            agt_entry,
            data: None,
            plots: Vec::new(),
        }
    }

    pub fn load(&mut self) {
        let npluto_key: &[u8] = &[
            0x01, 0x05, 0x06, 0x02, 0x04, 0x03, 0x07, 0x08, 0x01, 0x05, 0x06, 0x0F, 0x04, 0x03,
            0x07, 0x0C, 0x31, 0x85, 0x76, 0x39, 0x34, 0x3D, 0x30, 0xE8, 0x67, 0x36, 0x36, 0x32,
            0x3E, 0x33, 0x34, 0x3B, 0x11, 0x15, 0x16, 0x16, 0x14, 0x13, 0x1D, 0x18, 0x11, 0x03,
            0x06, 0x0C, 0x04, 0x03, 0x06, 0x08, 0x2E, 0x55, 0x26, 0x23, 0x2A, 0x23, 0x2E, 0x28,
            0x21, 0x21, 0x26, 0x27, 0x2E, 0x00, 0x2D, 0x2D, 0xCF, 0xA5, 0x06, 0x02, 0x04, 0x0F,
            0x07, 0x18, 0xE1, 0x15, 0x36, 0x18, 0x60, 0x13, 0x1A, 0x19, 0x11, 0x15, 0x16, 0x10,
            0x12, 0x13, 0x17, 0x38, 0xF1, 0x25,
        ];
        let mut agt_file = std::fs::File::open(&self.path).unwrap();
        let mut agt_reader = slidetown::parsers::agt::AgtReader::new(&mut agt_file, &npluto_key);

        let lm_bytes = agt_reader.read_entry(&self.agt_entry).unwrap();
        let mut lm_cursor = std::io::Cursor::new(lm_bytes);

        let lm = levelmodifier::LevelModifier::parse(&mut lm_cursor).unwrap();

        self.title = format!("{} ({})", self.agt_entry.path, lm.header.version_date);

        self.plots.push(LevelModifierPlot {
            name: "Speed".to_string(),
            curves: (0..lm.header.speed_length)
                .map(|g_idx| {
                    let g_idx = g_idx as usize;
                    let name = format!("{}", lm.header.speed_ids[g_idx]);
                    let curve = egui::plot::Curve::from_values_iter(
                        lm.speed.iter().enumerate().map(|(stat_val, go)| {
                            egui::plot::Value::new(stat_val as f64, go.values[g_idx])
                        }),
                    )
                    .name(name.clone());
                    LevelModifierPlotCurve {
                        curve,
                        visible: true,
                        name,
                    }
                })
                .collect(),
            visible: false,
        });

        self.plots.push(LevelModifierPlot {
            name: "Accel".to_string(),
            curves: (0..lm.header.accel_length)
                .map(|g_idx| {
                    let g_idx = g_idx as usize;
                    let name = format!("{}", lm.header.accel_ids[g_idx]);
                    let curve = egui::plot::Curve::from_values_iter(
                        lm.accel.iter().enumerate().map(|(stat_val, go)| {
                            egui::plot::Value::new(stat_val as f64, go.values[g_idx])
                        }),
                    )
                    .name(name.clone());
                    LevelModifierPlotCurve {
                        curve,
                        visible: true,
                        name,
                    }
                })
                .collect(),
            visible: false,
        });

        self.plots.push(LevelModifierPlot {
            name: "Dura".to_string(),
            curves: (0..lm.header.dura_length)
                .map(|g_idx| {
                    let g_idx = g_idx as usize;
                    let name = format!("{}", lm.header.dura_ids[g_idx]);
                    let curve = egui::plot::Curve::from_values_iter(
                        lm.dura.iter().enumerate().map(|(stat_val, go)| {
                            egui::plot::Value::new(stat_val as f64, go.values[g_idx])
                        }),
                    )
                    .name(name.clone());
                    LevelModifierPlotCurve {
                        curve,
                        visible: true,
                        name,
                    }
                })
                .collect(),
            visible: false,
        });

        self.plots.push(LevelModifierPlot {
            name: "Boost".to_string(),
            curves: (0..lm.header.boost_length)
                .map(|g_idx| {
                    let g_idx = g_idx as usize;
                    let name = format!("{}", lm.header.boost_ids[g_idx]);
                    let curve = egui::plot::Curve::from_values_iter(
                        lm.boost.iter().enumerate().map(|(stat_val, go)| {
                            egui::plot::Value::new(stat_val as f64, go.values[g_idx])
                        }),
                    )
                    .name(name.clone());
                    LevelModifierPlotCurve {
                        curve,
                        visible: true,
                        name,
                    }
                })
                .collect(),
            visible: false,
        });

        self.data = Some(lm);
    }

    pub fn display(&mut self, ctx: &egui::CtxRef, ui: &mut egui::Ui) {
        ui.checkbox(&mut self.open, &self.title);

        if !self.open {
            return;
        }

        if self.data.is_none() {
            self.load();
        }

        let data = self.data.as_mut().unwrap();
        let plots = &mut self.plots;

        egui::Window::new(&self.title)
            .open(&mut self.open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    for lm_plot in plots.iter_mut() {
                        ui.checkbox(&mut lm_plot.visible, &lm_plot.name);
                    }
                });
                for lm_plot in plots.iter_mut() {
                    if !lm_plot.visible {
                        continue;
                    }
                    ui.label(&lm_plot.name);

                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            let plot_curves = &mut lm_plot.curves;
                            for plot_curve in plot_curves {
                                ui.checkbox(&mut plot_curve.visible, &plot_curve.name);
                            }
                        });
                        let mut plot = egui::plot::Plot::default();
                        for curve in lm_plot.curves.iter() {
                            if !curve.visible {
                                continue;
                            }
                            plot = plot.curve(curve.curve.clone());
                        }
                        ui.add(plot.min_size(egui::vec2(320.0, 240.0)));
                    });
                }
            });
    }
}
