use camino::Utf8PathBuf;
use eframe::egui;
use slidetown::parsers::levelmodifier::LevelModifier;

use super::ProjectFileDialog;

#[derive(Debug, PartialEq)]
enum Stat {
    Accel,
    Speed,
    Dura,
    Boost,
}

impl Stat {
    fn name(&self) -> &'static str {
        match self {
            Stat::Accel => "Acceleration",
            Stat::Speed => "Speed",
            Stat::Dura => "Durability",
            Stat::Boost => "Boost",
        }
    }
}

#[derive(Debug)]
pub struct LevelmodifierFileDialog {
    path: Utf8PathBuf,
    data: LevelModifier,
    selected_stat: Stat,
}

fn get_value_id_name(id: u32) -> String {
    match id {
        2000 => "Normal Speed".into(),
        2001 => "Boost Speed".into(),
        2002 => "Reverse Speed".into(),

        3000 => "Acceleration Gain".into(),
        3001 => "Forward Angular Drag".into(),
        3002 => "Reverse Angular Drag".into(),
        3003 => "Drift Acceleration".into(),

        4014 => "Speed loss modifier (?)".into(),
        4015 => "Wall Type Crash Protection in Battlezone".into(),
        4016 => "Traffic Type Crash Protection in Battlezone".into(),
        4017 => "Nature Type Crash Protection in Battlezone".into(),
        4018 => "Unknown".into(),

        5000 => "Boost Time".into(),
        5001 => "Boost Strength".into(),
        5002 => "Boost Recharge Speed (sec)".into(),
        _ => id.to_string(),
    }
}

impl ProjectFileDialog for LevelmodifierFileDialog {
    fn create(path: Utf8PathBuf, _frame: &mut eframe::Frame) -> Self {
        let data_buf = std::fs::read(&path).unwrap();
        let data = LevelModifier::read(&mut std::io::Cursor::new(data_buf)).unwrap();
        Self {
            data,
            path,
            selected_stat: Stat::Accel,
        }
    }

    fn title(&self) -> String {
        self.path.file_name().unwrap().into()
    }

    fn show(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let Self {
            data,
            selected_stat,
            ..
        } = self;

        ui.horizontal_wrapped(|ui| {
            ui.selectable_value(selected_stat, Stat::Accel, Stat::Accel.name());
            ui.selectable_value(selected_stat, Stat::Speed, Stat::Speed.name());
            ui.selectable_value(selected_stat, Stat::Dura, Stat::Dura.name());
            ui.selectable_value(selected_stat, Stat::Boost, Stat::Boost.name());
        });

        let (value_sets, value_ids) = match selected_stat {
            Stat::Accel => (&mut data.accel, &mut data.accel_ids),
            Stat::Speed => (&mut data.speed, &mut data.speed_ids),
            Stat::Dura => (&mut data.dura, &mut data.dura_ids),
            Stat::Boost => (&mut data.boost, &mut data.boost_ids),
        };

        egui::plot::Plot::new(selected_stat.name())
            .data_aspect(10.0)
            .legend(egui::plot::Legend::default())
            .show(ui, |plot| {
                for (value_idx, value_id) in value_ids.iter().enumerate() {
                    let values = value_sets
                        .iter()
                        .enumerate()
                        .map(|(set_idx, set)| {
                            egui::plot::PlotPoint::new(set_idx as f64, set.values[value_idx] as f64)
                        })
                        .collect();
                    plot.line(
                        egui::plot::Line::new(egui::plot::PlotPoints::Owned(values))
                            .name(get_value_id_name(*value_id)),
                    )
                }
            });
    }
}
