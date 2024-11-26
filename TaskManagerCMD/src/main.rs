use sysinfo::{System, ProcessStatus}; // Retrieve system info
use std::{collections::HashMap, time::Duration, io::{self, Write}};
use clearscreen; // clear terminal screen
use crossterm::{event, terminal};
use nix::sys::signal::{self, Signal}; // For sending signals like SIGSTOP/SIGCONT
use nix::unistd::Pid; // For working with PIDs

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