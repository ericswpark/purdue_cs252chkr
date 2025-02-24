use chrono::{DateTime, Duration, Local, Utc};
use chrono_humanize::Accuracy::Precise;
use chrono_humanize::HumanTime;
use chrono_humanize::Tense::Present;

/// Get DateTime object from seconds
///
/// * `seconds` - seconds to convert into DateTime
pub fn get_time(seconds: i64) -> Option<DateTime<Utc>> {
    DateTime::from_timestamp(seconds, 0)
}

/// Get localized time from seconds
///
/// * `seconds` - seconds to convert into localized DateTime
pub fn get_localized_time(seconds: i64) -> Option<DateTime<Local>> {
    let time_utc = get_time(seconds)?;
    Some(time_utc.with_timezone(&Local))
}

/// Get time formatted into a human-readable format from a localized DateTime
///
/// * `time` - localized DateTime to get formatted human-readable date/time from
pub fn get_formatted_time(time: DateTime<Local>) -> String {
    time.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Get "humanized" (X minutes ago, X hours ago) time from a localized DateTime
///
/// * `time` - localized DateTime to get "humanized" date/time from
pub fn get_humanized_time(time: DateTime<Local>) -> String {
    HumanTime::from(time - Local::now()).to_string()
}

/// Get "humanized" (X hours X minutes) time from minutes
///
/// * `minutes` - minutes to convert into "humanized" time
pub fn get_humanized_minutes(minutes: i64) -> String {
    let duration = Duration::minutes(minutes);
    get_humanized_duration(&duration)
}

/// Get "humanized" (X hours X minutes) time from duration
///
/// * `duration` - Duration reference to convert into "humanized" time
pub fn get_humanized_duration(duration: &Duration) -> String {
    HumanTime::from(*duration).to_text_en(Precise, Present)
}
