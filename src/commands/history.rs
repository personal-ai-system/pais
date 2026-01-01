use colored::*;
use eyre::Result;

use crate::cli::HistoryAction;
use crate::config::Config;

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
    _json: bool,
    config: &Config,
) -> Result<()> {
    println!("{} Searching history for: {}", "🔍".blue(), query.cyan());

    if let Some(cat) = category {
        println!("  Category: {}", cat);
    }
    println!("  Limit: {}", limit);
    if let Some(date) = since {
        println!("  Since: {}", date);
    }

    let history_dir = Config::expand_path(&config.paths.history);
    println!("  History dir: {}", history_dir.display().to_string().dimmed());

    // TODO: Implement history search using ripgrep
    println!("  {} History search not yet implemented", "⚠".yellow());

    Ok(())
}

fn recent(category: Option<&str>, count: usize, config: &Config) -> Result<()> {
    println!("{} Recent history entries", "📋".blue());

    if let Some(cat) = category {
        println!("  Category: {}", cat);
    }
    println!("  Count: {}", count);

    let history_dir = Config::expand_path(&config.paths.history);
    println!("  History dir: {}", history_dir.display().to_string().dimmed());

    // TODO: Implement recent history listing
    println!("  {} Recent history not yet implemented", "⚠".yellow());

    Ok(())
}

fn categories(config: &Config) -> Result<()> {
    println!("{}", "History categories:".bold());
    println!();

    let history_dir = Config::expand_path(&config.paths.history);

    if !history_dir.exists() {
        println!("  {}", "(no history yet)".dimmed());
        return Ok(());
    }

    // List subdirectories as categories
    for entry in std::fs::read_dir(&history_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            // Count entries in category
            let count = std::fs::read_dir(entry.path())?
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                .flat_map(|e| std::fs::read_dir(e.path()))
                .flatten()
                .filter_map(|e| e.ok())
                .count();

            println!("  {:15} ({} entries)", name_str.cyan(), count);
        }
    }

    Ok(())
}
