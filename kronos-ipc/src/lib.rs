//! Inter-process communication between kronos and kronosctl
//! 
//! We use Unix domain sockets for local IPC - they're fast, secure,
//! and perfect for this use case.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Commands that kronosctl can send to kronos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    Start,
    Pause,
    Resume,
    Stop,
    Reset,
    Status,
    AddTask { description: String },
    ListTasks,
}

/// Responses from kronos back to kronosctl
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    Ok,
    Status(TimerStatus),
    Tasks(Vec<Task>),
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerStatus {
    pub state: TimerState,
    pub elapsed: u64, // seconds
    pub total: u64,   // seconds
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimerState {
    Idle,
    Running,
    Paused,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: u32,
    pub description: String,
    pub completed: bool,
}

#[derive(Error, Debug)]
pub enum IpcError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Connection refused - is kronos running?")]
    ConnectionRefused,
}

pub const SOCKET_PATH: &str = "/tmp/kronos.sock";
