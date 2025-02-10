
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
