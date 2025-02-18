/// Minutes to add to the start of each coding session
///
/// A session is defined in conjunction with the `MAX_SESSION_IDLE_IN_MINUTES` constant below.
/// The coding session duration is calculated by adding the time delta between the commits in that
/// specific session, as well as the session start minutes defined by this constant. This is done
/// as we do not know when the user has started working on a specific coding session, and therefore
/// we have to add based on a guess.
pub const SESSION_START_ADDITION_IN_MINUTES: usize = 15;

/// Minutes the user can idle before the session is considered to be separate
///
/// If no commits are made for the specified duration, the next commit starts a new coding session.
pub const MAX_SESSION_IDLE_IN_MINUTES: usize = SESSION_START_ADDITION_IN_MINUTES;

/// Threshold of insertion/deletion ratio to consider strange
///
/// If an author's insertion/deletion ratio is below this threshold, then a warning symbol is
/// displayed next to their statistics.
pub const INSERTION_DELETION_WARNING_RATIO: f64 = 0.6;

/// CS 252 username (for filtering out of statistics)
pub const CS252_USER_NAME: &str = "CS252";

/// Sentry DSN URL
///
/// Used by Sentry to send bug and crash reports
pub const SENTRY_DSN_URL: &str =
    "https://ef5e99a8089c74d5c41a66197d8c1d45@o444286.ingest.us.sentry.io/4508797911957504";
