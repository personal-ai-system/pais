#!/usr/bin/env python3
"""
PR Review Finder - Find PR review requests in Slack channels.

This plugin searches Slack channels for messages where people are asking
for PR reviews - a potential process smell indicating review bottlenecks.

Usage:
    pais run pr-review-finder search [--days N] [--channel CHANNEL_ID]
    pais run pr-review-finder list-channels
"""

import argparse
import json
import re
import sys
from datetime import datetime, timedelta
from pathlib import Path

import yaml

# PR review request patterns - things people say when begging for reviews
REVIEW_PATTERNS = [
    r"👀",
    r":eyes:",
    r"eyes on",
    r"review",
    r"please.*look",
    r"look.*at",
    r"need.*review",
    r"can.*someone",
    r"anyone.*review",
    r"could.*get",
    r"would.*appreciate",
    r"help.*review",
    r"waiting.*review",
    r"blocked.*review",
    r"ready.*review",
    r"PTAL",  # Please Take A Look
    r"lgtm\?",
    r"thoughts\?",
]

# GitHub PR URL pattern
GITHUB_PR_PATTERN = r"https?://github\.com/[^/]+/[^/]+/pull/\d+"


def load_channels(plugin_dir: Path) -> dict[str, str]:
    """Load channel mappings from channels.yaml."""
    channels_file = plugin_dir / "channels.yaml"
    if not channels_file.exists():
        print(f"Error: {channels_file} not found", file=sys.stderr)
        sys.exit(1)

    with open(channels_file) as f:
        data = yaml.safe_load(f)

    return data.get("channels", {})


def is_review_request(text: str) -> bool:
    """Check if message text looks like a PR review request."""
    text_lower = text.lower()
    for pattern in REVIEW_PATTERNS:
        if re.search(pattern, text_lower, re.IGNORECASE):
            return True
    return False


def extract_pr_urls(text: str, repo_pattern: str | None = None) -> list[str]:
    """Extract GitHub PR URLs from message text."""
    urls = re.findall(GITHUB_PR_PATTERN, text)
    if repo_pattern:
        urls = [u for u in urls if re.search(repo_pattern, u)]
    return urls


def format_timestamp(ts: str) -> str:
    """Convert Slack timestamp to human-readable format."""
    try:
        unix_ts = float(ts.split(".")[0])
        dt = datetime.fromtimestamp(unix_ts)
        return dt.strftime("%Y-%m-%d %H:%M")
    except (ValueError, IndexError):
        return ts


def cmd_list_channels(plugin_dir: Path) -> None:
    """List configured channels."""
    channels = load_channels(plugin_dir)
    print("Configured channels:")
    print("-" * 50)
    for channel_id, name in sorted(channels.items(), key=lambda x: x[1]):
        print(f"  {channel_id}: {name}")
    print(f"\nTotal: {len(channels)} channels")
    print(f"\nEdit {plugin_dir / 'channels.yaml'} to add/remove channels")


def cmd_search(
    plugin_dir: Path,
    days: int = 30,
    channel_filter: str | None = None,
    repo_pattern: str | None = None,
) -> None:
    """
    Search for PR review requests in Slack channels.

    This outputs instructions for Claude Code to use the Slack MCP tools
    to search the channels, since we can't directly call MCP from Python.
    """
    channels = load_channels(plugin_dir)

    if channel_filter:
        # Filter to specific channel
        if channel_filter in channels:
            channels = {channel_filter: channels[channel_filter]}
        else:
            # Try to match by name
            matched = {k: v for k, v in channels.items() if channel_filter.lower() in v.lower()}
            if matched:
                channels = matched
            else:
                print(f"Error: Channel '{channel_filter}' not found", file=sys.stderr)
                sys.exit(1)

    # Calculate the oldest timestamp we care about
    cutoff_date = datetime.now() - timedelta(days=days)
    cutoff_ts = cutoff_date.timestamp()

    # Output search instructions as JSON for Claude Code to execute
    search_config = {
        "action": "search_slack_for_pr_reviews",
        "channels": channels,
        "lookback_days": days,
        "cutoff_timestamp": cutoff_ts,
        "repo_pattern": repo_pattern or "github.com/tatari-tv/.*/pull/",
        "review_patterns": REVIEW_PATTERNS,
        "instructions": f"""
Search these Slack channels for PR review requests from the last {days} days.

For each channel, use mcp__slack__slack_get_channel_history to get messages,
then look for messages that:
1. Contain GitHub PR URLs matching: {repo_pattern or 'github.com/tatari-tv/*/pull/*'}
2. Contain review request patterns like: eyes, 👀, review, please look, etc.

Report findings as:
- Channel name
- Message author
- Date/time
- PR URL(s)
- Message snippet

Channels to search:
""",
    }

    for channel_id, name in channels.items():
        search_config["instructions"] += f"  - {name} ({channel_id})\n"

    print(json.dumps(search_config, indent=2))


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Find PR review requests in Slack channels",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    subparsers = parser.add_subparsers(dest="command", help="Commands")

    # search command
    search_parser = subparsers.add_parser("search", help="Search for PR review requests")
    search_parser.add_argument(
        "--days",
        type=int,
        default=30,
        help="Number of days to look back (default: 30)",
    )
    search_parser.add_argument(
        "--channel",
        type=str,
        help="Filter to specific channel ID or name",
    )
    search_parser.add_argument(
        "--repo-pattern",
        type=str,
        help="Regex pattern to filter GitHub repos (default: github.com/tatari-tv/.*/pull/)",
    )

    # list-channels command
    subparsers.add_parser("list-channels", help="List configured channels")

    args = parser.parse_args()

    # Find plugin directory (where this script lives)
    plugin_dir = Path(__file__).parent.parent

    if args.command == "list-channels":
        cmd_list_channels(plugin_dir)
    elif args.command == "search":
        cmd_search(
            plugin_dir,
            days=args.days,
            channel_filter=args.channel,
            repo_pattern=args.repo_pattern,
        )
    else:
        parser.print_help()
        sys.exit(1)


if __name__ == "__main__":
    main()
