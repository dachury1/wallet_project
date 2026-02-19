//! Transaction Service Library
//!
//! This library acts as the core of the Transaction Service, exporting the necessary modules
//! for the application binary and integration tests.
//!
//! # Modules
//!
//! * `api` - Contains the API interfaces (HTTP/gRPC).
//! * `domain` - Contains the domain entities and business rules.
//! * `infrastructure` - Contains the concrete implementations of repositories and gateways.
//! * `use_cases` - Contains the application business logic and workflows.

pub mod api;
pub mod domain;
pub mod infrastructure;
pub mod jobs;
pub mod use_cases;
