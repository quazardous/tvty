//! Whose work it is, as the programs' "About" says it: the copyright line,
//! its years running from the first to the current one, so it never needs
//! touching again.

/// The first year of the work.
pub const SINCE: i32 = 2026;
/// Who holds its copyright.
pub const HOLDER: &str = "David Berlioz";

/// "Copyright (c) 2026 David Berlioz", or "2026–2028" once the years run.
pub fn copyright() -> String {
    copyright_in(current_year())
}

/// The line as it reads in `year`.
pub fn copyright_in(year: i32) -> String {
    if year > SINCE {
        format!("Copyright (c) {SINCE}–{year} {HOLDER}")
    } else {
        format!("Copyright (c) {SINCE} {HOLDER}")
    }
}

/// The year now, in UTC, from the system clock.
fn current_year() -> i32 {
    let days = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| (d.as_secs() / 86_400) as i64);
    year_of_day(days)
}

/// The year of `days` after 1970-01-01 (the proleptic Gregorian calendar).
fn year_of_day(days: i64) -> i32 {
    // Howard Hinnant's civil_from_days, the year alone.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    (yoe + era * 400 + i64::from(month <= 2)) as i32
}

#[cfg(test)]
mod tests {
    use super::{copyright_in, year_of_day};

    #[test]
    fn the_years_run_from_the_first_to_this_one() {
        assert_eq!(copyright_in(2026), "Copyright (c) 2026 David Berlioz");
        assert_eq!(copyright_in(2028), "Copyright (c) 2026–2028 David Berlioz");
    }

    #[test]
    fn a_day_reads_as_its_year() {
        assert_eq!(year_of_day(0), 1970);
        // 2026-09-30, and the last and first days around a new year.
        assert_eq!(year_of_day(20_726), 2026);
        assert_eq!(year_of_day(20_818), 2026);
        assert_eq!(year_of_day(20_819), 2027);
        // 2028-02-29, a leap day.
        assert_eq!(year_of_day(21_243), 2028);
    }
}
