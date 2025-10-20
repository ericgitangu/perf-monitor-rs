use crate::collectors::*;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::time::{Duration, Instant};

pub struct App {
    pub should_quit: bool,
    pub cpu_collector: CpuCollector,
    pub memory_collector: MemoryCollector,
    pub process_collector: ProcessCollector,
    pub network_collector: NetworkCollector,
    pub disk_collector: DiskCollector,
    pub last_update: Instant,
    pub update_interval: Duration,
}

impl App {
    pub fn new() -> crate::Result<Self> {
        Ok(Self {
            should_quit: false,
            cpu_collector: CpuCollector::new(),
            memory_collector: MemoryCollector::new(),
            process_collector: ProcessCollector::new(),
            network_collector: NetworkCollector::new(),
            disk_collector: DiskCollector::new(),
            last_update: Instant::now(),
            update_interval: Duration::from_secs(1),
        })
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> crate::Result<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key_event(key);
                }
            }

            // Update metrics at regular intervals
            if self.last_update.elapsed() >= self.update_interval {
                self.update_metrics()?;
                self.last_update = Instant::now();
            }

            if self.should_quit {
                break;
            }
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
            }
            KeyCode::Char('r') => {
                // Force refresh
                self.last_update = Instant::now() - self.update_interval;
            }
            _ => {}
        }
    }

    fn update_metrics(&mut self) -> crate::Result<()> {
        // Update all collectors
        let _ = self.cpu_collector.collect();
        let _ = self.memory_collector.collect();
        let _ = self.process_collector.collect();
        let _ = self.network_collector.collect();
        let _ = self.disk_collector.collect();
        Ok(())
    }

    fn ui(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Min(10),     // Main content
                Constraint::Length(3),  // Footer
            ])
            .split(f.size());

        // Title
        let title = Paragraph::new("Monitor-RS - System Monitor (Press 'q' to quit, 'r' to refresh)")
            .block(Block::default().borders(Borders::ALL).title("Monitor-RS"));
        f.render_widget(title, chunks[0]);

        // Main content area
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[1]);

        // Left side: System metrics
        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(34),
            ])
            .split(main_chunks[0]);

        self.render_cpu(f, left_chunks[0]);
        self.render_memory(f, left_chunks[1]);
        self.render_network(f, left_chunks[2]);

        // Right side: Processes and disk
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(main_chunks[1]);

        self.render_processes(f, right_chunks[0]);
        self.render_disk(f, right_chunks[1]);

        // Footer
        let footer = Paragraph::new("Q: Quit | R: Refresh")
            .block(Block::default().borders(Borders::ALL).title("Controls"));
        f.render_widget(footer, chunks[2]);
    }

    fn render_cpu(&mut self, f: &mut Frame, area: Rect) {
        if let Ok(metrics) = self.cpu_collector.collect() {
            let text = format!(
                "CPU Usage: {:.2}%\nCores: {}\nLoad Avg: {:.2} {:.2} {:.2}",
                metrics.total_usage,
                metrics.core_count,
                metrics.load_average.0,
                metrics.load_average.1,
                metrics.load_average.2
            );
            let widget = Paragraph::new(text)
                .block(Block::default().borders(Borders::ALL).title("CPU"));
            f.render_widget(widget, area);
        }
    }

    fn render_memory(&mut self, f: &mut Frame, area: Rect) {
        if let Ok(metrics) = self.memory_collector.collect() {
            let text = format!(
                "Total: {}\nUsed: {} ({:.2}%)\nSwap: {} / {}",
                MemoryMetrics::format_bytes(metrics.total),
                MemoryMetrics::format_bytes(metrics.used),
                metrics.usage_percent,
                MemoryMetrics::format_bytes(metrics.swap_used),
                MemoryMetrics::format_bytes(metrics.swap_total)
            );
            let widget = Paragraph::new(text)
                .block(Block::default().borders(Borders::ALL).title("Memory"));
            f.render_widget(widget, area);
        }
    }

    fn render_network(&mut self, f: &mut Frame, area: Rect) {
        if let Ok(metrics) = self.network_collector.collect() {
            let text = format!(
                "Total RX: {}\nTotal TX: {}\nRX Rate: {}\nTX Rate: {}",
                MemoryMetrics::format_bytes(metrics.total_received_bytes),
                MemoryMetrics::format_bytes(metrics.total_transmitted_bytes),
                NetworkMetrics::format_rate(metrics.total_received_rate),
                NetworkMetrics::format_rate(metrics.total_transmitted_rate)
            );
            let widget = Paragraph::new(text)
                .block(Block::default().borders(Borders::ALL).title("Network"));
            f.render_widget(widget, area);
        }
    }

    fn render_processes(&mut self, f: &mut Frame, area: Rect) {
        if let Ok(metrics) = self.process_collector.collect() {
            let mut text = format!("Total Processes: {}\n\n", metrics.total_processes);

            let services = metrics.list_services();
            for service_name in services.iter().take(8) {
                if let Some(stats) = metrics.service_stats(service_name) {
                    text.push_str(&format!(
                        "{}: {} procs, {}\n",
                        service_name,
                        stats.process_count,
                        MemoryMetrics::format_bytes(stats.total_memory)
                    ));
                }
            }

            let widget = Paragraph::new(text)
                .block(Block::default().borders(Borders::ALL).title("Services"));
            f.render_widget(widget, area);
        }
    }

    fn render_disk(&mut self, f: &mut Frame, area: Rect) {
        if let Ok(metrics) = self.disk_collector.collect() {
            let mut text = format!(
                "Total: {} ({:.2}%)\n\n",
                DiskMetrics::format_bytes(metrics.total_space),
                metrics.overall_usage_percent
            );

            for disk in metrics.disks.iter().take(6) {
                let status = if disk.is_critical() {
                    "CRIT"
                } else if disk.is_warning() {
                    "WARN"
                } else {
                    "OK"
                };

                text.push_str(&format!(
                    "{} {} {:.1}%\n",
                    status,
                    disk.mount_point
                        .chars()
                        .take(20)
                        .collect::<String>(),
                    disk.usage_percent
                ));
            }

            let widget = Paragraph::new(text)
                .block(Block::default().borders(Borders::ALL).title("Disk"));
            f.render_widget(widget, area);
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new().unwrap()
    }
}
