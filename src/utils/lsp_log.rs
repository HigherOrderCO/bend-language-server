//! Logging module.
//!
//! Contains macros to easily send logging information to the client from the language server.

#![allow(unused_macros)]
#![allow(unused_imports)]

macro_rules! error {
    ($client:expr, $($arg:tt)+) => {
        $client.log_message(tower_lsp::lsp_types::MessageType::ERROR, format!($($arg)+)).await;
    };
}
pub(crate) use error;

macro_rules! warning {
    ($client:expr, $($arg:tt)+) => {
        $client.log_message(tower_lsp::lsp_types::MessageType::WARNING, format!($($arg)+)).await;
    };
}
pub(crate) use warning;

macro_rules! info {
    ($client:expr, $($arg:tt)+) => {
        $client.log_message(tower_lsp::lsp_types::MessageType::INFO, format!($($arg)+)).await;
    };
}
pub(crate) use info;

macro_rules! log {
    ($client:expr, $($arg:tt)+) => {
        $client.log_message(tower_lsp::lsp_types::MessageType::LOG, format!($($arg)+)).await;
    };
}
pub(crate) use log;

macro_rules! debug {
    ($client:expr, $($arg:tt)+) => {
        #[cfg(debug_assertions)]
        $client.log_message(tower_lsp::lsp_types::MessageType::INFO, format!($($arg)+)).await;
    };
}
pub(crate) use debug;
