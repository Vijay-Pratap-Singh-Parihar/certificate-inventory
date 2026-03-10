//! Test suite for certificate-service.
//! Run with: cargo test
//! Repository and API tests require DATABASE_URL (e.g. in CI).

mod api_tests;
mod expiration_tests;
mod parser_tests;
mod repository_tests;
