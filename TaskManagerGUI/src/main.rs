use sysinfo::{System, ProcessStatus, Process};
use std::{collections::HashMap, time::Duration, io::{self, Write}};
use std::time::{Instant};
use clearscreen; // clear terminal screen
use crossterm::{event, terminal};
use nix::sys::signal::{self, Signal}; // For sending signals like SIGSTOP/SIGCONT
use nix::unistd::Pid; // For working with PIDs
use eframe::{self, egui};


// PROCESS DISPLAY GUI

struct ProcessDisplay {
    last_update: Instant,
    refresh_interval: Duration,
    system: sysinfo::System, // default value
    sort_criteria: SortCriteria,
    reverse_sort: bool, // ASC or DEC

    // For alerts
    check_alerts: CheckAlerts,
    show_alert_popup: bool,
    alert_pid: u32,
    alert_name: String,
    alert_cpu: f32,
    alert_memory: u64,
}

impl ProcessDisplay {
    pub fn new() -> Self {
        Self {
            last_update: Instant::now(),
            refresh_interval: Duration::from_millis(400),
            system: System::new_all(),
            sort_criteria: SortCriteria::Memory,
            reverse_sort: false,

            check_alerts: CheckAlerts::new(90.0, 2 * 1024 * 1024 * 1024), // 90% CPU and 2 GB memory
            show_alert_popup: false,
            alert_pid: 0,
            alert_name: String::new(),
            alert_cpu: 0.0,
            alert_memory: 0,
        }
    }
}

impl Default for ProcessDisplay {
    fn default() -> Self {
        ProcessDisplay {
            last_update: Instant::now(),
            refresh_interval: Duration::from_millis(400),
            system: System::new_all(),
            sort_criteria: SortCriteria::Memory,
            reverse_sort: false,

            check_alerts: CheckAlerts::new(90.0, 2 * 1024 * 1024 * 1024), // 90% CPU and 2 GB memory
            show_alert_popup: false,
            alert_pid: 0,
            alert_name: String::new(),
            alert_cpu: 0.0,
            alert_memory: 0,
        }
    }
}

// TREE VIEW GUI

struct TreeView {
    system: sysinfo::System,
}

impl TreeView {
    pub fn new() -> Self {
        Self {
            system: System::new_all(),
        }
    }
}

impl Default for TreeView {
    fn default() -> Self {
        TreeView {
            system: System::new_all(),
        }
    }
}

// CHECK ALERTS GUI

struct CheckAlerts {
    system: System,
    alert_message: Option<String>,
    cpu_threshold: f32,
    memory_threshold: u64, // in bytes (e.g., 2 GB = 2 * 1024 * 1024 * 1024)
}


// ---------------------------------------------------------------------------------

// used to determine sort style
#[derive(PartialEq)] //this is an attribute it can be derived from so that it allows comparisons (If sort_crit==mem)
enum SortCriteria {
    Memory,
    CPU,
}

fn get_total_memory_mb(system: &sysinfo::System) -> f32 {
    system.total_memory() as f32 / 1024.0 // Convert from KB to MB
}

impl eframe::App for TreeView {

    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.system.refresh_all(); // Refresh system info

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.allocate_space(egui::vec2(0.0, 20.0));
            ui.end_row();
            ui.vertical_centered_justified(|ui| {//this centers the text and at the top of the screen
                //here we create a heading element, this element has text
                    ui.heading(
                        egui::RichText::new("Process Tree") // modify text here
                        .size(50.0) //text size
                        .color(egui::Color32::WHITE) //test color
                        .strong(), // make it bold
                    );
                });
    
