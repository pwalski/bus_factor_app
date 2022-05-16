//! Bus factor estimation
//!
//! # Overview
//!
//! Bus factor is a measurement which attempts to estimate the number of key persons a repository would need to lose in order for it to become stalled due to lack of expertise.
//! It is commonly used in the context of software development.
//! For example, if a given repository is developed by a single person, then the repository's bus factor is equal to 1 (it's likely for the repository to become unmaintained if the main contributor suddenly stops working on it).
//!
//! Library finds popular GitHub repositories with a bus factor of 1.
//! Given a programming language name (`language`) and a repository count (`repo_count`), library fetches the first `repo_count` most popular repositories (sorted by the number of GitHub stars) from the given language.
//! Then, for each repository, it inspect its contributor statistics.
//! We assume a repository's bus factor is 1 if its most active developer's contributions account for 75% or more of the total contributions count from the top 25 most active developers.
//! Repositories with a bus factor of 75% or higher are returned as a Result.

#[cfg(feature = "api")]
pub mod api;

#[cfg(feature = "calculator")]
pub mod calculator;

#[cfg(feature = "calculator")]
pub use calculator::BusFactor;
#[cfg(feature = "calculator")]
pub use calculator::BusFactorCalculator;
#[cfg(feature = "calculator")]
pub use calculator::BusFactorStream;
