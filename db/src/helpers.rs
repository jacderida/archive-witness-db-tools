use chrono::Duration;
use regex::Regex;
use sqlx::postgres::types::PgInterval;
use std::path::{Path, PathBuf};

pub fn interval_to_duration(interval: &PgInterval) -> Duration {
    let total_microseconds = interval.microseconds;
    let seconds = total_microseconds / 1_000_000;
    let nanoseconds = (total_microseconds % 1_000_000) * 1_000;
    Duration::seconds(seconds) + Duration::nanoseconds(nanoseconds)
}

pub fn parse_duration(time: &str) -> Duration {
    let re = Regex::new(r"^(\d+):(\d+):(\d+)(?:\.(\d+))?$").unwrap();
    let re_min_sec = Regex::new(r"^(?:(\d+)m)?(?:(\d+)s)?$").unwrap();

    if let Some(caps) = re.captures(time) {
        let hours = caps[1].parse::<i64>().unwrap();
        let minutes = caps[2].parse::<i64>().unwrap();
        let seconds = caps[3].parse::<i64>().unwrap();
        let millis = caps
            .get(4)
            .map_or(0, |m| m.as_str().parse::<i64>().unwrap());
        return Duration::milliseconds(
            hours * 3_600_000 + minutes * 60_000 + seconds * 1000 + millis,
        );
    }

    if let Some(caps) = re_min_sec.captures(time) {
        let minutes = caps
            .get(1)
            .map_or(0, |m| m.as_str().parse::<i64>().unwrap());
        let seconds = caps
            .get(2)
            .map_or(0, |s| s.as_str().parse::<i64>().unwrap());
        return Duration::milliseconds(minutes * 60_000 + seconds * 1000);
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
