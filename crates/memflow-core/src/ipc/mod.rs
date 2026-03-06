//! IPC Server module for MemFlow Core
//!
//! This module provides the IPC server that allows Core to communicate
//! with Desktop UI and MCP clients over TCP (localhost).

pub mod server;
pub mod commands;
pub mod protocol;

pub use server::{IpcServer, CoreStateManager, CoreState, DEFAULT_IPC_PORT};
pub use commands::{CoreStatus, ConfigUpdateRequest, ActivityRecord, SearchQueryRequest, RecentActivityRequest};
pub use protocol::{IpcRequest, IpcResponse, IpcError};
