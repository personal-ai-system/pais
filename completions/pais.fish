# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_pais_global_optspecs
	string join \n c/config= v/verbose q/quiet h/help V/version
end

function __fish_pais_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_pais_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_pais_using_subcommand
	set -l cmd (__fish_pais_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c pais -n "__fish_pais_needs_command" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_needs_command" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_needs_command" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_needs_command" -s h -l help -d 'Print help'
complete -c pais -n "__fish_pais_needs_command" -s V -l version -d 'Print version'
complete -c pais -n "__fish_pais_needs_command" -f -a "init" -d 'Initialize PAIS configuration'
complete -c pais -n "__fish_pais_needs_command" -f -a "doctor" -d 'Diagnose setup issues'
complete -c pais -n "__fish_pais_needs_command" -f -a "plugin" -d 'Manage plugins'
complete -c pais -n "__fish_pais_needs_command" -f -a "skill" -d 'Manage skills'
complete -c pais -n "__fish_pais_needs_command" -f -a "hook" -d 'Handle hook events from Claude Code'
complete -c pais -n "__fish_pais_needs_command" -f -a "history" -d 'Query and manage history'
complete -c pais -n "__fish_pais_needs_command" -f -a "config" -d 'Manage configuration'
complete -c pais -n "__fish_pais_needs_command" -f -a "context" -d 'Context injection for hooks'
complete -c pais -n "__fish_pais_needs_command" -f -a "registry" -d 'Manage plugin registries'
complete -c pais -n "__fish_pais_needs_command" -f -a "security" -d 'Security validation tools'
complete -c pais -n "__fish_pais_needs_command" -f -a "observe" -d 'Live event stream (tail events in real-time)'
complete -c pais -n "__fish_pais_needs_command" -f -a "agent" -d 'Manage agent personalities'
complete -c pais -n "__fish_pais_needs_command" -f -a "run" -d 'Run a plugin action directly'
complete -c pais -n "__fish_pais_needs_command" -f -a "status" -d 'Show system status'
complete -c pais -n "__fish_pais_needs_command" -f -a "sync" -d 'Sync skills to Claude Code (~/.claude/skills/)'
complete -c pais -n "__fish_pais_needs_command" -f -a "upgrade" -d 'Upgrade PAIS configuration (run migrations)'
complete -c pais -n "__fish_pais_needs_command" -f -a "completions" -d 'Generate shell completions'
complete -c pais -n "__fish_pais_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c pais -n "__fish_pais_using_subcommand init" -l path -d 'Directory to initialize (defaults to ~/.config/pais)' -r -F
complete -c pais -n "__fish_pais_using_subcommand init" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand init" -l force -d 'Overwrite existing configuration'
complete -c pais -n "__fish_pais_using_subcommand init" -l no-git -d 'Skip git repository initialization'
complete -c pais -n "__fish_pais_using_subcommand init" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand init" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand init" -s h -l help -d 'Print help'
complete -c pais -n "__fish_pais_using_subcommand doctor" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand doctor" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand doctor" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand doctor" -s h -l help -d 'Print help'
complete -c pais -n "__fish_pais_using_subcommand plugin; and not __fish_seen_subcommand_from list install remove update info new verify help" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand plugin; and not __fish_seen_subcommand_from list install remove update info new verify help" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand plugin; and not __fish_seen_subcommand_from list install remove update info new verify help" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand plugin; and not __fish_seen_subcommand_from list install remove update info new verify help" -s h -l help -d 'Print help'
complete -c pais -n "__fish_pais_using_subcommand plugin; and not __fish_seen_subcommand_from list install remove update info new verify help" -f -a "list" -d 'List installed plugins'
complete -c pais -n "__fish_pais_using_subcommand plugin; and not __fish_seen_subcommand_from list install remove update info new verify help" -f -a "install" -d 'Install a plugin'
complete -c pais -n "__fish_pais_using_subcommand plugin; and not __fish_seen_subcommand_from list install remove update info new verify help" -f -a "remove" -d 'Remove a plugin'
complete -c pais -n "__fish_pais_using_subcommand plugin; and not __fish_seen_subcommand_from list install remove update info new verify help" -f -a "update" -d 'Update a plugin'
complete -c pais -n "__fish_pais_using_subcommand plugin; and not __fish_seen_subcommand_from list install remove update info new verify help" -f -a "info" -d 'Show plugin details'
complete -c pais -n "__fish_pais_using_subcommand plugin; and not __fish_seen_subcommand_from list install remove update info new verify help" -f -a "new" -d 'Create a new plugin'
complete -c pais -n "__fish_pais_using_subcommand plugin; and not __fish_seen_subcommand_from list install remove update info new verify help" -f -a "verify" -d 'Verify plugin installation'
complete -c pais -n "__fish_pais_using_subcommand plugin; and not __fish_seen_subcommand_from list install remove update info new verify help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c pais -n "__fish_pais_using_subcommand plugin; and __fish_seen_subcommand_from list" -s o -l format -d 'Output format (default: text for TTY, json for pipes)' -r -f -a "text\t'Human-readable text'
json\t'JSON format'
yaml\t'YAML format'"
complete -c pais -n "__fish_pais_using_subcommand plugin; and __fish_seen_subcommand_from list" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand plugin; and __fish_seen_subcommand_from list" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand plugin; and __fish_seen_subcommand_from list" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand plugin; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c pais -n "__fish_pais_using_subcommand plugin; and __fish_seen_subcommand_from install" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand plugin; and __fish_seen_subcommand_from install" -l dev -d 'Symlink for development (don\'t copy)'
complete -c pais -n "__fish_pais_using_subcommand plugin; and __fish_seen_subcommand_from install" -l force -d 'Overwrite existing installation'
complete -c pais -n "__fish_pais_using_subcommand plugin; and __fish_seen_subcommand_from install" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand plugin; and __fish_seen_subcommand_from install" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand plugin; and __fish_seen_subcommand_from install" -s h -l help -d 'Print help'
complete -c pais -n "__fish_pais_using_subcommand plugin; and __fish_seen_subcommand_from remove" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand plugin; and __fish_seen_subcommand_from remove" -l force -d 'Remove even if other plugins depend on it'
complete -c pais -n "__fish_pais_using_subcommand plugin; and __fish_seen_subcommand_from remove" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand plugin; and __fish_seen_subcommand_from remove" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand plugin; and __fish_seen_subcommand_from remove" -s h -l help -d 'Print help'
complete -c pais -n "__fish_pais_using_subcommand plugin; and __fish_seen_subcommand_from update" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand plugin; and __fish_seen_subcommand_from update" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand plugin; and __fish_seen_subcommand_from update" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand plugin; and __fish_seen_subcommand_from update" -s h -l help -d 'Print help'
complete -c pais -n "__fish_pais_using_subcommand plugin; and __fish_seen_subcommand_from info" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand plugin; and __fish_seen_subcommand_from info" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand plugin; and __fish_seen_subcommand_from info" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand plugin; and __fish_seen_subcommand_from info" -s h -l help -d 'Print help'
complete -c pais -n "__fish_pais_using_subcommand plugin; and __fish_seen_subcommand_from new" -l language -d 'Language (python or rust)' -r
complete -c pais -n "__fish_pais_using_subcommand plugin; and __fish_seen_subcommand_from new" -l type -d 'Plugin type (foundation, integration, skill)' -r
complete -c pais -n "__fish_pais_using_subcommand plugin; and __fish_seen_subcommand_from new" -l path -d 'Output path' -r -F
complete -c pais -n "__fish_pais_using_subcommand plugin; and __fish_seen_subcommand_from new" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand plugin; and __fish_seen_subcommand_from new" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand plugin; and __fish_seen_subcommand_from new" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand plugin; and __fish_seen_subcommand_from new" -s h -l help -d 'Print help'
complete -c pais -n "__fish_pais_using_subcommand plugin; and __fish_seen_subcommand_from verify" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand plugin; and __fish_seen_subcommand_from verify" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand plugin; and __fish_seen_subcommand_from verify" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand plugin; and __fish_seen_subcommand_from verify" -s h -l help -d 'Print help'
complete -c pais -n "__fish_pais_using_subcommand plugin; and __fish_seen_subcommand_from help" -f -a "list" -d 'List installed plugins'
complete -c pais -n "__fish_pais_using_subcommand plugin; and __fish_seen_subcommand_from help" -f -a "install" -d 'Install a plugin'
complete -c pais -n "__fish_pais_using_subcommand plugin; and __fish_seen_subcommand_from help" -f -a "remove" -d 'Remove a plugin'
complete -c pais -n "__fish_pais_using_subcommand plugin; and __fish_seen_subcommand_from help" -f -a "update" -d 'Update a plugin'
complete -c pais -n "__fish_pais_using_subcommand plugin; and __fish_seen_subcommand_from help" -f -a "info" -d 'Show plugin details'
complete -c pais -n "__fish_pais_using_subcommand plugin; and __fish_seen_subcommand_from help" -f -a "new" -d 'Create a new plugin'
complete -c pais -n "__fish_pais_using_subcommand plugin; and __fish_seen_subcommand_from help" -f -a "verify" -d 'Verify plugin installation'
complete -c pais -n "__fish_pais_using_subcommand plugin; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c pais -n "__fish_pais_using_subcommand skill; and not __fish_seen_subcommand_from list add info edit remove validate scan index workflow help" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand skill; and not __fish_seen_subcommand_from list add info edit remove validate scan index workflow help" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand skill; and not __fish_seen_subcommand_from list add info edit remove validate scan index workflow help" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand skill; and not __fish_seen_subcommand_from list add info edit remove validate scan index workflow help" -s h -l help -d 'Print help'
complete -c pais -n "__fish_pais_using_subcommand skill; and not __fish_seen_subcommand_from list add info edit remove validate scan index workflow help" -f -a "list" -d 'List all skills (simple and plugin-based)'
complete -c pais -n "__fish_pais_using_subcommand skill; and not __fish_seen_subcommand_from list add info edit remove validate scan index workflow help" -f -a "add" -d 'Create a new skill from template'
complete -c pais -n "__fish_pais_using_subcommand skill; and not __fish_seen_subcommand_from list add info edit remove validate scan index workflow help" -f -a "info" -d 'Show skill details'
complete -c pais -n "__fish_pais_using_subcommand skill; and not __fish_seen_subcommand_from list add info edit remove validate scan index workflow help" -f -a "edit" -d 'Edit a skill in $EDITOR'
complete -c pais -n "__fish_pais_using_subcommand skill; and not __fish_seen_subcommand_from list add info edit remove validate scan index workflow help" -f -a "remove" -d 'Remove a skill'
complete -c pais -n "__fish_pais_using_subcommand skill; and not __fish_seen_subcommand_from list add info edit remove validate scan index workflow help" -f -a "validate" -d 'Validate SKILL.md format'
complete -c pais -n "__fish_pais_using_subcommand skill; and not __fish_seen_subcommand_from list add info edit remove validate scan index workflow help" -f -a "scan" -d 'Scan directories for .pais/SKILL.md files'
complete -c pais -n "__fish_pais_using_subcommand skill; and not __fish_seen_subcommand_from list add info edit remove validate scan index workflow help" -f -a "index" -d 'Generate skill index for context injection'
complete -c pais -n "__fish_pais_using_subcommand skill; and not __fish_seen_subcommand_from list add info edit remove validate scan index workflow help" -f -a "workflow" -d 'Show or list workflows for a skill'
complete -c pais -n "__fish_pais_using_subcommand skill; and not __fish_seen_subcommand_from list add info edit remove validate scan index workflow help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from list" -s o -l format -d 'Output format (default: text for TTY, json for pipes)' -r -f -a "text\t'Human-readable text'
json\t'JSON format'
yaml\t'YAML format'"
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from list" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from list" -l simple -d 'Show only simple skills (no plugin.yaml)'
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from list" -l plugin -d 'Show only plugin skills'
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from list" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from list" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from add" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from add" -s e -l edit -d 'Open in $EDITOR after creation'
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from add" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from add" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from add" -s h -l help -d 'Print help'
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from info" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from info" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from info" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from info" -s h -l help -d 'Print help'
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from edit" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from edit" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from edit" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from edit" -s h -l help -d 'Print help'
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from remove" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from remove" -l force -d 'Remove without confirmation'
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from remove" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from remove" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from remove" -s h -l help -d 'Print help'
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from validate" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from validate" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from validate" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from validate" -s h -l help -d 'Print help'
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from scan" -l depth -d 'Maximum depth to scan (default: 4)' -r
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from scan" -s o -l format -d 'Output format (default: text for TTY, json for pipes)' -r -f -a "text\t'Human-readable text'
json\t'JSON format'
yaml\t'YAML format'"
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from scan" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from scan" -l register -d 'Register found skills (create symlinks in ~/.config/pais/skills/)'
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from scan" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from scan" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from scan" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from index" -s o -l format -d 'Output format (default: text for TTY, json for pipes)' -r -f -a "text\t'Human-readable text'
json\t'JSON format'
yaml\t'YAML format'"
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from index" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from index" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from index" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from index" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from workflow" -s o -l format -d 'Output format (default: text for TTY, json for pipes)' -r -f -a "text\t'Human-readable text'
json\t'JSON format'
yaml\t'YAML format'"
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from workflow" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from workflow" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from workflow" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from workflow" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from help" -f -a "list" -d 'List all skills (simple and plugin-based)'
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from help" -f -a "add" -d 'Create a new skill from template'
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from help" -f -a "info" -d 'Show skill details'
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from help" -f -a "edit" -d 'Edit a skill in $EDITOR'
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from help" -f -a "remove" -d 'Remove a skill'
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from help" -f -a "validate" -d 'Validate SKILL.md format'
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from help" -f -a "scan" -d 'Scan directories for .pais/SKILL.md files'
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from help" -f -a "index" -d 'Generate skill index for context injection'
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from help" -f -a "workflow" -d 'Show or list workflows for a skill'
complete -c pais -n "__fish_pais_using_subcommand skill; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c pais -n "__fish_pais_using_subcommand hook; and not __fish_seen_subcommand_from dispatch list help" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand hook; and not __fish_seen_subcommand_from dispatch list help" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand hook; and not __fish_seen_subcommand_from dispatch list help" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand hook; and not __fish_seen_subcommand_from dispatch list help" -s h -l help -d 'Print help'
complete -c pais -n "__fish_pais_using_subcommand hook; and not __fish_seen_subcommand_from dispatch list help" -f -a "dispatch" -d 'Dispatch a hook event to handlers'
complete -c pais -n "__fish_pais_using_subcommand hook; and not __fish_seen_subcommand_from dispatch list help" -f -a "list" -d 'List registered hook handlers'
complete -c pais -n "__fish_pais_using_subcommand hook; and not __fish_seen_subcommand_from dispatch list help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c pais -n "__fish_pais_using_subcommand hook; and __fish_seen_subcommand_from dispatch" -l payload -d 'Event payload JSON (reads from stdin if not provided)' -r
complete -c pais -n "__fish_pais_using_subcommand hook; and __fish_seen_subcommand_from dispatch" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand hook; and __fish_seen_subcommand_from dispatch" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand hook; and __fish_seen_subcommand_from dispatch" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand hook; and __fish_seen_subcommand_from dispatch" -s h -l help -d 'Print help'
complete -c pais -n "__fish_pais_using_subcommand hook; and __fish_seen_subcommand_from list" -l event -d 'Filter by event type' -r
complete -c pais -n "__fish_pais_using_subcommand hook; and __fish_seen_subcommand_from list" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand hook; and __fish_seen_subcommand_from list" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand hook; and __fish_seen_subcommand_from list" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand hook; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c pais -n "__fish_pais_using_subcommand hook; and __fish_seen_subcommand_from help" -f -a "dispatch" -d 'Dispatch a hook event to handlers'
complete -c pais -n "__fish_pais_using_subcommand hook; and __fish_seen_subcommand_from help" -f -a "list" -d 'List registered hook handlers'
complete -c pais -n "__fish_pais_using_subcommand hook; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c pais -n "__fish_pais_using_subcommand history; and not __fish_seen_subcommand_from query recent categories show stats events help" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand history; and not __fish_seen_subcommand_from query recent categories show stats events help" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand history; and not __fish_seen_subcommand_from query recent categories show stats events help" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand history; and not __fish_seen_subcommand_from query recent categories show stats events help" -s h -l help -d 'Print help'
complete -c pais -n "__fish_pais_using_subcommand history; and not __fish_seen_subcommand_from query recent categories show stats events help" -f -a "query" -d 'Search history'
complete -c pais -n "__fish_pais_using_subcommand history; and not __fish_seen_subcommand_from query recent categories show stats events help" -f -a "recent" -d 'Show recent entries'
complete -c pais -n "__fish_pais_using_subcommand history; and not __fish_seen_subcommand_from query recent categories show stats events help" -f -a "categories" -d 'List available categories'
complete -c pais -n "__fish_pais_using_subcommand history; and not __fish_seen_subcommand_from query recent categories show stats events help" -f -a "show" -d 'Show a specific history entry'
complete -c pais -n "__fish_pais_using_subcommand history; and not __fish_seen_subcommand_from query recent categories show stats events help" -f -a "stats" -d 'Show event statistics'
complete -c pais -n "__fish_pais_using_subcommand history; and not __fish_seen_subcommand_from query recent categories show stats events help" -f -a "events" -d 'List raw event dates'
complete -c pais -n "__fish_pais_using_subcommand history; and not __fish_seen_subcommand_from query recent categories show stats events help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c pais -n "__fish_pais_using_subcommand history; and __fish_seen_subcommand_from query" -l category -d 'Category to search' -r
complete -c pais -n "__fish_pais_using_subcommand history; and __fish_seen_subcommand_from query" -l limit -d 'Max results' -r
complete -c pais -n "__fish_pais_using_subcommand history; and __fish_seen_subcommand_from query" -l since -d 'Only entries after this date' -r
complete -c pais -n "__fish_pais_using_subcommand history; and __fish_seen_subcommand_from query" -s o -l format -d 'Output format (default: text for TTY, json for pipes)' -r -f -a "text\t'Human-readable text'
json\t'JSON format'
yaml\t'YAML format'"
complete -c pais -n "__fish_pais_using_subcommand history; and __fish_seen_subcommand_from query" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand history; and __fish_seen_subcommand_from query" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand history; and __fish_seen_subcommand_from query" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand history; and __fish_seen_subcommand_from query" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c pais -n "__fish_pais_using_subcommand history; and __fish_seen_subcommand_from recent" -l category -d 'Category' -r
complete -c pais -n "__fish_pais_using_subcommand history; and __fish_seen_subcommand_from recent" -l count -d 'Number of entries' -r
complete -c pais -n "__fish_pais_using_subcommand history; and __fish_seen_subcommand_from recent" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand history; and __fish_seen_subcommand_from recent" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand history; and __fish_seen_subcommand_from recent" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand history; and __fish_seen_subcommand_from recent" -s h -l help -d 'Print help'
complete -c pais -n "__fish_pais_using_subcommand history; and __fish_seen_subcommand_from categories" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand history; and __fish_seen_subcommand_from categories" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand history; and __fish_seen_subcommand_from categories" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand history; and __fish_seen_subcommand_from categories" -s h -l help -d 'Print help'
complete -c pais -n "__fish_pais_using_subcommand history; and __fish_seen_subcommand_from show" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand history; and __fish_seen_subcommand_from show" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand history; and __fish_seen_subcommand_from show" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand history; and __fish_seen_subcommand_from show" -s h -l help -d 'Print help'
complete -c pais -n "__fish_pais_using_subcommand history; and __fish_seen_subcommand_from stats" -l days -d 'Number of days to include' -r
complete -c pais -n "__fish_pais_using_subcommand history; and __fish_seen_subcommand_from stats" -s o -l format -d 'Output format (default: text for TTY, json for pipes)' -r -f -a "text\t'Human-readable text'
json\t'JSON format'
yaml\t'YAML format'"
complete -c pais -n "__fish_pais_using_subcommand history; and __fish_seen_subcommand_from stats" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand history; and __fish_seen_subcommand_from stats" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand history; and __fish_seen_subcommand_from stats" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand history; and __fish_seen_subcommand_from stats" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c pais -n "__fish_pais_using_subcommand history; and __fish_seen_subcommand_from events" -l limit -d 'Number of recent dates to show' -r
complete -c pais -n "__fish_pais_using_subcommand history; and __fish_seen_subcommand_from events" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand history; and __fish_seen_subcommand_from events" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand history; and __fish_seen_subcommand_from events" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand history; and __fish_seen_subcommand_from events" -s h -l help -d 'Print help'
complete -c pais -n "__fish_pais_using_subcommand history; and __fish_seen_subcommand_from help" -f -a "query" -d 'Search history'
complete -c pais -n "__fish_pais_using_subcommand history; and __fish_seen_subcommand_from help" -f -a "recent" -d 'Show recent entries'
complete -c pais -n "__fish_pais_using_subcommand history; and __fish_seen_subcommand_from help" -f -a "categories" -d 'List available categories'
complete -c pais -n "__fish_pais_using_subcommand history; and __fish_seen_subcommand_from help" -f -a "show" -d 'Show a specific history entry'
complete -c pais -n "__fish_pais_using_subcommand history; and __fish_seen_subcommand_from help" -f -a "stats" -d 'Show event statistics'
complete -c pais -n "__fish_pais_using_subcommand history; and __fish_seen_subcommand_from help" -f -a "events" -d 'List raw event dates'
complete -c pais -n "__fish_pais_using_subcommand history; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c pais -n "__fish_pais_using_subcommand config; and not __fish_seen_subcommand_from show get set help" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand config; and not __fish_seen_subcommand_from show get set help" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand config; and not __fish_seen_subcommand_from show get set help" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand config; and not __fish_seen_subcommand_from show get set help" -s h -l help -d 'Print help'
complete -c pais -n "__fish_pais_using_subcommand config; and not __fish_seen_subcommand_from show get set help" -f -a "show" -d 'Show current configuration'
complete -c pais -n "__fish_pais_using_subcommand config; and not __fish_seen_subcommand_from show get set help" -f -a "get" -d 'Get a configuration value'
complete -c pais -n "__fish_pais_using_subcommand config; and not __fish_seen_subcommand_from show get set help" -f -a "set" -d 'Set a configuration value'
complete -c pais -n "__fish_pais_using_subcommand config; and not __fish_seen_subcommand_from show get set help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c pais -n "__fish_pais_using_subcommand config; and __fish_seen_subcommand_from show" -s o -l format -d 'Output format (default: text for TTY, json for pipes)' -r -f -a "text\t'Human-readable text'
json\t'JSON format'
yaml\t'YAML format'"
complete -c pais -n "__fish_pais_using_subcommand config; and __fish_seen_subcommand_from show" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand config; and __fish_seen_subcommand_from show" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand config; and __fish_seen_subcommand_from show" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand config; and __fish_seen_subcommand_from show" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c pais -n "__fish_pais_using_subcommand config; and __fish_seen_subcommand_from get" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand config; and __fish_seen_subcommand_from get" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand config; and __fish_seen_subcommand_from get" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand config; and __fish_seen_subcommand_from get" -s h -l help -d 'Print help'
complete -c pais -n "__fish_pais_using_subcommand config; and __fish_seen_subcommand_from set" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand config; and __fish_seen_subcommand_from set" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand config; and __fish_seen_subcommand_from set" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand config; and __fish_seen_subcommand_from set" -s h -l help -d 'Print help'
complete -c pais -n "__fish_pais_using_subcommand config; and __fish_seen_subcommand_from help" -f -a "show" -d 'Show current configuration'
complete -c pais -n "__fish_pais_using_subcommand config; and __fish_seen_subcommand_from help" -f -a "get" -d 'Get a configuration value'
complete -c pais -n "__fish_pais_using_subcommand config; and __fish_seen_subcommand_from help" -f -a "set" -d 'Set a configuration value'
complete -c pais -n "__fish_pais_using_subcommand config; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c pais -n "__fish_pais_using_subcommand context; and not __fish_seen_subcommand_from inject help" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand context; and not __fish_seen_subcommand_from inject help" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand context; and not __fish_seen_subcommand_from inject help" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand context; and not __fish_seen_subcommand_from inject help" -s h -l help -d 'Print help'
complete -c pais -n "__fish_pais_using_subcommand context; and not __fish_seen_subcommand_from inject help" -f -a "inject" -d 'Inject skill context for SessionStart hook'
complete -c pais -n "__fish_pais_using_subcommand context; and not __fish_seen_subcommand_from inject help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c pais -n "__fish_pais_using_subcommand context; and __fish_seen_subcommand_from inject" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand context; and __fish_seen_subcommand_from inject" -l raw -d 'Output raw content without system-reminder wrapper'
complete -c pais -n "__fish_pais_using_subcommand context; and __fish_seen_subcommand_from inject" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand context; and __fish_seen_subcommand_from inject" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand context; and __fish_seen_subcommand_from inject" -s h -l help -d 'Print help'
complete -c pais -n "__fish_pais_using_subcommand context; and __fish_seen_subcommand_from help" -f -a "inject" -d 'Inject skill context for SessionStart hook'
complete -c pais -n "__fish_pais_using_subcommand context; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c pais -n "__fish_pais_using_subcommand registry; and not __fish_seen_subcommand_from list add remove update search show help" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand registry; and not __fish_seen_subcommand_from list add remove update search show help" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand registry; and not __fish_seen_subcommand_from list add remove update search show help" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand registry; and not __fish_seen_subcommand_from list add remove update search show help" -s h -l help -d 'Print help'
complete -c pais -n "__fish_pais_using_subcommand registry; and not __fish_seen_subcommand_from list add remove update search show help" -f -a "list" -d 'List configured registries'
complete -c pais -n "__fish_pais_using_subcommand registry; and not __fish_seen_subcommand_from list add remove update search show help" -f -a "add" -d 'Add a registry'
complete -c pais -n "__fish_pais_using_subcommand registry; and not __fish_seen_subcommand_from list add remove update search show help" -f -a "remove" -d 'Remove a registry'
complete -c pais -n "__fish_pais_using_subcommand registry; and not __fish_seen_subcommand_from list add remove update search show help" -f -a "update" -d 'Update registry listings'
complete -c pais -n "__fish_pais_using_subcommand registry; and not __fish_seen_subcommand_from list add remove update search show help" -f -a "search" -d 'Search for plugins in cached registries'
complete -c pais -n "__fish_pais_using_subcommand registry; and not __fish_seen_subcommand_from list add remove update search show help" -f -a "show" -d 'Show all plugins in a cached registry'
complete -c pais -n "__fish_pais_using_subcommand registry; and not __fish_seen_subcommand_from list add remove update search show help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c pais -n "__fish_pais_using_subcommand registry; and __fish_seen_subcommand_from list" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand registry; and __fish_seen_subcommand_from list" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand registry; and __fish_seen_subcommand_from list" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand registry; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c pais -n "__fish_pais_using_subcommand registry; and __fish_seen_subcommand_from add" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand registry; and __fish_seen_subcommand_from add" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand registry; and __fish_seen_subcommand_from add" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand registry; and __fish_seen_subcommand_from add" -s h -l help -d 'Print help'
complete -c pais -n "__fish_pais_using_subcommand registry; and __fish_seen_subcommand_from remove" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand registry; and __fish_seen_subcommand_from remove" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand registry; and __fish_seen_subcommand_from remove" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand registry; and __fish_seen_subcommand_from remove" -s h -l help -d 'Print help'
complete -c pais -n "__fish_pais_using_subcommand registry; and __fish_seen_subcommand_from update" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand registry; and __fish_seen_subcommand_from update" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand registry; and __fish_seen_subcommand_from update" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand registry; and __fish_seen_subcommand_from update" -s h -l help -d 'Print help'
complete -c pais -n "__fish_pais_using_subcommand registry; and __fish_seen_subcommand_from search" -s o -l format -d 'Output format (default: text for TTY, json for pipes)' -r -f -a "text\t'Human-readable text'
json\t'JSON format'
yaml\t'YAML format'"
complete -c pais -n "__fish_pais_using_subcommand registry; and __fish_seen_subcommand_from search" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand registry; and __fish_seen_subcommand_from search" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand registry; and __fish_seen_subcommand_from search" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand registry; and __fish_seen_subcommand_from search" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c pais -n "__fish_pais_using_subcommand registry; and __fish_seen_subcommand_from show" -s o -l format -d 'Output format (default: text for TTY, json for pipes)' -r -f -a "text\t'Human-readable text'
json\t'JSON format'
yaml\t'YAML format'"
complete -c pais -n "__fish_pais_using_subcommand registry; and __fish_seen_subcommand_from show" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand registry; and __fish_seen_subcommand_from show" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand registry; and __fish_seen_subcommand_from show" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand registry; and __fish_seen_subcommand_from show" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c pais -n "__fish_pais_using_subcommand registry; and __fish_seen_subcommand_from help" -f -a "list" -d 'List configured registries'
complete -c pais -n "__fish_pais_using_subcommand registry; and __fish_seen_subcommand_from help" -f -a "add" -d 'Add a registry'
complete -c pais -n "__fish_pais_using_subcommand registry; and __fish_seen_subcommand_from help" -f -a "remove" -d 'Remove a registry'
complete -c pais -n "__fish_pais_using_subcommand registry; and __fish_seen_subcommand_from help" -f -a "update" -d 'Update registry listings'
complete -c pais -n "__fish_pais_using_subcommand registry; and __fish_seen_subcommand_from help" -f -a "search" -d 'Search for plugins in cached registries'
complete -c pais -n "__fish_pais_using_subcommand registry; and __fish_seen_subcommand_from help" -f -a "show" -d 'Show all plugins in a cached registry'
complete -c pais -n "__fish_pais_using_subcommand registry; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c pais -n "__fish_pais_using_subcommand security; and not __fish_seen_subcommand_from tiers log test help" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand security; and not __fish_seen_subcommand_from tiers log test help" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand security; and not __fish_seen_subcommand_from tiers log test help" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand security; and not __fish_seen_subcommand_from tiers log test help" -s h -l help -d 'Print help'
complete -c pais -n "__fish_pais_using_subcommand security; and not __fish_seen_subcommand_from tiers log test help" -f -a "tiers" -d 'Show security tiers and their actions'
complete -c pais -n "__fish_pais_using_subcommand security; and not __fish_seen_subcommand_from tiers log test help" -f -a "log" -d 'View security event log'
complete -c pais -n "__fish_pais_using_subcommand security; and not __fish_seen_subcommand_from tiers log test help" -f -a "test" -d 'Test a command against security patterns'
complete -c pais -n "__fish_pais_using_subcommand security; and not __fish_seen_subcommand_from tiers log test help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c pais -n "__fish_pais_using_subcommand security; and __fish_seen_subcommand_from tiers" -s o -l format -d 'Output format (default: text for TTY, json for pipes)' -r -f -a "text\t'Human-readable text'
json\t'JSON format'
yaml\t'YAML format'"
complete -c pais -n "__fish_pais_using_subcommand security; and __fish_seen_subcommand_from tiers" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand security; and __fish_seen_subcommand_from tiers" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand security; and __fish_seen_subcommand_from tiers" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand security; and __fish_seen_subcommand_from tiers" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c pais -n "__fish_pais_using_subcommand security; and __fish_seen_subcommand_from log" -l days -d 'Number of days to show' -r
complete -c pais -n "__fish_pais_using_subcommand security; and __fish_seen_subcommand_from log" -s o -l format -d 'Output format (default: text for TTY, json for pipes)' -r -f -a "text\t'Human-readable text'
json\t'JSON format'
yaml\t'YAML format'"
complete -c pais -n "__fish_pais_using_subcommand security; and __fish_seen_subcommand_from log" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand security; and __fish_seen_subcommand_from log" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand security; and __fish_seen_subcommand_from log" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand security; and __fish_seen_subcommand_from log" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c pais -n "__fish_pais_using_subcommand security; and __fish_seen_subcommand_from test" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand security; and __fish_seen_subcommand_from test" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand security; and __fish_seen_subcommand_from test" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand security; and __fish_seen_subcommand_from test" -s h -l help -d 'Print help'
complete -c pais -n "__fish_pais_using_subcommand security; and __fish_seen_subcommand_from help" -f -a "tiers" -d 'Show security tiers and their actions'
complete -c pais -n "__fish_pais_using_subcommand security; and __fish_seen_subcommand_from help" -f -a "log" -d 'View security event log'
complete -c pais -n "__fish_pais_using_subcommand security; and __fish_seen_subcommand_from help" -f -a "test" -d 'Test a command against security patterns'
complete -c pais -n "__fish_pais_using_subcommand security; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c pais -n "__fish_pais_using_subcommand observe" -s f -l filter -d 'Filter by event type (e.g., PreToolUse, SessionStart)' -r
complete -c pais -n "__fish_pais_using_subcommand observe" -s n -l last -d 'Number of recent events to show before tailing' -r
complete -c pais -n "__fish_pais_using_subcommand observe" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand observe" -l payload -d 'Include full payload in output'
complete -c pais -n "__fish_pais_using_subcommand observe" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand observe" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand observe" -s h -l help -d 'Print help'
complete -c pais -n "__fish_pais_using_subcommand agent; and not __fish_seen_subcommand_from list show traits prompt create help" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand agent; and not __fish_seen_subcommand_from list show traits prompt create help" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand agent; and not __fish_seen_subcommand_from list show traits prompt create help" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand agent; and not __fish_seen_subcommand_from list show traits prompt create help" -s h -l help -d 'Print help'
complete -c pais -n "__fish_pais_using_subcommand agent; and not __fish_seen_subcommand_from list show traits prompt create help" -f -a "list" -d 'List available agents'
complete -c pais -n "__fish_pais_using_subcommand agent; and not __fish_seen_subcommand_from list show traits prompt create help" -f -a "show" -d 'Show agent details'
complete -c pais -n "__fish_pais_using_subcommand agent; and not __fish_seen_subcommand_from list show traits prompt create help" -f -a "traits" -d 'List available traits'
complete -c pais -n "__fish_pais_using_subcommand agent; and not __fish_seen_subcommand_from list show traits prompt create help" -f -a "prompt" -d 'Generate prompt for an agent'
complete -c pais -n "__fish_pais_using_subcommand agent; and not __fish_seen_subcommand_from list show traits prompt create help" -f -a "create" -d 'Create a new agent from template'
complete -c pais -n "__fish_pais_using_subcommand agent; and not __fish_seen_subcommand_from list show traits prompt create help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c pais -n "__fish_pais_using_subcommand agent; and __fish_seen_subcommand_from list" -s o -l format -d 'Output format (default: text for TTY, json for pipes)' -r -f -a "text\t'Human-readable text'
json\t'JSON format'
yaml\t'YAML format'"
complete -c pais -n "__fish_pais_using_subcommand agent; and __fish_seen_subcommand_from list" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand agent; and __fish_seen_subcommand_from list" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand agent; and __fish_seen_subcommand_from list" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand agent; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c pais -n "__fish_pais_using_subcommand agent; and __fish_seen_subcommand_from show" -s o -l format -d 'Output format (default: text for TTY, json for pipes)' -r -f -a "text\t'Human-readable text'
json\t'JSON format'
yaml\t'YAML format'"
complete -c pais -n "__fish_pais_using_subcommand agent; and __fish_seen_subcommand_from show" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand agent; and __fish_seen_subcommand_from show" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand agent; and __fish_seen_subcommand_from show" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand agent; and __fish_seen_subcommand_from show" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c pais -n "__fish_pais_using_subcommand agent; and __fish_seen_subcommand_from traits" -s o -l format -d 'Output format (default: text for TTY, json for pipes)' -r -f -a "text\t'Human-readable text'
json\t'JSON format'
yaml\t'YAML format'"
complete -c pais -n "__fish_pais_using_subcommand agent; and __fish_seen_subcommand_from traits" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand agent; and __fish_seen_subcommand_from traits" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand agent; and __fish_seen_subcommand_from traits" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand agent; and __fish_seen_subcommand_from traits" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c pais -n "__fish_pais_using_subcommand agent; and __fish_seen_subcommand_from prompt" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand agent; and __fish_seen_subcommand_from prompt" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand agent; and __fish_seen_subcommand_from prompt" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand agent; and __fish_seen_subcommand_from prompt" -s h -l help -d 'Print help'
complete -c pais -n "__fish_pais_using_subcommand agent; and __fish_seen_subcommand_from create" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand agent; and __fish_seen_subcommand_from create" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand agent; and __fish_seen_subcommand_from create" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand agent; and __fish_seen_subcommand_from create" -s h -l help -d 'Print help'
complete -c pais -n "__fish_pais_using_subcommand agent; and __fish_seen_subcommand_from help" -f -a "list" -d 'List available agents'
complete -c pais -n "__fish_pais_using_subcommand agent; and __fish_seen_subcommand_from help" -f -a "show" -d 'Show agent details'
complete -c pais -n "__fish_pais_using_subcommand agent; and __fish_seen_subcommand_from help" -f -a "traits" -d 'List available traits'
complete -c pais -n "__fish_pais_using_subcommand agent; and __fish_seen_subcommand_from help" -f -a "prompt" -d 'Generate prompt for an agent'
complete -c pais -n "__fish_pais_using_subcommand agent; and __fish_seen_subcommand_from help" -f -a "create" -d 'Create a new agent from template'
complete -c pais -n "__fish_pais_using_subcommand agent; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c pais -n "__fish_pais_using_subcommand run" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand run" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand run" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand run" -s h -l help -d 'Print help'
complete -c pais -n "__fish_pais_using_subcommand status" -s o -l format -d 'Output format (default: text for TTY, json for pipes)' -r -f -a "text\t'Human-readable text'
json\t'JSON format'
yaml\t'YAML format'"
complete -c pais -n "__fish_pais_using_subcommand status" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand status" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand status" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand status" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c pais -n "__fish_pais_using_subcommand sync" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand sync" -l dry-run -d 'Show what would happen without making changes'
complete -c pais -n "__fish_pais_using_subcommand sync" -l clean -d 'Remove orphaned symlinks from Claude skills directory'
complete -c pais -n "__fish_pais_using_subcommand sync" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand sync" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand sync" -s h -l help -d 'Print help'
complete -c pais -n "__fish_pais_using_subcommand upgrade" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand upgrade" -l dry-run -d 'Show what would happen without making changes'
complete -c pais -n "__fish_pais_using_subcommand upgrade" -l status -d 'Show current version info only'
complete -c pais -n "__fish_pais_using_subcommand upgrade" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand upgrade" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand upgrade" -s h -l help -d 'Print help'
complete -c pais -n "__fish_pais_using_subcommand completions" -s c -l config -d 'Path to pais.yaml config file' -r -F
complete -c pais -n "__fish_pais_using_subcommand completions" -s v -l verbose -d 'Enable verbose output'
complete -c pais -n "__fish_pais_using_subcommand completions" -s q -l quiet -d 'Suppress non-error output'
complete -c pais -n "__fish_pais_using_subcommand completions" -s h -l help -d 'Print help'
complete -c pais -n "__fish_pais_using_subcommand help; and not __fish_seen_subcommand_from init doctor plugin skill hook history config context registry security observe agent run status sync upgrade completions help" -f -a "init" -d 'Initialize PAIS configuration'
complete -c pais -n "__fish_pais_using_subcommand help; and not __fish_seen_subcommand_from init doctor plugin skill hook history config context registry security observe agent run status sync upgrade completions help" -f -a "doctor" -d 'Diagnose setup issues'
complete -c pais -n "__fish_pais_using_subcommand help; and not __fish_seen_subcommand_from init doctor plugin skill hook history config context registry security observe agent run status sync upgrade completions help" -f -a "plugin" -d 'Manage plugins'
complete -c pais -n "__fish_pais_using_subcommand help; and not __fish_seen_subcommand_from init doctor plugin skill hook history config context registry security observe agent run status sync upgrade completions help" -f -a "skill" -d 'Manage skills'
complete -c pais -n "__fish_pais_using_subcommand help; and not __fish_seen_subcommand_from init doctor plugin skill hook history config context registry security observe agent run status sync upgrade completions help" -f -a "hook" -d 'Handle hook events from Claude Code'
complete -c pais -n "__fish_pais_using_subcommand help; and not __fish_seen_subcommand_from init doctor plugin skill hook history config context registry security observe agent run status sync upgrade completions help" -f -a "history" -d 'Query and manage history'
complete -c pais -n "__fish_pais_using_subcommand help; and not __fish_seen_subcommand_from init doctor plugin skill hook history config context registry security observe agent run status sync upgrade completions help" -f -a "config" -d 'Manage configuration'
complete -c pais -n "__fish_pais_using_subcommand help; and not __fish_seen_subcommand_from init doctor plugin skill hook history config context registry security observe agent run status sync upgrade completions help" -f -a "context" -d 'Context injection for hooks'
complete -c pais -n "__fish_pais_using_subcommand help; and not __fish_seen_subcommand_from init doctor plugin skill hook history config context registry security observe agent run status sync upgrade completions help" -f -a "registry" -d 'Manage plugin registries'
complete -c pais -n "__fish_pais_using_subcommand help; and not __fish_seen_subcommand_from init doctor plugin skill hook history config context registry security observe agent run status sync upgrade completions help" -f -a "security" -d 'Security validation tools'
complete -c pais -n "__fish_pais_using_subcommand help; and not __fish_seen_subcommand_from init doctor plugin skill hook history config context registry security observe agent run status sync upgrade completions help" -f -a "observe" -d 'Live event stream (tail events in real-time)'
complete -c pais -n "__fish_pais_using_subcommand help; and not __fish_seen_subcommand_from init doctor plugin skill hook history config context registry security observe agent run status sync upgrade completions help" -f -a "agent" -d 'Manage agent personalities'
complete -c pais -n "__fish_pais_using_subcommand help; and not __fish_seen_subcommand_from init doctor plugin skill hook history config context registry security observe agent run status sync upgrade completions help" -f -a "run" -d 'Run a plugin action directly'
complete -c pais -n "__fish_pais_using_subcommand help; and not __fish_seen_subcommand_from init doctor plugin skill hook history config context registry security observe agent run status sync upgrade completions help" -f -a "status" -d 'Show system status'
complete -c pais -n "__fish_pais_using_subcommand help; and not __fish_seen_subcommand_from init doctor plugin skill hook history config context registry security observe agent run status sync upgrade completions help" -f -a "sync" -d 'Sync skills to Claude Code (~/.claude/skills/)'
complete -c pais -n "__fish_pais_using_subcommand help; and not __fish_seen_subcommand_from init doctor plugin skill hook history config context registry security observe agent run status sync upgrade completions help" -f -a "upgrade" -d 'Upgrade PAIS configuration (run migrations)'
complete -c pais -n "__fish_pais_using_subcommand help; and not __fish_seen_subcommand_from init doctor plugin skill hook history config context registry security observe agent run status sync upgrade completions help" -f -a "completions" -d 'Generate shell completions'
complete -c pais -n "__fish_pais_using_subcommand help; and not __fish_seen_subcommand_from init doctor plugin skill hook history config context registry security observe agent run status sync upgrade completions help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c pais -n "__fish_pais_using_subcommand help; and __fish_seen_subcommand_from plugin" -f -a "list" -d 'List installed plugins'
complete -c pais -n "__fish_pais_using_subcommand help; and __fish_seen_subcommand_from plugin" -f -a "install" -d 'Install a plugin'
complete -c pais -n "__fish_pais_using_subcommand help; and __fish_seen_subcommand_from plugin" -f -a "remove" -d 'Remove a plugin'
complete -c pais -n "__fish_pais_using_subcommand help; and __fish_seen_subcommand_from plugin" -f -a "update" -d 'Update a plugin'
complete -c pais -n "__fish_pais_using_subcommand help; and __fish_seen_subcommand_from plugin" -f -a "info" -d 'Show plugin details'
complete -c pais -n "__fish_pais_using_subcommand help; and __fish_seen_subcommand_from plugin" -f -a "new" -d 'Create a new plugin'
complete -c pais -n "__fish_pais_using_subcommand help; and __fish_seen_subcommand_from plugin" -f -a "verify" -d 'Verify plugin installation'
complete -c pais -n "__fish_pais_using_subcommand help; and __fish_seen_subcommand_from skill" -f -a "list" -d 'List all skills (simple and plugin-based)'
complete -c pais -n "__fish_pais_using_subcommand help; and __fish_seen_subcommand_from skill" -f -a "add" -d 'Create a new skill from template'
complete -c pais -n "__fish_pais_using_subcommand help; and __fish_seen_subcommand_from skill" -f -a "info" -d 'Show skill details'
complete -c pais -n "__fish_pais_using_subcommand help; and __fish_seen_subcommand_from skill" -f -a "edit" -d 'Edit a skill in $EDITOR'
complete -c pais -n "__fish_pais_using_subcommand help; and __fish_seen_subcommand_from skill" -f -a "remove" -d 'Remove a skill'
complete -c pais -n "__fish_pais_using_subcommand help; and __fish_seen_subcommand_from skill" -f -a "validate" -d 'Validate SKILL.md format'
complete -c pais -n "__fish_pais_using_subcommand help; and __fish_seen_subcommand_from skill" -f -a "scan" -d 'Scan directories for .pais/SKILL.md files'
complete -c pais -n "__fish_pais_using_subcommand help; and __fish_seen_subcommand_from skill" -f -a "index" -d 'Generate skill index for context injection'
complete -c pais -n "__fish_pais_using_subcommand help; and __fish_seen_subcommand_from skill" -f -a "workflow" -d 'Show or list workflows for a skill'
complete -c pais -n "__fish_pais_using_subcommand help; and __fish_seen_subcommand_from hook" -f -a "dispatch" -d 'Dispatch a hook event to handlers'
complete -c pais -n "__fish_pais_using_subcommand help; and __fish_seen_subcommand_from hook" -f -a "list" -d 'List registered hook handlers'
complete -c pais -n "__fish_pais_using_subcommand help; and __fish_seen_subcommand_from history" -f -a "query" -d 'Search history'
complete -c pais -n "__fish_pais_using_subcommand help; and __fish_seen_subcommand_from history" -f -a "recent" -d 'Show recent entries'
complete -c pais -n "__fish_pais_using_subcommand help; and __fish_seen_subcommand_from history" -f -a "categories" -d 'List available categories'
complete -c pais -n "__fish_pais_using_subcommand help; and __fish_seen_subcommand_from history" -f -a "show" -d 'Show a specific history entry'
complete -c pais -n "__fish_pais_using_subcommand help; and __fish_seen_subcommand_from history" -f -a "stats" -d 'Show event statistics'
complete -c pais -n "__fish_pais_using_subcommand help; and __fish_seen_subcommand_from history" -f -a "events" -d 'List raw event dates'
complete -c pais -n "__fish_pais_using_subcommand help; and __fish_seen_subcommand_from config" -f -a "show" -d 'Show current configuration'
complete -c pais -n "__fish_pais_using_subcommand help; and __fish_seen_subcommand_from config" -f -a "get" -d 'Get a configuration value'
complete -c pais -n "__fish_pais_using_subcommand help; and __fish_seen_subcommand_from config" -f -a "set" -d 'Set a configuration value'
complete -c pais -n "__fish_pais_using_subcommand help; and __fish_seen_subcommand_from context" -f -a "inject" -d 'Inject skill context for SessionStart hook'
complete -c pais -n "__fish_pais_using_subcommand help; and __fish_seen_subcommand_from registry" -f -a "list" -d 'List configured registries'
complete -c pais -n "__fish_pais_using_subcommand help; and __fish_seen_subcommand_from registry" -f -a "add" -d 'Add a registry'
complete -c pais -n "__fish_pais_using_subcommand help; and __fish_seen_subcommand_from registry" -f -a "remove" -d 'Remove a registry'
complete -c pais -n "__fish_pais_using_subcommand help; and __fish_seen_subcommand_from registry" -f -a "update" -d 'Update registry listings'
complete -c pais -n "__fish_pais_using_subcommand help; and __fish_seen_subcommand_from registry" -f -a "search" -d 'Search for plugins in cached registries'
complete -c pais -n "__fish_pais_using_subcommand help; and __fish_seen_subcommand_from registry" -f -a "show" -d 'Show all plugins in a cached registry'
complete -c pais -n "__fish_pais_using_subcommand help; and __fish_seen_subcommand_from security" -f -a "tiers" -d 'Show security tiers and their actions'
complete -c pais -n "__fish_pais_using_subcommand help; and __fish_seen_subcommand_from security" -f -a "log" -d 'View security event log'
complete -c pais -n "__fish_pais_using_subcommand help; and __fish_seen_subcommand_from security" -f -a "test" -d 'Test a command against security patterns'
complete -c pais -n "__fish_pais_using_subcommand help; and __fish_seen_subcommand_from agent" -f -a "list" -d 'List available agents'
complete -c pais -n "__fish_pais_using_subcommand help; and __fish_seen_subcommand_from agent" -f -a "show" -d 'Show agent details'
complete -c pais -n "__fish_pais_using_subcommand help; and __fish_seen_subcommand_from agent" -f -a "traits" -d 'List available traits'
complete -c pais -n "__fish_pais_using_subcommand help; and __fish_seen_subcommand_from agent" -f -a "prompt" -d 'Generate prompt for an agent'
complete -c pais -n "__fish_pais_using_subcommand help; and __fish_seen_subcommand_from agent" -f -a "create" -d 'Create a new agent from template'
