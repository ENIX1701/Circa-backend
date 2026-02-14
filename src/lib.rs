// shared modules, easier to test
pub mod config;
pub mod db;
pub mod error;
pub mod models;
pub mod modules;
pub use modules::{auth, user};
