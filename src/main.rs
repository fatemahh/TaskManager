use sysinfo::System;
use std::{collections::HashMap, time::Duration};
use clearscreen;

fn main() {
    let mut system = System::new_all();

    loop {
        // Refresh system and process information
        system.refresh_all();

        // Collect process data and aggregate by name
        let mut aggregated_processes: HashMap<String, (u64, f32)> = HashMap::new();
        for (_, process) in system.processes() {
            if process.memory() > 0 {
                let entry = aggregated_processes
                    .entry(process.name().to_string_lossy().to_string())
                    .or_insert((0, 0.0));
                entry.0 += process.memory();  // Sum memory usage
                entry.1 += process.cpu_usage();  // Sum CPU usage
            }
        }

        // Sort processes by memory usage (descending)
        let mut sorted_processes: Vec<_> = aggregated_processes.into_iter().collect();
        sorted_processes.sort_by(|a, b| {
            // Compare by memory usage, then by CPU usage if memory is equal
            let mem_cmp = b.1 .0.cmp(&a.1 .0);
            if mem_cmp == std::cmp::Ordering::Equal {
                b.1 .1.partial_cmp(&a.1 .1).unwrap_or(std::cmp::Ordering::Equal)
            } else {
                mem_cmp
            }
        });

        clearscreen::clear().unwrap();

        // Print header
        println!("{:<20} {:<10} {:<10}", "Name", "Memory (KB)", "CPU Usage (%)");

        // Print aggregated process details
        for (name, (memory, cpu)) in sorted_processes {
            println!(
                "{:<20} {:<10} {:<10.2}",
                name,
                memory / 1024,  // Convert memory to KB
                cpu
            );
        }

        // Wait before refreshing the display
        std::thread::sleep(Duration::from_secs(1));  // Update every 1 second
    }
}
