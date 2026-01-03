use chrono::NaiveDate;
use colored::*;
use eyre::{Context, Result};
use serde::Serialize;
use std::fs;

use crate::cli::{HistoryAction, OutputFormat};
use crate::config::Config;
use crate::history::HistoryStore;
use crate::history::capture::EventCapture;

pub fn run(action: HistoryAction, config: &Config) -> Result<()> {
    match action {
        HistoryAction::Query {
            query,
            category,
            limit,
            since,
            format,
        } => query_history(
            &query,
            category.as_deref(),
            limit,
            since.as_deref(),
            OutputFormat::resolve(format),
            config,
        ),
        HistoryAction::Recent { category, count } => recent(category.as_deref(), count, config),
        HistoryAction::Categories => categories(config),
        HistoryAction::Show { id } => show_entry(&id, config),
        HistoryAction::Stats { days, format } => stats(days, OutputFormat::resolve(format), config),
        HistoryAction::Events { limit } => list_events(limit, config),
    }
}

#[derive(Serialize)]
struct HistoryEntryOutput {
    id: String,
    category: String,
    title: String,
    created_at: String,
    tags: Vec<String>,
}

fn query_history(
    query: &str,
    category: Option<&str>,
    limit: usize,
    since: Option<&str>,
    format: OutputFormat,
    config: &Config,
) -> Result<()> {
    let history_dir = Config::expand_path(&config.paths.history);
    let store = HistoryStore::new(history_dir);

    // Parse since date if provided
    let since_date = since
        .map(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d"))
        .transpose()
        .context("Invalid date format (use YYYY-MM-DD)")?;

    let entries = store.query(query, category, since_date, limit)?;

    match format {
        OutputFormat::Json | OutputFormat::Yaml => {
            let output: Vec<HistoryEntryOutput> = entries
                .iter()
                .map(|e| HistoryEntryOutput {
                    id: e.id.clone(),
                    category: e.category.clone(),
                    title: e.title.clone(),
                    created_at: e.created_at.format("%Y-%m-%dT%H:%M:%S%z").to_string(),
                    tags: e.tags.clone(),
                })
                .collect();
            match format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&output)?),
                OutputFormat::Yaml => println!("{}", serde_yaml::to_string(&output)?),
                _ => unreachable!(),
            }
        }
        OutputFormat::Text => {
            println!(
                "{} Found {} entries matching '{}':",
                "🔍".blue(),
                entries.len(),
                query.cyan()
            );
            println!();

            if entries.is_empty() {
                println!("  {}", "(no matches)".dimmed());
            } else {
                for entry in &entries {
                    print_entry_summary(entry);
                }
            }
        }
    }

    Ok(())
}

fn recent(category: Option<&str>, count: usize, config: &Config) -> Result<()> {
    let history_dir = Config::expand_path(&config.paths.history);
    let store = HistoryStore::new(history_dir);

    let entries = store.recent(category, count)?;

    println!("{} Recent history entries:", "📋".blue());
    println!();

    if entries.is_empty() {
        println!("  {}", "(no history yet)".dimmed());
    } else {
        for entry in &entries {
            print_entry_summary(entry);
        }
    }

    Ok(())
}

fn categories(config: &Config) -> Result<()> {
    println!("{}", "History categories:".bold());
    println!();

    let history_dir = Config::expand_path(&config.paths.history);
    let store = HistoryStore::new(history_dir);

    let cats = store.categories()?;

    if cats.is_empty() {
        println!("  {}", "(no history yet)".dimmed());
        return Ok(());
    }

    for cat in cats {
        let count = store.count(&cat)?;
        println!("  {:15} ({} entries)", cat.cyan(), count);
    }

    Ok(())
}

fn print_entry_summary(entry: &crate::history::HistoryEntry) {
    let date = entry.created_at.format("%Y-%m-%d %H:%M").to_string();
    println!(
        "  {} {} {} {}",
        entry.id[..8.min(entry.id.len())].dimmed(),
        entry.category.cyan(),
        date.dimmed(),
        entry.title.bold()
    );
    if !entry.tags.is_empty() {
        println!("    tags: {}", entry.tags.join(", ").dimmed());
    }
}

/// Show a specific history entry
fn show_entry(id: &str, config: &Config) -> Result<()> {
    let history_dir = Config::expand_path(&config.paths.history);
    let store = HistoryStore::new(history_dir.clone());

    // Search all categories for the entry
    let cats = store.categories()?;

    for cat in &cats {
        let cat_path = history_dir.join(cat);
        if !cat_path.exists() {
            continue;
        }

        // Search date directories
        for date_entry in fs::read_dir(&cat_path)? {
            let date_entry = date_entry?;
            let date_path = date_entry.path();

            if !date_path.is_dir() {
                continue;
            }

            for file_entry in fs::read_dir(&date_path)? {
                let file_entry = file_entry?;
                let path = file_entry.path();

                if let Some(stem) = path.file_stem().and_then(|s| s.to_str())
                    && (stem == id || stem.starts_with(id))
                {
                    // Found it!
                    let content = fs::read_to_string(&path)?;
                    println!("{}", content);
                    return Ok(());
                }
            }
        }
    }

    eyre::bail!("Entry '{}' not found", id)
}

/// Show event statistics
fn stats(days: usize, format: OutputFormat, config: &Config) -> Result<()> {
    let history_dir = Config::expand_path(&config.paths.history);
    let capture = EventCapture::new(history_dir, true);

    let stats = capture.stats(days)?;

    match format {
        OutputFormat::Json => {
            #[derive(Serialize)]
            struct StatsOutput {
                total: usize,
                by_type: std::collections::HashMap<String, usize>,
                days: usize,
            }
            let output = StatsOutput {
                total: stats.total,
                by_type: stats.by_type.clone(),
                days,
            };
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Yaml => {
            #[derive(Serialize)]
            struct StatsOutput {
                total: usize,
                by_type: std::collections::HashMap<String, usize>,
                days: usize,
            }
            let output = StatsOutput {
                total: stats.total,
                by_type: stats.by_type.clone(),
                days,
            };
            println!("{}", serde_yaml::to_string(&output)?);
        }
        OutputFormat::Text => {
            println!("{} Event statistics (last {} days):", "📊".blue(), days);
            println!();
            println!("  Total events: {}", stats.total.to_string().bold());
            println!();

            if stats.by_type.is_empty() {
                println!("  {}", "(no events captured)".dimmed());
            } else {
                println!("  By type:");
                let mut types: Vec<_> = stats.by_type.iter().collect();
                types.sort_by(|a, b| b.1.cmp(a.1));
                for (event_type, count) in types {
                    println!("    {:20} {}", event_type.cyan(), count);
                }
            }
        }
    }

    Ok(())
}

/// List available raw event dates
fn list_events(limit: usize, config: &Config) -> Result<()> {
    let history_dir = Config::expand_path(&config.paths.history);
    let capture = EventCapture::new(history_dir, true);

    let dates = capture.list_dates()?;

    println!("{} Raw event logs:", "📅".blue());
    println!();

    if dates.is_empty() {
        println!("  {}", "(no events captured)".dimmed());
    } else {
        for date in dates.iter().take(limit) {
            if let Ok(events) = capture.read_events(date) {
                println!("  {} ({} events)", date.cyan(), events.len());
            } else {
                println!("  {}", date.cyan());
            }
        }

        if dates.len() > limit {
            println!();
            println!("  {} more dates...", dates.len() - limit);
        }
    }

    Ok(())
}