            ui.allocate_space(egui::vec2(0.0, 20.0));

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
                //this is a utlity function that changes process color based on its depth
                fn get_color_for_depth(depth: usize) -> egui::Color32 {
                    match depth {
                        0 => egui::Color32::YELLOW, // paren is yellow
                        1 => egui::Color32::LIGHT_BLUE, // direct child is light blue
                        2..=6 => { //from 2 to 6 it goes from purplish to more white
                            let intensity = 150 + ((depth - 2) as u8 * 25);
                            egui::Color32::from_rgb(intensity, intensity, 255)
                        }
                        7 => egui::Color32::WHITE, //depth 7 has color white
                        8..=12 => { //from depth 8 to 12 it keeps going closer to gray
                            let intensity = 255 - ((depth - 8) as u8 * 50);
                            egui::Color32::from_gray(intensity)
                        }
                        _ => egui::Color32::GRAY, //more than 12 is gray
                    }
                }
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
                                ui.horizontal(|ui| {
                                    ui.add_space(depth as f32 * 60.0); //increase the value to increase space between parent and child
                                    
                                    // Create styled label
                                    let label_text = match depth {
                                        0 => "Parent:",
                                        _ => "Child:",
                                    };
                                    
                                    let text = egui::RichText::new(format!(
                                        "{} PID: {} - Name: {}",
                                        label_text,
                                        child_pid,
                                        child.name().to_string_lossy()
                                    ))
                                    .color(get_color_for_depth(depth))
                                    .size(15.0);
                                    
                                    ui.label(text);
                                });
                
                                // space between elements vertically
                                ui.add_space(7.0);
                
                                show_tree(ui, tree_map, system, child_pid, depth + 1);
                            }
                        }
                    }
                }
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        show_tree(ui, &tree_map, &self.system, 0, 0);
                    });
                    ui.add_space(50.0); //this adds horizental space between most depth child and scroll bar
                });
            });
        });
    }
}

impl CheckAlerts {
    // Create a new instance of CheckAlerts with CPU and memory thresholds
    fn new(cpu_threshold: f32, memory_threshold: u64) -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        CheckAlerts {
            system,
            alert_message: None,
            cpu_threshold,
            memory_threshold,
        }
    }

    // Method to check if any process exceeds the thresholds
    fn check_for_alerts(&mut self) -> Option<(u32, String, f32, u64)> {
        // Refresh system data (e.g., processes, memory, CPU usage)
        self.system.refresh_all();

        // Check each process's CPU and memory usage
        for (_, process) in self.system.processes() {
            let cpu_usage = process.cpu_usage();
            let memory_usage = process.memory();

            // If the process exceeds the threshold, return the process info
            if cpu_usage > self.cpu_threshold || memory_usage > self.memory_threshold {
                return Some((
                    process.pid().as_u32(),
                    process.name().to_string_lossy().to_string(),
                    cpu_usage,
                    memory_usage,
                ));
            }
        }
        None
    }
}

impl eframe::App for ProcessDisplay { // this is 3rd time struct is used

    // update here is a special function that is called automatically every frame
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        
        // Check for alerts on every update
        if let Some((pid, name, cpu, memory)) = self.check_alerts.check_for_alerts() {
            self.alert_pid = pid;
            self.alert_name = name;
            self.alert_cpu = cpu;
            self.alert_memory = memory;
            self.show_alert_popup = true; // Show popup when a threshold is exceeded
        }

        let now = Instant::now();

        // Refresh system info only if 0.1 seconds have passed
        if now.duration_since(self.last_update) >= self.refresh_interval {
            self.system.refresh_all();
            self.last_update = now;
        }

        // Request a repaint
        ctx.request_repaint();

