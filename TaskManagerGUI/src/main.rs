use sysinfo::{System, ProcessStatus};
use std::{collections::HashMap, time::Duration, io::{self, Write}};
use std::time::{Instant};
use clearscreen; // clear terminal screen
use crossterm::{event, terminal};
use nix::sys::signal::{self, Signal}; // For sending signals like SIGSTOP/SIGCONT
use nix::unistd::Pid; // For working with PIDs
use eframe::{self, egui};

//this is a struct that does some UI stuff that I never understood, it is used in 3 parts, this is the first
impl Default for TaskManager {
    fn default() -> Self {
        TaskManager {
            last_update: Instant::now(),
            refresh_interval: Duration::from_secs(1),
            system: System::new_all(),
            sort_criteria: SortCriteria::Memory,
            reverse_sort: false,
        }
    }
}
//used to determine sort style
#[derive(PartialEq)]//this is an attribute it can be derived from so that it allows comparisons (If sort_crit==mem)
enum SortCriteria {
    Memory,
    CPU,
}

struct TaskManager {
    last_update: Instant,
    refresh_interval: Duration,
    system: sysinfo::System, //default value
    sort_criteria: SortCriteria,
    reverse_sort: bool,//ASC or DEC
}

fn get_total_memory_mb(system: &sysinfo::System) -> f32 {
    system.total_memory() as f32 / 1024.0 // Convert from KB to MB
}

fn main() {

    startGUI(); 
}

fn startGUI(){
    //open the gui with name Task Manager
    eframe::run_native(
        "Task Manager",
        eframe::NativeOptions {
            drag_and_drop_support: true,
            maximized: true,
            initial_window_size: Some(egui::vec2(800.0, 600.0)), //this determines starting resolution
            ..Default::default()
        },
        Box::new(|_cc| Box::<TaskManager>::default()),//second time the struct is used, this is related to memory and how gui is stored
    );
}


impl TaskManager {
    pub fn new() -> Self {
        Self {
            last_update: Instant::now(),
            refresh_interval: Duration::from_millis(100),
            system: System::new_all(),
            sort_criteria: SortCriteria::Memory,
            reverse_sort: false,
        }
    }
}

impl eframe::App for TaskManager { //this is 3rd time struct is used

    // update here is a special function that is called automatically every frame
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        
        let now = Instant::now();

        // Refresh system info only if 0.1 seconds have passed
        if now.duration_since(self.last_update) >= self.refresh_interval {
            self.system.refresh_all();
            self.last_update = now;
        }

        // Request a repaint
        ctx.request_repaint();
    
