use anyhow::Result;
use clap::{Parser, Subcommand};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use monitor_rs::{
    collectors::{
        CpuCollector, DiskCollector, MemoryCollector, MetricCollector, NetworkCollector,
        ProcessCollector, Snapshot,
    },
    config::Config,
    ui::App,
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use tracing::{info, Level};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[cfg(feature = "server")]
use monitor_rs::export::start_server;

#[derive(Parser, Debug)]
#[command(name = "monitor-rs")]
#[command(version, about = "Real-time system monitor with Prometheus integration", long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    config: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run in TUI mode (interactive terminal UI)
    Tui,

    /// Run in server mode (Prometheus metrics export)
    Server {
        #[arg(short, long, default_value = "0.0.0.0:9100")]
        listen: String,
    },

    /// Show current system snapshot
    Snapshot,

    /// Generate default configuration file
    GenerateConfig {
        #[arg(short, long, default_value = "config.toml")]
        output: String,
    },
}

fn init_tracing(log_level: &str) {
    let level = match log_level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(level.to_string()));

    tracing_subscriber::registry()
        .with(fmt::layer().with_target(false).compact())
        .with(env_filter)
        .init();
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let config = match &cli.config {
        Some(path) => Config::load_from_file(path)?,
        None => Config::load().unwrap_or_default(),
    };

    init_tracing(&config.general.log_level);

    info!("Starting monitor-rs v{}", env!("CARGO_PKG_VERSION"));
    info!("Configuration loaded: log_level={}", config.general.log_level);

    match cli.command {
        Some(Commands::Tui) => {
            info!("Starting TUI mode");
            run_tui()?;
            Ok(())
        }
        Some(Commands::Server { listen }) => {
            #[cfg(feature = "server")]
            {
                info!("Starting server mode on {}", listen);
                let addr: std::net::SocketAddr = listen.parse()?;
                start_server(config, addr).await?;
                Ok(())
            }
            #[cfg(not(feature = "server"))]
            {
                eprintln!("Error: Server feature not enabled. Rebuild with --features server");
                std::process::exit(1);
            }
        }
        Some(Commands::Snapshot) => {
            info!("Collecting system snapshot");
            collect_snapshot()?;
            Ok(())
        }
        Some(Commands::GenerateConfig { output }) => {
            info!("Generating config file: {}", output);
            generate_config(&output)?;
            Ok(())
        }
        None => {
            info!("Starting default mode (TUI)");
            run_tui()?;
            Ok(())
        }
    }
}