        egui::CentralPanel::default().show(ctx, |ui| {
            
            // Alert message popup
            if self.show_alert_popup {
                // let total_memory = get_total_memory_mb(&self.system) * 1024.0 * 1024.0;
                let num_cores = self.system.cpus().len() as f32;

                egui::Window::new(egui::RichText::new("High Resource Usage Alert")
                    .size(20.0)
                    .color(egui::Color32::RED)
                    .strong()
                )
                    .open(&mut true) // The window is open by default
                    .show(ctx, |ui| {
                        ui.add_space(10.0);
                        ui.label(
                            egui::RichText::new(format!(
                                "PID: {}\nName: {}\nCPU Usage: {:.2}%\nMemory Usage: {} MB",
                                self.alert_pid,
                                self.alert_name,
                                self.alert_cpu / num_cores,
                                self.alert_memory / 1024 / 1024 // Convert memory from bytes to MB
                            ))
                            .size(16.0)
                            .color(egui::Color32::LIGHT_RED),
                        );
                        // Add space between text and button
                        ui.add_space(10.0);

                        // Button to close the alert window
                        ui.horizontal(|ui| {
                            ui.add_space(120.0);
                        
                            if ui.button(
                                egui::RichText::new("OK")
                                    .size(15.0)
                                    .color(egui::Color32::WHITE),
                            )
                            .clicked()
                            {
                                self.show_alert_popup = false; // Close the popup
                            }
                        });
                        ui.add_space(10.0);
                    });
            }

            //some vertical space
            ui.allocate_space(egui::vec2(0.0, 20.0));
            ui.end_row();

            ui.vertical_centered_justified(|ui| {//this centers the text and at the top of the screen
            //here we create a heading element, this element has text
                ui.heading(
                    egui::RichText::new("Task Manager") // modify text here
                    .size(50.0) //text size
                    .color(egui::Color32::WHITE) //test color
                    .strong(), // make it bold
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

            egui::Grid::new("header_grid").show(ui, |ui| {
                ui.label(//this creates a UI label with text PID, color white, and size 18
                    egui::RichText::new("PID")
                        .color(egui::Color32::WHITE)
                        .size(18.0)
                );
                ui.allocate_space(egui::vec2(20.0, 0.0));//this creates a space, as 2d vector where
                //20 is the horizontal value and 0 is the vertical one it creates only hroizontal space
                ui.label(
                    egui::RichText::new("Name")
                        .color(egui::Color32::WHITE)
                        .size(18.0),
                );
                ui.allocate_space(egui::vec2(110.0, 0.0));

                // create a button, same setup as label but it has clicked event which decides what happens 
                // once it is clicked, here we change ProcessDisplay struct reverse_sort and sort criteria 
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
            });
            
            // Create a scrollable area for displaying processes
            egui::ScrollArea::vertical().show(ui, |ui| { // Use `vertical()` for vertical scrolling
                // Create a table layout to show processes
                egui::Grid::new("process_grid").show(ui, |ui| {

                    // Collect and sort processes by memory
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
                            sorted_processes.sort_by(|a, b| {
                                let primary = b.1 .0.cmp(&a.1 .0);
                                //check if same memory then sort by pid
                                if primary == std::cmp::Ordering::Equal {
                                    a.1 .2.cmp(&b.1 .2)
                                } else {
                                    primary
                                }
                            });
                        }
                        SortCriteria::CPU => {
                            sorted_processes.sort_by(|a, b| {
                                let primary = b.1 .1.partial_cmp(&a.1 .1).unwrap_or(std::cmp::Ordering::Equal); // Descending order
                                //check if same memory then sort by pid
                                if primary == std::cmp::Ordering::Equal {
                                    a.1 .2.cmp(&b.1 .2)
                                } else {
                                    primary
                                }
                            });
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
                            egui::RichText::new((memory / (1024 * 1024)).to_string())
                                 .color(memory_color)
                                    .size(15.0),
                            );
                        ui.allocate_space(egui::vec2(110.0, 0.0));
                        let rounded_cpu = format!("{:.2}%", normalized_cpu); // here we set cpu text color based on cpu value
                            let cpu_color = if normalized_cpu < 5.0 {
                                egui::Color32::from_gray(128) // gray if less than 5%
                            } else if normalized_cpu < 30.0 {
                                egui::Color32::GREEN // green if less than 30%
                            } else if normalized_cpu < 60.0 {
                                egui::Color32::YELLOW // yellow if less thann 60% etc...
                            } else if normalized_cpu < 80.0 {
                                egui::Color32::from_rgb(255, 165, 0) // this is orange because it isn't predefined like the others
                            } else {
                                egui::Color32::RED
                            };
                            ui.label( // then create the label with the desired color and text
                                egui::RichText::new(rounded_cpu)
                                    .color(cpu_color)
                                    .size(15.0),
                            );
                        // Same as cpu
                        ui.allocate_space(egui::vec2(110.0, 0.0));
                        let st_color = match status {
                            Some(ProcessStatus::Run) => {
                                egui::Color32::GREEN
                            },
                            _ => {
                                egui::Color32::from_gray(128)
                            }
                        };       
                        // This is because status can have no value so if that is the case we set its value to unknown                                       
                        ui.label(
                            egui::RichText::new(status.map_or_else(|| "Unknown".to_string(), |s| format!("{:?}", s)))
                                .color(st_color)
                                .size(15.0),
                        );
                        ui.allocate_space(egui::vec2(50.0, 0.0));
                        ui.end_row();
                        ui.allocate_space(egui::vec2(0.0, 2.0));
                        ui.end_row();
                    }
                });
            });
        });
    }
}

