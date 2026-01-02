use chrono::NaiveDate;
use colored::*;
use eyre::{Context, Result};

use crate::cli::HistoryAction;
use crate::config::Config;
use crate::history::HistoryStore;

pub fn run(action: HistoryAction, config: &Config) -> Result<()> {
    match action {
        HistoryAction::Query {
            query,
            category,
            limit,
            since,
            json,
        } => query_history(&query, category.as_deref(), limit, since.as_deref(), json, config),
        HistoryAction::Recent { category, count } => recent(category.as_deref(), count, config),
        HistoryAction::Categories => categories(config),
    }
}

fn query_history(
    query: &str,
    category: Option<&str>,
    limit: usize,
    since: Option<&str>,
    json: bool,
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

    if json {
        let output: Vec<serde_json::Value> = entries
            .iter()
            .map(|e| {
                serde_json::json!({
                    "id": e.id,
                    "category": e.category,
                    "title": e.title,
                    "created_at": e.created_at.format("%Y-%m-%dT%H:%M:%S%z").to_string(),
                    "tags": e.tags,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
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
    println!("  {} {} {}", entry.category.cyan(), date.dimmed(), entry.title.bold());
    if !entry.tags.is_empty() {
        println!("    tags: {}", entry.tags.join(", ").dimmed());
    }
}
