//! One job: the current UTC time as an ISO 8601 string, without pulling in a
//! date library. Stored timestamps use this format
//! (docs/decisions/0003-sign-in.md data model).

use std::time::{SystemTime, UNIX_EPOCH};

/// The current time as `YYYY-MM-DDThh:mm:ssZ` in UTC.
pub fn now_iso8601() -> String {
    to_iso8601(now_unix())
}

/// The current time in whole seconds since the Unix epoch. Used to decide when
/// an access token is close enough to expiry to be renewed.
pub fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn to_iso8601(unix_secs: i64) -> String {
    let days = unix_secs.div_euclid(86_400);
    let secs_of_day = unix_secs.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let (hh, mm, ss) = (
        secs_of_day / 3600,
        (secs_of_day % 3600) / 60,
        secs_of_day % 60,
    );
    format!("{year:04}-{month:02}-{day:02}T{hh:02}:{mm:02}:{ss:02}Z")
}

/// Days since the Unix epoch to a civil (year, month, day). Howard Hinnant's
/// `civil_from_days`, which is exact for the whole range we care about.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
    (y + i64::from(m <= 2), m, d)
}

#[cfg(test)]
mod tests {
    use super::to_iso8601;

    #[test]
    fn epoch_zero() {
        assert_eq!(to_iso8601(0), "1970-01-01T00:00:00Z");
    }

    #[test]
    fn a_known_moment() {
        // 2026-08-28T13:20:45Z
        assert_eq!(to_iso8601(1_787_923_245), "2026-08-28T13:20:45Z");
    }

    #[test]
    fn leap_day() {
        // 2024-02-29T12:00:00Z
        assert_eq!(to_iso8601(1_709_208_000), "2024-02-29T12:00:00Z");
    }

    #[test]
    fn format_is_fixed_width() {
        assert_eq!(to_iso8601(0).len(), 20);
        assert_eq!(to_iso8601(1_787_923_245).len(), 20);
    }
}
