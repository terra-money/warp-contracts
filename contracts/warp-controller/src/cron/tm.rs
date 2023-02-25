use chrono::{DateTime, Datelike, NaiveDateTime, TimeZone, Timelike, Utc, Weekday};
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct Tm {
    pub tm_sec: u32,
    pub tm_min: u32,
    pub tm_hour: u32,
    pub tm_mday: u32,
    pub tm_mon: u32,
    pub tm_year: i32,
    pub tm_wday: u32,
    pub tm_yday: u32,
}

impl Tm {
    pub fn from_timestamp(time_s: u64) -> Tm {
        let secs = time_s;

        let initial_datetime: DateTime<Utc> = DateTime::from_utc(
            NaiveDateTime::from_timestamp_opt(secs as i64, 0).unwrap(),
            Utc,
        );

        Tm {
            tm_sec: initial_datetime.second(),
            tm_min: initial_datetime.minute(),
            tm_hour: initial_datetime.hour(),
            tm_mday: initial_datetime.day(),
            tm_mon: initial_datetime.month(),
            tm_year: initial_datetime.year(),
            tm_wday: weekday_to_tm_wday(initial_datetime.weekday()),
            tm_yday: initial_datetime.ordinal(),
        }
    }

    pub fn to_datetime(&self) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(
            self.tm_year,
            self.tm_mon,
            self.tm_mday,
            self.tm_hour,
            self.tm_min,
            self.tm_sec,
        )
        .unwrap()
    }

    pub fn to_unix(&self) -> u64 {
        self.to_datetime().timestamp() as u64
    }
}

pub fn weekday_to_tm_wday(weekday: Weekday) -> u32 {
    match weekday {
        chrono::Weekday::Mon => 1,
        chrono::Weekday::Tue => 2,
        chrono::Weekday::Wed => 3,
        chrono::Weekday::Thu => 4,
        chrono::Weekday::Fri => 5,
        chrono::Weekday::Sat => 6,
        chrono::Weekday::Sun => 0,
    }
}
