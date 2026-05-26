use std::time::{SystemTime, UNIX_EPOCH};

fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}

fn format_time_utc8() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
        + 8 * 3600; // CST = UTC+8

    let days = secs / 86400;
    let time = secs % 86400;
    let hours = time / 3600;
    let minutes = (time % 3600) / 60;

    let mut year = 1970i64;
    let mut remaining = days;
    loop {
        let diy = if is_leap(year) { 366 } else { 365 };
        if remaining < diy {
            break;
        }
        remaining -= diy;
        year += 1;
    }

    let month_len = if is_leap(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1;
    for &ml in &month_len {
        if remaining < ml {
            break;
        }
        remaining -= ml;
        month += 1;
    }
    let day = remaining + 1;

    format!("v{:04}.{:02}.{:02}.{:02}{:02}", year, month, day, hours, minutes)
}

fn main() {
    // Version override via env var (e.g. build.ps1 sets NETHELPER_VERSION)
    let version = std::env::var("NETHELPER_VERSION")
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(format_time_utc8);

    println!("cargo:rustc-env=NETHELPER_VERSION={}", version);
    // Always regenerate — version changes with time
    println!("cargo:rerun-if-changed=");
}
