//! Bus factor estimation
//!
//! # Overview
//!
//! Bus factor is a measurement which attempts to estimate the number of key persons a project would need to lose in order for it to become stalled due to lack of expertise.
//! It is commonly used in the context of software development.
//! For example, if a given project is developed by a single person, then the project's bus factor is equal to 1 (it's likely for the project to become unmaintained if the main contributor suddenly stops working on it).
//!
//! Library finds popular GitHub projects with a bus factor of 1.
//! Given a programming language name (`language`) and a project count (`project_count`), library fetches the first `project_count` most popular projects (sorted by the number of GitHub stars) from the given language.
//! Then, for each project, it inspect its contributor statistics.
//! We assume a project's bus factor is 1 if its most active developer's contributions account for 75% or more of the total contributions count from the top 25 most active developers.
//! Projects with a bus factor of 75% or higher are returned as a Result.
