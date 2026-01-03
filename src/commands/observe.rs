//! Live event observation command
//!
//! Tails the event log in real-time, similar to `tail -f`.

use chrono::Local;
use colored::*;
use eyre::{Context, Result};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::Path;
use std::thread;
use std::time::Duration;

use crate::config::Config;
use crate::observability::Event;

/// Run the observe command
pub fn run(filter: Option<&str>, last: usize, include_payload: bool, config: &Config) -> Result<()> {
    let history_path = Config::expand_path(&config.paths.history);
    let events_dir = history_path.join("raw-events");

    println!("{} Observing events (Ctrl+C to stop)...", "👁".blue());
    if let Some(f) = filter {
        println!("  Filter: {}", f.cyan());
    }
    println!();

    // Show last N events first
    if last > 0 {
        show_recent_events(&events_dir, last, filter, include_payload)?;
        println!("{}", "--- Live tail ---".dimmed());
        println!();
    }

    // Now tail the current day's file
    tail_events(&events_dir, filter, include_payload)?;

    Ok(())
}

/// Show the last N events from recent log files
fn show_recent_events(events_dir: &Path, count: usize, filter: Option<&str>, include_payload: bool) -> Result<()> {
    let mut all_events = Vec::new();

    // Get today's and yesterday's log files
    let today = Local::now();
    let yesterday = today - chrono::Duration::days(1);

    for date in [yesterday, today] {
        let month_dir = events_dir.join(date.format("%Y-%m").to_string());
        let log_file = month_dir.join(format!("{}.jsonl", date.format("%Y-%m-%d")));

        if log_file.exists()
            && let Ok(content) = fs::read_to_string(&log_file)
        {
            for line in content.lines() {
                if line.trim().is_empty() {
                    continue;
                }
                if let Ok(event) = serde_json::from_str::<Event>(line) {
                    // Apply filter - skip if doesn't match
                    if let Some(f) = filter
                        && !event.event_type.to_lowercase().contains(&f.to_lowercase())
                    {
                        continue;
                    }
                    all_events.push(event);
                }
            }
        }
    }

    // Take last N events
    let start = all_events.len().saturating_sub(count);
    for event in &all_events[start..] {
        print_event(event, include_payload);
    }

    Ok(())
}

/// Tail the current day's log file
fn tail_events(events_dir: &Path, filter: Option<&str>, include_payload: bool) -> Result<()> {
    loop {
        let today = Local::now();
        let month_dir = events_dir.join(today.format("%Y-%m").to_string());
        let log_file = month_dir.join(format!("{}.jsonl", today.format("%Y-%m-%d")));

        if !log_file.exists() {
            // Wait for file to be created
            thread::sleep(Duration::from_secs(1));
            continue;
        }

        // Open file and seek to end
        let file = File::open(&log_file).context("Failed to open log file")?;
        let mut reader = BufReader::new(file);

        // Seek to end
        reader.seek(SeekFrom::End(0))?;

        // Read new lines as they appear
        let mut line = String::new();
        loop {
            match reader.read_line(&mut line) {
                Ok(0) => {
                    // No new data, wait a bit
                    thread::sleep(Duration::from_millis(100));

                    // Check if we've crossed midnight
                    let now = Local::now();
                    if now.format("%Y-%m-%d").to_string() != today.format("%Y-%m-%d").to_string() {
                        // New day, switch to new file
                        break;
                    }
                }
                Ok(_) => {
                    let trimmed = line.trim();
                    if !trimmed.is_empty()
                        && let Ok(event) = serde_json::from_str::<Event>(trimmed)
                    {
                        // Apply filter
                        let should_show = match filter {
                            Some(f) => event.event_type.to_lowercase().contains(&f.to_lowercase()),
                            None => true,
                        };

                        if should_show {
                            print_event(&event, include_payload);
                        }
                    }
                    line.clear();
                }
                Err(e) => {
                    log::warn!("Error reading log file: {}", e);
                    thread::sleep(Duration::from_secs(1));
                }
            }
        }
    }
}

/// Print a single event
fn print_event(event: &Event, include_payload: bool) {
    println!("{}", event.format_display());

    if include_payload && let Some(ref payload) = event.payload {
        let pretty = serde_json::to_string_pretty(payload).unwrap_or_default();
        for line in pretty.lines() {
            println!("  {}", line.dimmed());
        }
    }
}
