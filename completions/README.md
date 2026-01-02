# Shell Completions

Pre-generated shell completions for `paii`.

## Installation

### Zsh

```bash
# Option 1: Copy to a directory in your fpath
mkdir -p ~/.zsh/completions
cp completions/_paii ~/.zsh/completions/

# Add to ~/.zshrc (if not already):
# fpath=(~/.zsh/completions $fpath)
# autoload -Uz compinit && compinit
```

### Bash

```bash
# Option 1: Copy to bash-completion directory
sudo cp completions/paii.bash /etc/bash_completion.d/paii

# Option 2: Source in ~/.bashrc
# source /path/to/paii/completions/paii.bash
```

### Fish

```bash
cp completions/paii.fish ~/.config/fish/completions/
```

## Regenerating

If you build from source or the CLI changes:

```bash
paii completions zsh > completions/_paii
paii completions bash > completions/paii.bash
paii completions fish > completions/paii.fish
```