        egui::CentralPanel::default().show(ctx, |ui| {
            //some vertical space
            ui.allocate_space(egui::vec2(0.0, 20.0));
            ui.end_row();

            ui.vertical_centered_justified(|ui| {//this centers the text and at the top of the screen
            //here we create a heading element, this element has text
                ui.heading(
                    egui::RichText::new("Task Manager")//modify text here
                    .size(50.0) //text size
                    .color(egui::Color32::WHITE) //test color
                    .strong(), //make it bold
                );
            });

            ui.allocate_space(egui::vec2(0.0, 20.0));
            //display a sorting text to make sure the user knows what we are sorting by instead of guessing
            let sorting_text = format!(
                "Sorting by: {} ({})",
                match self.sort_criteria {
                    SortCriteria::Memory => "Memory",
                    SortCriteria::CPU => "CPU",
                },
                if self.reverse_sort { "ASC" } else { "DESC" }
            );            
            ui.label(
                egui::RichText::new(sorting_text)
                    .color(egui::Color32::LIGHT_BLUE)
                    .size(20.0),
            );
            //some vertical space
            ui.allocate_space(egui::vec2(0.0, 40.0));
            ui.end_row();
            // Create a scrollable area for displaying processes
            egui::ScrollArea::vertical().show(ui, |ui| { // Use `vertical()` for vertical scrolling
                // Create a table layout to show processes
                egui::Grid::new("process_grid").show(ui, |ui| {

                    ui.label(//this creates a UI label with text PID, color white, and size 18
                        egui::RichText::new("PID")
                            .color(egui::Color32::WHITE)
                            .size(18.0)
                    );
                    ui.allocate_space(egui::vec2(20.0, 0.0));//this creates a space, as 2d vector where
                    //20 is the horizental value and 0 is the vertical one it creates only hroizental space
                    ui.label(
                        egui::RichText::new("Name")
                            .color(egui::Color32::WHITE)
                            .size(18.0),
                    );
                    ui.allocate_space(egui::vec2(30.0, 0.0));
                    //create a button, same setup as label but it has clicked event which  decides what happens 
                    //once it is clicked, here we change taskManager struct reverse_sort and sort criteria 
                    if ui.button(
                        egui::RichText::new("Memory (MB)")
                            .color(egui::Color32::WHITE)
                            .size(18.0),).clicked() {
                        if let SortCriteria::Memory = self.sort_criteria {
                            self.reverse_sort = !self.reverse_sort; // Reverse order
                        } else {
                            self.sort_criteria = SortCriteria::Memory;
                            self.reverse_sort = false; // Reset order
                        }
                    }
                    ui.allocate_space(egui::vec2(20.0, 0.0));
                    if ui.button( 
                        egui::RichText::new("CPU Usage (%)")
                    .color(egui::Color32::WHITE)
                    .size(18.0),).clicked()
                    {
                        if let SortCriteria::CPU = self.sort_criteria {
                            self.reverse_sort = !self.reverse_sort;
                        } else {
                            self.sort_criteria = SortCriteria::CPU;
                            self.reverse_sort = false;
                        }
                    }
                    ui.allocate_space(egui::vec2(20.0, 0.0));
                    ui.label(
                        egui::RichText::new("Status")
                            .color(egui::Color32::WHITE)
                            .size(18.0),
                    );
                    ui.allocate_space(egui::vec2(20.0, 0.0));
                    ui.end_row();//this creates a new row
                    ui.allocate_space(egui::vec2(0.0, 20.0));
                    ui.end_row();


                    //collect and sort processes by memory
                    let mut aggregated_processes: HashMap<String, (u64, f32, Option<u32>, Option<ProcessStatus>)> = HashMap::new();
                    for process in self.system.processes().values() {
                        if process.memory() > 0 {
                            let entry = aggregated_processes
                                .entry(process.name().to_string_lossy().to_string())
                                .or_insert((0, 0.0, None, None));
                            entry.0 += process.memory(); // Sum memory usage
                            entry.1 += process.cpu_usage(); // Sum CPU usage
                    
                            if entry.2.is_none() {
                                entry.2 = Some(process.pid().as_u32());
                            }
                    
                            if entry.3.is_none() {
                                entry.3 = Some(process.status());
                            }
                        }
                    }
                    
                    // Sort the aggregated data by memory
                    let mut sorted_processes: Vec<_> = aggregated_processes.into_iter().collect();
                    match self.sort_criteria {
                        SortCriteria::Memory => {
                            sorted_processes.sort_by(|a, b| b.1 .0.cmp(&a.1 .0));
                        }
                        SortCriteria::CPU => {
                            sorted_processes.sort_by(|a, b| b.1.1.partial_cmp(&a.1.1).unwrap());
                        }
                    }
                    if self.reverse_sort {
                        sorted_processes.reverse();
                    }
                    let total_memory = get_total_memory_mb(&self.system) * 1024.0 * 1024.0;
                    let num_cores = self.system.cpus().len() as f32;
                    // Display sorted processes in the table by looping over them one by one
                    for (name, (memory, cpu, pid, status)) in sorted_processes {
                        let normalized_cpu = cpu / num_cores;
                        ui.label(
                            egui::RichText::new(pid.map_or("Unknown".to_string(), |v| v.to_string()))
                                    .color(egui::Color32::WHITE)
                                    .size(15.0),
                            );//here it creates a label and displays pid in it
                        ui.allocate_space(egui::vec2(20.0, 0.0));//horizental space to match headers
                        ui.label(
                            egui::RichText::new(name)
                                    .color(egui::Color32::WHITE)
                                    .size(15.0),
                            );
                        ui.allocate_space(egui::vec2(30.0, 0.0));
                        let temp = memory as f32;
                        let memory_bytes = temp * 1024.0 as f32;
                        let memory_color = if memory_bytes < total_memory * 0.05 {
                            egui::Color32::from_gray(128)
                        } else if memory_bytes < total_memory * 0.20 {
                            egui::Color32::GREEN
                        } else if memory_bytes < total_memory * 0.50 {
                            egui::Color32::YELLOW
                        } else if memory_bytes < total_memory * 0.75 {
                            egui::Color32::from_rgb(255, 165, 0)
                        } else {
                            egui::Color32::RED
                        };
                        ui.label(                          
                            egui::RichText::new(((memory / (1024 * 1024)).to_string()))
                                 .color(memory_color)
                                    .size(15.0),
                            );
                        ui.allocate_space(egui::vec2(20.0, 0.0));
                        let rounded_cpu = format!("{:.2}%", normalized_cpu);//here we set cpu text color based on cpu value
                            let cpu_color = if normalized_cpu < 5.0 {
                                egui::Color32::from_gray(128)//gray if less than 5%
                            } else if normalized_cpu < 30.0 {
                                egui::Color32::GREEN//green if less than 30%
                            } else if normalized_cpu < 60.0 {
                                egui::Color32::YELLOW//yellow if less thann 60% etc...
                            } else if normalized_cpu < 80.0 {
                                egui::Color32::from_rgb(255, 165, 0)//this is orange because it isn't predefined like the others
                            } else {
                                egui::Color32::RED
                            };
                            ui.label(//then create the label with the desired color and text
                                egui::RichText::new(rounded_cpu)
                                    .color(cpu_color)
                                    .size(15.0),
                            );
                        //same as cpu
                        ui.allocate_space(egui::vec2(20.0, 0.0));
                        let st_color = match status {
                            Some(ProcessStatus::Run) => {
                                egui::Color32::GREEN
                            },
                            _ => {
                                egui::Color32::from_gray(128)
                            }
                        };       
                        //this is because status can have no value so if that is the case we set its value to unknown                                       
                        ui.label(
                            egui::RichText::new(status.map_or_else(|| "Unknown".to_string(), |s| format!("{:?}", s)))
                                .color(st_color)
                                .size(15.0),
                        );
                        ui.end_row();
                        ui.allocate_space(egui::vec2(0.0, 2.0));
                        ui.end_row();
                    }
                });
            });
        });
    }
}