fn display(system: &mut sysinfo::System)
{
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
            println! (
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

fn display_processes(system: &mut System, status_filter: Option<&str>) {
    // Refresh system and process information
    system.refresh_all();

    // Collect process data and aggregate by name
    let mut aggregated_processes: HashMap<String, (u64, f32, Option<u32>, Option<ProcessStatus>)> = HashMap::new();
    for (_, process) in system.processes() {
        if process.memory() > 0 {
            if let Some(status_filter) = status_filter {
                if !format!("{:?}", process.status()).to_lowercase().contains(status_filter) {
                    continue;
                }
            }
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

    // Print header
    println!("{:<10} {:<20} {:<15} {:<15} {:<15}", "PID", "Name", "Memory (MB)", "CPU Usage (%)", "Status");

    // Print aggregated process details
    for (name, (memory, cpu, pid, status)) in sorted_processes {
        println! (
            "{:<10} {:<20} {:<15.2} {:<15.2} {:<15}",
            pid.unwrap_or(0),
            name,
            memory / (1024*1024),
            cpu,
            status.map_or("Unknown".to_string(), |s| format!("{:?}", s))
        );
    }
}    


fn with_process(pid_str: &str, system: &mut System) -> bool {
    // Parse the input string into a numeric PID
    if let Ok(pid_num) = pid_str.parse::<u32>() {
        // Convert the numeric PID into the `sysinfo::Pid` type
        let sys_pid = sysinfo::Pid::from_u32(pid_num);
        // Check if a process with the given PID exists in the system
        if system.process(sys_pid).is_some() {
            return true; // Process exists, return true
        } else {
            // Print a message if the process is not found
            println!("Process with PID {} not found.", pid_num);
            return false; // Process does not exist, return false
        }
    } else {
        // If the input is not a valid numeric PID, print an error message
        println!("Invalid PID. Please provide a valid numeric PID.");
        return false; // Return false for invalid input
    }
}

fn search_process(pid: u32, system: &System) {
    // Attempt to retrieve the process with the given PID from the system
    if let Some(process) = system.process(sysinfo::Pid::from_u32(pid)) {
        // If the process is found, print its details including:
        // PID, name, memory usage in MB, CPU usage percentage, and status
        println!(
            "Process found: \nPID: {} \nName: {} \nMemory: {} MB \nCPU Usage: {:.2}% \nStatus: {:?}",
            pid,
            process.name().to_string_lossy(), // Converts process name to a displayable string
            process.memory() / 1024 / 1024, // Convert memory usage from bytes to MB
            process.cpu_usage(), // CPU usage percentage
            process.status() // Current status of the process (Running, Sleeping)
        );
    } else {
        // If the process is not found, print an error message
        println!("Process with PID {} not found.", pid);
    }
}

fn show_process_count(system: &System) {
    // Initialize counters for running, sleeping, and stopped processes
    let mut running = 0;
    let mut sleeping = 0;
    let mut stopped = 0;
    // Iterate through all processes in the system
    for (_, process) in system.processes() {
        // Categorize the process based on its current status
        match process.status() {
            // If the status contains "Run", increment the running counter
            status if format!("{:?}", status).contains("Run") => running += 1,
            // If the status contains "Sleep", increment the sleeping counter
            status if format!("{:?}", status).contains("Sleep") => sleeping += 1,
            // If the status contains "Stop", increment the stopped counter
            status if format!("{:?}", status).contains("Stop") => stopped += 1,
            // Ignore all other statuses
            _ => {}
        }
    }
    // Calculate the total number of processes
    let total = running + sleeping + stopped;
    println!( // Print the counts for total, running, sleeping, and stopped processes
        "Total processes: {}\nRunning: {}\nSleeping: {}\nStopped: {}",
        total, running, sleeping, stopped
    );
}

fn kill_process(pid: u32) {
    match signal::kill(Pid::from_raw(pid as i32), Signal::SIGKILL) {
        Ok(_) => println!("Process with PID {} killed successfully.", pid),
        Err(e) => println!("Failed to kill process with PID {}: {}", pid, e),
    }
}

fn sleep_process(pid: u32) {
    match signal::kill(Pid::from_raw(pid as i32), Signal::SIGSTOP) {
        Ok(_) => println!("Process with PID {} paused (SIGSTOP).", pid),
        Err(e) => println!("Failed to pause process with PID {}: {}", pid, e),
    }
}

fn resume_process(pid: u32) {
    match signal::kill(Pid::from_raw(pid as i32), Signal::SIGCONT) {
        Ok(_) => println!("Process with PID {} resumed (SIGCONT).", pid),
        Err(e) => println!("Failed to resume process with PID {}: {}", pid, e),
    }
}

fn main() {

    println!("Welcome! Type 'help' to view all commands.");
    let mut system = System::new_all();
    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read input");
        let input = input.trim();
      
        match input.split_whitespace().collect::<Vec<&str>>().as_slice() {
            &["GUI", "display"] => {
                // open the gui with name Process display
                eframe::run_native(
                "GUI Process Display",
                eframe::NativeOptions {
                    drag_and_drop_support: true,
                    maximized: true,
                    initial_window_size: Some(egui::vec2(800.0, 600.0)), // this determines starting resolution
                    ..Default::default()
                },
                Box::new(|_cc| Box::<ProcessDisplay>::default()), // second time the struct is used, this is related to memory and how gui is stored
                );
            }
            &["Tree", "View", "display"] => {            
                eframe::run_native(
                    "Process Tree",
                    eframe::NativeOptions {
                        drag_and_drop_support: true,
                        maximized: true,
                        initial_window_size: Some(egui::vec2(800.0, 600.0)), // this determines starting resolution
                        ..Default::default()
                    },
                    Box::new(|_cc| Box::<TreeView>::default()),
                )
                .expect("Failed to start eframe app");
            }
            &["display"] => {
                display(&mut system);
            }
            &["display", status] => {
                display_processes(&mut system, Some(status));
            }
            &["search", pid] => {
                if with_process(pid, &mut system) {
                    if let Ok(pid_num) = pid.parse::<u32>() {
                        search_process(pid_num, &system);
                    }
                }
            }
            &["count"] => {
                show_process_count(&system);
            }
            &["kill", pid] => {
                if with_process(pid, &mut system) {
                    if let Ok(pid_num) = pid.parse::<u32>() {
                        kill_process(pid_num);
                    }
                }
            }
            &["sleep", pid] => {
                if with_process(pid, &mut system) {
                    if let Ok(pid_num) = pid.parse::<u32>() {
                        sleep_process(pid_num);
                    }
                }
            }
            &["resume", pid] => {
                if with_process(pid, &mut system) {
                    if let Ok(pid_num) = pid.parse::<u32>() {
                        resume_process(pid_num);
                    }
                }
            }
            &["exit"] => {
                println!("Goodbye!");
                break;
            }
            &["help"] => {
                println!(
                    "Available commands:
                    \n  -- 'GUI display'        : View processes in GUI window
                    \n  -- 'Tree View display'  : View Tree View of process in GUI window
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
