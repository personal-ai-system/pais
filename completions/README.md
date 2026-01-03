# Shell Completions

Pre-generated shell completions for `pais`.

## Installation

### Zsh

```bash
# Option 1: Copy to a directory in your fpath
mkdir -p ~/.zsh/completions
cp completions/_pais ~/.zsh/completions/

# Add to ~/.zshrc (if not already):
# fpath=(~/.zsh/completions $fpath)
# autoload -Uz compinit && compinit
```

### Bash

```bash
# Option 1: Copy to bash-completion directory
sudo cp completions/pais.bash /etc/bash_completion.d/pais

# Option 2: Source in ~/.bashrc
# source /path/to/pais/completions/pais.bash
```

### Fish

```bash
cp completions/pais.fish ~/.config/fish/completions/
```

## Regenerating

If you build from source or the CLI changes:

```bash
pais completions zsh > completions/_pais
pais completions bash > completions/pais.bash
pais completions fish > completions/pais.fish
```

