// Avoid a lot of boilerplate compared to using the `systemd` crate by leveraging the API contract
// better. I'm not iterating journal values, so I don't really need as many safeguards.

mod cursor;
mod id128;
#[cfg(test)]
mod journal_mocks;
mod journal_ref;
mod journal_types;
mod provider;

pub use cursor::Cursor;
pub use id128::Id128;
#[cfg(test)]
pub use journal_mocks::*;
pub use journal_ref::NativeJournalRef;
pub use journal_types::*;
pub use provider::NativeSystemdProvider;
