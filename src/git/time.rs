use std::fmt::Display;

use chrono::{Datelike, Month, NaiveDateTime, Timelike};
use git2::Time as GitTime;
use num_traits::FromPrimitive;

// Larger values are rough estimates, they don't need
// to be super exact.
const SECONDS_IN_MINTUE: u64 = 60;
const SECONDS_IN_HOUR: u64 = 3600;
const SECONDS_IN_DAY: u64 = 86400;
const SECONDS_IN_WEEK: u64 = 604_800;
const SECONDS_IN_MONTH: u64 = 2_419_200;
const SECONDS_IN_YEAR: u64 = 31_536_000;

#[derive(Clone, Debug)]
pub struct CommitDate {
    date: NaiveDateTime,
    time_since_commit: TimeSinceCommit,
}

impl CommitDate {
    pub fn new(git_time: GitTime) -> Self {
        let unix_time = git_time.seconds();
        let date = NaiveDateTime::from_timestamp(unix_time, 0);
        let time_since_commit = time_since_commit(unix_time as u64);

        Self {
            date,
            time_since_commit,
        }
    }

    pub fn time_since_commit(&self) -> &TimeSinceCommit {
        &self.time_since_commit
    }
}

impl Display for CommitDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Mon Jun 6 20:07:16 2022 -0400
        let year = self.date.year();
        let weekday = self.date.weekday();
        let month = Month::from_u32(self.date.month()).expect("What kind of calendar is this??");
        let day = self.date.day();
        let hours = self.date.hour();
        let minutes = self.date.minute();
        let seconds = self.date.second();

        write!(
            f,
            "{} {:?} {} {:02}:{:02}:{:02} {} UTC",
            weekday, month, day, hours, minutes, seconds, year
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeSinceCommit(pub u64);

impl From<TimeSinceCommit> for String {
    fn from(time: TimeSinceCommit) -> Self {
        match time.0 {
            x if x < SECONDS_IN_HOUR => format!("{}{}", (x / SECONDS_IN_MINTUE), "m"),
            x if x < SECONDS_IN_DAY => format!("{}{}", (x / SECONDS_IN_HOUR), "hr"),
            x if x < SECONDS_IN_WEEK => format!("{}{}", (x / SECONDS_IN_DAY), "d"),
            x if x < SECONDS_IN_MONTH => format!("{}{}", (x / SECONDS_IN_WEEK), "wk"),
            x if x < SECONDS_IN_YEAR => format!("{}{}", (x / SECONDS_IN_MONTH), "mo"),
            x => format!("{}{}", (x / SECONDS_IN_YEAR), "yr"),
        }
    }
}

fn time_since_commit(seconds: u64) -> TimeSinceCommit {
    let commit_time = std::time::Duration::new(seconds, 0);
    let start = std::time::SystemTime::now();
    let since_epoch = start
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards");
    let diff = since_epoch.saturating_sub(commit_time);

    TimeSinceCommit(diff.as_secs())
}
