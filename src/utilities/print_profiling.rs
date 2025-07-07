use wgpu_profiler::GpuTimerQueryResult;

fn scopes_to_console_recursive(results: &[GpuTimerQueryResult], indentation: u32) {
    for scope in results {
        if indentation > 0 {
            print!("{:<width$}", "|", width = 4);
        }

        if let Some(time) = &scope.time {
            println!(
                "{:.3}μs - {}",
                (time.end - time.start) * 1000.0 * 1000.0,
                scope.label
            );
        } else {
            println!("n/a - {}", scope.label);
        }

        if !scope.nested_queries.is_empty() {
            scopes_to_console_recursive(&scope.nested_queries, indentation + 1);
        }
    }
}

pub fn console_output(results: &Option<Vec<GpuTimerQueryResult>>, enabled_features: wgpu::Features) {
    print!("\x1B[2J\x1B[1;1H"); // Clear terminal and put cursor to first row first column
    println!("Welcome to wgpu_profiler demo!");
    println!();
    println!("Enabled device features: {:?}", enabled_features);
    println!();
    
    match results {
        Some(results) => {
            scopes_to_console_recursive(results, 0);
        }
        None => println!("No profiling results available yet!"),
    }
}