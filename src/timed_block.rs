use std::time::Instant;
use std::io::Write;

pub fn timed_block<T>(text: &str, func: impl FnOnce() -> T) -> T {
    print!("{:40} ", format!("{}...", text));

    std::io::stdout().flush().unwrap();

    let start_time = Instant::now();
    let result = func();
    let end_time = Instant::now();

    println!("done in {:.3} seconds.", end_time.duration_since(start_time).as_secs_f64());

    result
}
