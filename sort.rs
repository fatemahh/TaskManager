use sysinfo::{System, SystemExt, ProcessExt};
use std::collections::HashMap;

fn main() {
    // Initialize the system
    let mut system = System::new_all();
    system.refresh_all();

    // Create a HashMap to aggregate processes by their name
    let mut aggregated_processes: HashMap<String, (u64, f32)> = HashMap::new();

    // Collect process information and aggregate by name
    for (_, process) in system.processes() {
        // If memory usage is greater than 0, aggregate it
        if process.memory() > 0 {
            let entry = aggregated_processes.entry(process.name().to_string()).or_insert((0, 0.0));
            entry.0 += process.memory();  // Sum the memory usage
            entry.1 += process.cpu_usage();  // Sum the CPU usage
        }
    }

    // Convert the aggregated data to a vector and sort it by memory usage
    let mut sorted_processes: Vec<_> = aggregated_processes.into_iter().collect();
    sorted_processes.sort_by(|a, b| {
        // First, compare memory usage in descending order
        let mem_cmp = b.1 .0.cmp(&a.1 .0);
        if mem_cmp == std::cmp::Ordering::Equal {
            // If memory is the same, compare CPU usage in descending order
            b.1 .1.partial_cmp(&a.1 .1).unwrap_or(std::cmp::Ordering::Equal)
        } else {
            mem_cmp
        }
    });

    // Print headers for the process table
    println!("{:<20} {:<10} {:<10}", "Name", "Memory (KB)", "CPU Usage (%)");

    // Print aggregated and sorted process details
    for (name, (memory, cpu)) in sorted_processes {
        println!(
            "{:<20} {:<10} {:<10.2}",
            name,
            memory,
            cpu
        );
    }
}