fn collect_snapshot() -> Result<()> {
    let mut cpu_collector = CpuCollector::new();
    let mut memory_collector = MemoryCollector::new();
    let mut process_collector = ProcessCollector::new();
    let mut network_collector = NetworkCollector::new();
    let mut disk_collector = DiskCollector::new();

    let cpu_metrics = cpu_collector.collect()?;
    let memory_metrics = memory_collector.collect()?;
    let process_metrics = process_collector.collect()?;
    let network_metrics = network_collector.collect()?;
    let disk_metrics = disk_collector.collect()?;

    let snapshot = Snapshot::new()
        .with_cpu(cpu_metrics)
        .with_memory(memory_metrics)
        .with_processes(process_metrics)
        .with_network(network_metrics)
        .with_disk(disk_metrics);

    println!("\n=== System Snapshot ===");
    println!("Timestamp: {}", snapshot.timestamp);

    if let Some(cpu) = &snapshot.cpu {
        println!("\n--- CPU ---");
        println!("Total Usage: {:.2}%", cpu.total_usage);
        println!("Core Count: {}", cpu.core_count);
        println!(
            "Load Average: {:.2} {:.2} {:.2}",
            cpu.load_average.0, cpu.load_average.1, cpu.load_average.2
        );
        println!("Per-core usage:");
        for (i, usage) in cpu.per_core.iter().enumerate() {
            println!("  CPU {}: {:.2}%", i, usage);
        }
    }

    if let Some(memory) = &snapshot.memory {
        println!("\n--- Memory ---");
        println!("Total: {}", memory.total_formatted());
        println!("Used: {} ({:.2}%)", memory.used_formatted(), memory.usage_percent);
        println!("Available: {}", memory.available_formatted());

        if memory.swap_total > 0 {
            println!("\n--- Swap ---");
            println!(
                "Total: {}",
                monitor_rs::collectors::MemoryMetrics::format_bytes(memory.swap_total)
            );
            println!(
                "Used: {} ({:.2}%)",
                monitor_rs::collectors::MemoryMetrics::format_bytes(memory.swap_used),
                memory.swap_usage_percent
            );
        }
    }

    if let Some(network) = &snapshot.network {
        println!("\n--- Network ---");
        println!("Total RX: {}", monitor_rs::collectors::MemoryMetrics::format_bytes(network.total_received_bytes));
        println!("Total TX: {}", monitor_rs::collectors::MemoryMetrics::format_bytes(network.total_transmitted_bytes));
        println!("RX Rate: {}", monitor_rs::collectors::NetworkMetrics::format_rate(network.total_received_rate));
        println!("TX Rate: {}", monitor_rs::collectors::NetworkMetrics::format_rate(network.total_transmitted_rate));

        println!("\nActive Interfaces:");
        let mut interfaces: Vec<_> = network.interfaces.iter().collect();
        interfaces.sort_by_key(|(name, _)| *name);

        for (name, iface) in interfaces.iter().take(5) {
            println!(
                "  {} - RX: {} ({}) TX: {} ({})",
                name,
                monitor_rs::collectors::MemoryMetrics::format_bytes(iface.received_bytes),
                monitor_rs::collectors::NetworkMetrics::format_rate(iface.received_rate),
                monitor_rs::collectors::MemoryMetrics::format_bytes(iface.transmitted_bytes),
                monitor_rs::collectors::NetworkMetrics::format_rate(iface.transmitted_rate)
            );
        }
    }

    if let Some(disk) = &snapshot.disk {
        println!("\n--- Disk ---");
        println!("Total: {}", disk.total_capacity_formatted());
        println!("Used: {} ({:.2}%)", disk.total_used_formatted(), disk.overall_usage_percent);
        println!("Available: {}", disk.total_available_formatted());

        if !disk.disks.is_empty() {
            println!("\nMounted Disks:");
            for disk_info in &disk.disks {
                let status = if disk_info.is_critical() {
                    "⚠️  CRITICAL"
                } else if disk_info.is_warning() {
                    "⚠️  WARNING"
                } else {
                    "✓"
                };

                println!(
                    "  {} {} - {} used of {} ({:.1}%) {}",
                    status,
                    disk_info.mount_point,
                    disk_info.used_formatted(),
                    disk_info.total_formatted(),
                    disk_info.usage_percent,
                    disk_info.disk_kind
                );
            }
        }
    }

    if let Some(processes) = &snapshot.processes {
        println!("\n--- Processes ---");
        println!("Total: {}", processes.total_processes);
        println!("Running: {}", processes.running_processes);

        println!("\nTop 10 by CPU:");
        for (i, proc) in processes.top_cpu(10).iter().enumerate() {
            let service_tag = match &proc.service_type {
                Some(svc) => format!(" [{}]", svc),
                None => String::new(),
            };
            println!(
                "  {}. {} (PID {}) - CPU: {:.2}%, Mem: {:.2}%{}",
                i + 1,
                proc.name,
                proc.pid,
                proc.cpu_usage,
                proc.memory_percent,
                service_tag
            );
        }

        let services = processes.list_services();
        if !services.is_empty() {
            println!("\n--- Detected Services ---");
            for service_name in &services {
                if let Some(stats) = processes.service_stats(service_name) {
                    println!(
                        "  {} - {} process(es), CPU: {:.2}%, Memory: {}",
                        service_name,
                        stats.process_count,
                        stats.total_cpu_usage,
                        monitor_rs::collectors::MemoryMetrics::format_bytes(stats.total_memory)
                    );
                }
            }
        }
    }

    println!();
    Ok(())
}

fn generate_config(output: &str) -> Result<()> {
    let config = Config::default();
    let toml_string = toml::to_string_pretty(&config)?;

    std::fs::write(output, toml_string)?;
    println!("Configuration file generated: {}", output);
    println!("\nYou can now edit this file and use it with:");
    println!("  monitor-rs --config {}", output);

    Ok(())
}

fn run_tui() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let mut app = App::new()?;
    let res = app.run(&mut terminal);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(res?)
}
