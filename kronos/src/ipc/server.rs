//! Unix domain socket server for IPC

use crate::app::App;
use anyhow::Result;
use kronos_ipc::{Command, Response, SOCKET_PATH};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tracing::{error, info, warn};

pub async fn start() -> Result<()> {
    // Remove old socket if it exists
    let _ = std::fs::remove_file(SOCKET_PATH);
    
    // Bind to socket
    let listener = UnixListener::bind(SOCKET_PATH)?;
    info!("IPC server listening on {}", SOCKET_PATH);
    
    // TODO: We need to share the App instance between the main thread and this server
    // For now, we'll create a new one, but in real implementation, we'd pass it in
    let app = Arc::new(tokio::sync::Mutex::new(App::new()));
    
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let app = app.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_client(stream, app).await {
                        error!("Error handling client: {}", e);
                    }
                });
            }
            Err(e) => {
                error!("Error accepting connection: {}", e);
            }
        }
    }
}

async fn handle_client(stream: UnixStream, app: Arc<tokio::sync::Mutex<App>>) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    
    // Read command
    reader.read_line(&mut line).await?;
    let command: Command = serde_json::from_str(&line)?;
    
    // Process command
    let response = match command {
        Command::Start => {
            let mut app = app.lock().await;
            app.toggle_timer(); // TODO: This should specifically start, not toggle
            Response::Ok
        }
        Command::Pause | Command::Resume => {
            let mut app = app.lock().await;
            app.toggle_timer();
            Response::Ok
        }
        Command::Stop | Command::Reset => {
            let mut app = app.lock().await;
            app.reset_timer();
            Response::Ok
        }
        Command::Status => {
            let app = app.lock().await;
            Response::Status(app.get_status())
        }
        Command::AddTask { description } => {
            let mut app = app.lock().await;
            app.add_task(description);
            Response::Ok
        }
        Command::ListTasks => {
            let app = app.lock().await;
            Response::Tasks(app.get_tasks())
        }
    };
    
    // Send response
    let response_json = serde_json::to_vec(&response)?;
    writer.write_all(&response_json).await?;
    
    Ok(())
}
