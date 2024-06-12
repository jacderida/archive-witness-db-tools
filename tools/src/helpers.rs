use chrono::Duration;
use sqlx::postgres::types::PgInterval;
use std::path::{Path, PathBuf};

pub fn interval_to_duration(interval: &PgInterval) -> Duration {
    let total_microseconds = interval.microseconds;
    let seconds = total_microseconds / 1_000_000;
    let nanoseconds = (total_microseconds % 1_000_000) * 1_000;
    Duration::seconds(seconds) + Duration::nanoseconds(nanoseconds)
}

pub fn parse_duration(time: &str) -> Duration {
    let parts: Vec<&str> = time.split(':').collect();
    if parts.len() == 3 {
        let hours = parts[0].parse::<i64>().unwrap();
        let minutes = parts[1].parse::<i64>().unwrap();
        let seconds_parts: Vec<&str> = parts[2].split('.').collect();
        let seconds = seconds_parts[0].parse::<i64>().unwrap();
        let millis = if seconds_parts.len() > 1 {
            seconds_parts[1].parse::<i64>().unwrap()
        } else {
            0
        };
        return Duration::milliseconds(
            hours * 3_600_000 + minutes * 60_000 + seconds * 1000 + millis,
        );
    }
    Duration::zero()
}

pub fn duration_to_string(d: &Duration) -> String {
    format!(
        "{:02}:{:02}:{:02}",
        d.num_hours(),
        d.num_minutes() % 60,
        d.num_seconds() % 60
    )
}

pub fn strip_first_two_directories(path: &Path) -> PathBuf {
    path.components().skip(2).collect()
}

pub fn human_readable_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if size >= GB {
        format!("{:.2} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else {
        format!("{} bytes", size)
    }
}

pub fn print_banner(text: &str) {
    let padding = 2;
    let text_width = text.len() + padding * 2;
    let border_chars = 2;
    let total_width = text_width + border_chars;
    let top_bottom = "═".repeat(total_width);

    println!("╔{}╗", top_bottom);
    println!("║ {:^width$} ║", text, width = text_width);
    println!("╚{}╝", top_bottom);
}
