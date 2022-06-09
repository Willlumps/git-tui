use std::fmt::Display;

use chrono::{Datelike, Month, NaiveDateTime, Timelike};
use git2::Time as GitTime;
use num_traits::FromPrimitive;

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
            x if x < 3600 => format!("{}{}", (x / 60), "m"),
            x if x < 86400 => format!("{}{}", (x / 3600), "hr"),
            x if x < 604_800 => format!("{}{}", (x / 86400), "d"),
            x if x < 2_419_200 => format!("{}{}", (x / 604_800), "wk"),
            x if x < 31_536_000 => format!("{}{}", (x / 2_419_200), "mo"),
            x => format!("{}{}", (x / 31_536_000), "yr"),
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
