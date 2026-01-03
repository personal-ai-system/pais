---
name: git-tools
description: Scott's git productivity tools. Use when cloning repos, finding repos, checking CODEOWNERS, finding stale branches/PRs, or working with GitHub organization management.
---

# Git Tools

This skill teaches you Scott's custom git productivity CLI tools. These are Rust CLIs that extend git for common SRE and repository management workflows.

## Quick Reference

| Tool | Purpose |
|------|---------|
| `clone` | Smart git clone with SSH key per-org, versioning, mirrors |
| `reposlug` | Extract `owner/repo` from a git remote |
| `ls-git-repos` | Find all local git repos recursively |
| `ls-github-repos` | List all repos under a GitHub org/user |
| `ls-owners` | Analyze CODEOWNERS, detect unowned paths |
| `ls-stale-branches` | Find branches older than N days |
| `ls-stale-prs` | Find PRs older than N days |

---

## clone

**Smart git clone with org-specific SSH keys, versioning, and mirror support.**

```bash
clone <repospec> [revision]
clone scottidler/pais                    # Clone to ./scottidler/pais
clone scottidler/pais main               # Clone and checkout main
clone scottidler/pais --versioning       # Clone to scottidler/pais/<sha>
clone scottidler/pais --mirrorpath ~/mirrors  # Use local mirror for speed
```

### Key Features

- **Org-specific SSH keys**: Configure `~/.config/clone/clone.cfg` with per-org SSH keys
- **Auto-stash**: If updating an existing repo with changes, auto-stashes them
- **Versioning mode**: Creates `repo/sha` structure for pinned checkouts
- **Mirror support**: `--mirrorpath` for fast clones from local bare repos

### Config File (`~/.config/clone/clone.cfg`)

```ini
[org.default]
sshkey = ~/.ssh/id_ed25519

[org.mycompany]
sshkey = ~/.ssh/mycompany_ed25519
```

---

## reposlug

**Extract the `owner/repo` slug from the current directory's git remote.**

```bash
reposlug              # In a git repo, prints "owner/repo"
reposlug /path/to/repo
```

Useful for scripting and piping into other commands.

---

## ls-git-repos

**Recursively find all local git repositories and list their reposlugs.**

```bash
ls-git-repos              # Search from current dir
ls-git-repos ~/repos      # Search from specific path
ls-git-repos ~/repos | wc -l  # Count repos
```

Useful for finding what you have cloned locally.

---

## ls-github-repos

**List all repositories under a GitHub organization or user.**

```bash
ls-github-repos scottidler           # List all repos for user
ls-github-repos mycompany            # List all repos for org
ls-github-repos mycompany -A         # Include archived repos
ls-github-repos mycompany -a         # Show with creation date
```

### Requirements

- GitHub token at `~/.config/github/tokens/<name>` (where `<name>` is the org/user)
- Auto-detects if the name is a user or organization

---

## ls-owners

**Analyze CODEOWNERS files and detect unowned code paths.**

```bash
ls-owners                    # Check current repo
ls-owners ~/repos            # Check all repos under path
ls-owners -o unowned         # Show only unowned repos
ls-owners -o partial         # Show only partially owned
ls-owners -d                 # Detailed output with paths
```

### Output

- **owned**: All code paths have CODEOWNERS entries
- **partial**: Some paths are unowned
- **unowned**: Missing or empty CODEOWNERS

Shows top authors (from git history) for unowned repos to suggest owners.

### Config

List ex-employees to exclude from author suggestions:
`~/.config/ls-owners/<org>/ex-employees` (one name per line)

---

## ls-stale-branches

**Find remote branches that haven't been updated in N days.**

```bash
ls-stale-branches 30           # Branches untouched for 30+ days
ls-stale-branches 60 ~/repos   # Check multiple repos
ls-stale-branches 90 -d        # Detailed YAML output
```

### Output (default)

```
org/repo:
  author1: (count, max_age_days)
  author2: (count, max_age_days)
```

### Output (detailed)

Full YAML with each branch name and age.

---

## ls-stale-prs

**Find open PRs that haven't been updated in N days.**

```bash
ls-stale-prs 30                # PRs open for 30+ days
ls-stale-prs 14 ~/repos        # Check multiple repos
ls-stale-prs 7 -d              # Detailed YAML output
```

### Requirements

- GitHub CLI (`gh`) installed and authenticated
- Queries GitHub API for PR metadata

### Output (default)

```
org/repo:
  author1: (count, max_age_days)
  author2: (count, max_age_days)
```

---

## Common Patterns

### Find all your repos and check ownership
```bash
ls-git-repos ~/repos | xargs -I{} sh -c 'cd ~/repos/{} && ls-owners'
```

### Audit stale work across an org
```bash
ls-stale-branches 30 ~/repos/mycompany
ls-stale-prs 14 ~/repos/mycompany
```

### Clone all repos from an org
```bash
ls-github-repos mycompany | while read repo; do
  clone "$repo" --clonepath ~/repos
done
```

---

## Installation

All tools are in the `scottidler/git-tools` repo. Build with:

```bash
cd ~/repos/scottidler/git-tools
cargo build --release
# Binaries in target/release/
```

Or install individually:
```bash
cargo install --path clone
cargo install --path ls-git-repos
# etc.
```

