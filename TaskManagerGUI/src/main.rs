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
        }
    }
}

struct TaskManager {
    last_update: Instant,
    refresh_interval: Duration,
    system: sysinfo::System, //default value
}

fn main() {

    // SINCE WE ARE SURE THE TERMINAL ONE IS WORKING CORRECTLY ONCE WE DON'T KNOW WHETHER THE GUI
    // DISPLAYS CORRECT RESULTS OR NOT, WE DISABLE GUI CODE AND ENABLE THE CODE BELOW TO SEE THE CORRECT VALUES
    // terminal_ui();

    //THIS IS GUI FUNCTION, COMMENT TO DISABLE
    startGUI();
    
}

fn startGUI(){
    //open the gui with name Task Manager
    eframe::run_native(
        "Task Manager",
        eframe::NativeOptions {
            drag_and_drop_support: true,
            initial_window_size: Some(egui::vec2(800.0, 600.0)), //this determines starting resolution
            ..Default::default()
        },
        Box::new(|_cc| Box::<TaskManager>::default()),//second time the struct is used, this is related to memory and how gui is stored
    );
}

//this is currently just a terminal function, it isn't actaully called or anything like that, but still important
// don't delete!!!
fn terminal_ui() {
    println!("Welcome! Type 'help' to view all commands.");
    let mut system = sysinfo::System::new_all();

    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read input");
        let input = input.trim();

        match input.split_whitespace().collect::<Vec<&str>>().as_slice() {
            &["display"] => {
                loop {
                    if event::poll(Duration::from_millis(100)).expect("Failed to poll event") {
                        if let event::Event::Key(_) = event::read().expect("Failed to read event") {
                            terminal::disable_raw_mode().expect("Failed to disable raw mode");
                            println!("Process data view ended.");
                            break;
                        }
                    }
                    // Refresh system and process information
                    system.refresh_all();

                    // Collect process data and aggregate by name
                    let mut aggregated_processes: HashMap<String, (u64, f32, Option<u32>, Option<ProcessStatus>)> = HashMap::new();
                    for (_, process) in system.processes() {
                        if process.memory() > 0 {
                            let entry = aggregated_processes
                                .entry(process.name().to_string_lossy().to_string())
                                .or_insert((0, 0.0, None, None));
                            entry.0 += process.memory();  // Sum memory usage
                            entry.1 += process.cpu_usage();  // Sum CPU usage

                            if entry.2.is_none() {
                                entry.2 = Some(process.pid().as_u32());
                            }

                            if entry.3.is_none() {
                                entry.3 = Some(process.status());
                            }
                        }
                    }
                    let mut sorted_processes: Vec<_> = aggregated_processes.into_iter().collect();
                    sorted_processes.sort_by(|a, b| b.1 .0.cmp(&a.1 .0)); // Compare the memory usage values

                    clearscreen::clear().unwrap();
                    terminal::disable_raw_mode().expect("Failed to re-enter raw mode");

                    // Print header
                    println!("{:<10} {:<20} {:<15} {:<15} {:<15}", "PID", "Name", "Memory (MB)", "CPU Usage (%)", "Status");

                    // Print aggregated process details
                    for (name, (memory, cpu, pid, status)) in sorted_processes {
                        println!(
                            "{:<10} {:<20} {:<15.2} {:<15.2} {:<15}",
                            pid.unwrap_or(0),
                            name,
                            memory / (1024*1024),
                            cpu,
                            status.map_or("Unknown".to_string(), |s| format!("{:?}", s))
                        );
                    }
                    terminal::enable_raw_mode().expect("Failed to re-enter raw mode");

                    std::thread::sleep(Duration::from_millis(100));
                }
            }
            &["exit"] => {
                println!("Goodbye!");
                break;
            }
            &["help"] => {
                println!(
                    "Available commands:
                    \n  -- 'display'            : View processes info.
                    \n  -- 'display <status>'   : View processes by status (e.g., 'display sleep')
                    \n  -- 'search <proc_id>'   : Search for a process by its PID.
                    \n  -- 'kill <proc_id>'     : Kill a process, where <proc_id> is the process ID.
                    \n  -- 'sleep <proc_id>'    : Put a process to sleep, where <proc_id> is the process ID.
                    \n  -- 'resume <proc_id>'   : Resume a sleeping process, where <proc_id> is the process ID.
                    \n  -- 'count'              : Display process counts by state.
                    \n  -- 'exit'               : To exit the Task Manager.
                    \n"
                );
            }
            _ => {
                println!("Unknown command. Type 'help' to view all commands.");
            }
        }
    }
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            last_update: Instant::now(),
            refresh_interval: Duration::from_millis(100),
            system: System::new_all(),
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
                            .size(18.0),
                    );
                    ui.allocate_space(egui::vec2(20.0, 0.0));//this creates a space, as 2d vector where
                    //20 is the horizental value and 0 is the vertical one it creates only hroizental space
                    ui.label(
                        egui::RichText::new("Name")
                            .color(egui::Color32::WHITE)
                            .size(18.0),
                    );
                    ui.allocate_space(egui::vec2(20.0, 0.0));
                    ui.label(
                        egui::RichText::new("Memory (MB)")
                            .color(egui::Color32::WHITE)
                            .size(18.0),
                    );
                    ui.allocate_space(egui::vec2(20.0, 0.0));
                    ui.label(
                        egui::RichText::new("CPU Usage (%)")
                            .color(egui::Color32::WHITE)
                            .size(18.0),
                    );
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
                    sorted_processes.sort_by(|a, b| b.1 .0.cmp(&a.1 .0)); // Compare memory usage
                    
                    // Display sorted processes in the table by looping over them one by one
                    for (name, (memory, cpu, pid, status)) in sorted_processes {
                        ui.label(pid.map_or("Unknown".to_string(), |v| v.to_string()));//here it creates a label and displays pid in it
                        ui.allocate_space(egui::vec2(20.0, 0.0));//horizental space to match headers
                        ui.label(name);
                        ui.allocate_space(egui::vec2(20.0, 0.0));
                        ui.label((memory / (1024 * 1024)).to_string());
                        ui.allocate_space(egui::vec2(20.0, 0.0));
                        let rounded_cpu = format!("{:.2}%", cpu);//here we set cpu text color based on cpu value
                            let cpu_color = if cpu < 20.0 {
                                egui::Color32::GREEN//green if less than 20%
                            } else if cpu < 50.0 {
                                egui::Color32::YELLOW//yellow if less thann 50% etc...
                            } else if cpu < 70.0 {
                                egui::Color32::from_rgb(255, 165, 0)//this is oragne because it isn't predefined like the others
                            } else {
                                egui::Color32::RED
                            };
                            ui.label(//then create the label with the desired color and text
                                egui::RichText::new(rounded_cpu)
                                    .color(cpu_color)
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
                        );
                        ui.end_row();
                    }
                });
            });
        });
    }
}