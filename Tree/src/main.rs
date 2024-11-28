use sysinfo::{ProcessExt, System, SystemExt, PidExt};
use std::collections::HashMap;
use eframe::{self, egui};

struct TaskManager {
    system: sysinfo::System,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            system: System::new_all(),
        }
    }
}

impl eframe::App for TaskManager {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.system.refresh_all(); // Refresh system info

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Process Tree");

            //HashMap to store parent-child relationships
            let mut tree_map: HashMap<u32, Vec<u32>> = HashMap::new();
            for process in self.system.processes().values() {
                let parent_pid = process.parent().map_or(0, |p| p.as_u32());
                tree_map
                    .entry(parent_pid)
                    .or_default()
                    .push(process.pid().as_u32());
            }
            egui::ScrollArea::vertical().show(ui, |ui| {
                fn show_tree(
                    ui: &mut egui::Ui,
                    tree_map: &HashMap<u32, Vec<u32>>,
                    system: &sysinfo::System,
                    pid: u32,
                    depth: usize,
                ) {
                    if let Some(children) = tree_map.get(&pid) {
                        for &child_pid in children {
                            if let Some(child) = system.process(sysinfo::Pid::from_u32(child_pid)) {
                                let label = match depth {
                                    0 => "Parent:".to_string(),
                                    1 => "  Child:".to_string(),
                                    _ => format!("{:indent$}Child:", "", indent = depth * 2),
                                };
                                ui.label(format!(
                                    "{} PID: {} - Name: {}",
                                    label,
                                    child_pid,
                                    child.name()
                                ));

                                // Recursively display child processes
                                show_tree(ui, tree_map, system, child_pid, depth + 1);
                            }
                        }
                    }
                }
                show_tree(ui, &tree_map, &self.system, 0, 0);
            });
        });
    }
}



fn main() {
    let options = eframe::NativeOptions {
        vsync: true, 
        ..Default::default()
    };

    eframe::run_native(
        "Process Tree",
        options,
        Box::new(|_cc| Ok(Box::new(TaskManager::new()))),
    )
    .expect("Failed to start eframe app");
}
