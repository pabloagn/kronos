use anyhow::Result;
use clap::{Parser, Subcommand};
use kronos_ipc::{Command, Response, SOCKET_PATH};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

#[derive(Parser)]
#[command(name = "kronosctl")]
#[command(about = "Control the Kronos timer", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the timer
    Start,
    /// Pause the timer
    Pause,
    /// Resume the timer
    Resume,
    /// Stop the timer
    Stop,
    /// Reset the timer
    Reset,
    /// Get timer status
    Status,
    /// Add a new task
    Task {
        #[arg(short, long)]
        add: Option<String>,
    },
    /// List all tasks
    Tasks,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Convert CLI command to IPC command
    let command = match cli.command {
        Commands::Start => Command::Start,
        Commands::Pause => Command::Pause,
        Commands::Resume => Command::Resume,
        Commands::Stop => Command::Stop,
        Commands::Reset => Command::Reset,
        Commands::Status => Command::Status,
        Commands::Task { add: Some(desc) } => Command::AddTask { description: desc },
        Commands::Task { add: None } => Command::ListTasks,
        Commands::Tasks => Command::ListTasks,
    };
    
    // Send command and get response
    let response = send_command(command).await?;
    
    // Handle response
    match response {
        Response::Ok => println!("OK"),
        Response::Status(status) => {
            println!("State: {:?}", status.state);
            println!("Elapsed: {}s", status.elapsed);
        }
        Response::Tasks(tasks) => {
            for task in tasks {
                let check = if task.completed { "✓" } else { " " };
                println!("[{}] {}: {}", check, task.id, task.description);
            }
        }
        Response::Error(e) => eprintln!("Error: {}", e),
    }
    
    Ok(())
}

async fn send_command(cmd: Command) -> Result<Response> {
    let mut stream = UnixStream::connect(SOCKET_PATH).await?;
    
    // Send command
    let msg = serde_json::to_vec(&cmd)?;
    stream.write_all(&msg).await?;
    stream.write_all(b"\n").await?;
    
    // Read response
    let mut buf = vec![0; 1024];
    let n = stream.read(&mut buf).await?;
    let response: Response = serde_json::from_slice(&buf[..n])?;
    
    Ok(response)
}
