use colored::*;
use eyre::{Context, Result};
use mermaid_rs::{
    Diagram, ERDiagram, FlowChart, FromConfig, Journey, MermaidClient, Mindmap, PieChart, RenderOptions,
    SequenceDiagram, StateDiagram,
};
use std::fs;
use std::io::{self, Read, Write as IoWrite};
use std::path::PathBuf;

use crate::cli::{DiagramAction, OutputFormat};
use crate::config::Config;

pub fn run(action: DiagramAction, _config: &Config) -> Result<()> {
    match action {
        DiagramAction::Render {
            file,
            mermaid,
            format,
            output,
            width,
            height,
            scale,
            background,
            server,
            clipboard,
            open,
        } => render(RenderArgs {
            file,
            mermaid,
            format,
            output,
            width,
            height,
            scale,
            background,
            server,
            clipboard,
            open,
        }),
        DiagramAction::Flowchart {
            direction,
            config,
            format,
            output,
            server,
        } => flowchart(&direction, config.as_ref(), &format, output.as_ref(), &server),
        DiagramAction::Sequence {
            config,
            format,
            output,
            server,
        } => sequence(config.as_ref(), &format, output.as_ref(), &server),
        DiagramAction::Er {
            config,
            format,
            output,
            server,
        } => er(config.as_ref(), &format, output.as_ref(), &server),
        DiagramAction::State {
            config,
            format,
            output,
            server,
        } => state(config.as_ref(), &format, output.as_ref(), &server),
        DiagramAction::Mindmap {
            config,
            format,
            output,
            server,
        } => mindmap(config.as_ref(), &format, output.as_ref(), &server),
        DiagramAction::Pie {
            title,
            show_data,
            config,
            format,
            output,
            server,
        } => pie(
            title.as_deref(),
            show_data,
            config.as_ref(),
            &format,
            output.as_ref(),
            &server,
        ),
        DiagramAction::Journey {
            title,
            config,
            format,
            output,
            server,
        } => journey(title.as_deref(), config.as_ref(), &format, output.as_ref(), &server),
        DiagramAction::Types { format } => list_types(OutputFormat::resolve(format)),
    }
}

struct RenderArgs {
    file: Option<PathBuf>,
    mermaid: Option<String>,
    format: String,
    output: Option<PathBuf>,
    width: Option<u32>,
    height: Option<u32>,
    scale: Option<f32>,
    background: Option<String>,
    server: String,
    clipboard: bool,
    open: bool,
}

fn render(args: RenderArgs) -> Result<()> {
    let script = get_script(args.file.as_ref(), args.mermaid.as_deref())?;

    let render_options = RenderOptions {
        width: args.width,
        height: args.height,
        scale: args.scale,
        background_color: args.background,
    };

    let format = args.format.to_lowercase();
    match format.as_str() {
        "mermaid" | "mmd" => {
            output_text(&script, args.output.as_ref(), args.clipboard)?;
        }
        "svg" => {
            let svg = render_svg(&script, &render_options, &args.server)?;
            output_text(&svg, args.output.as_ref(), args.clipboard)?;
        }
        "png" => {
            let png = render_png(&script, &render_options, &args.server)?;
            output_binary(&png, args.output.as_ref())?;
        }
        _ => eyre::bail!("Unsupported format: {}. Use svg, png, or mermaid.", format),
    }

    if args.open
        && let Some(path) = &args.output
    {
        open_file(path)?;
    }

    Ok(())
}

fn get_script(file: Option<&PathBuf>, mermaid: Option<&str>) -> Result<String> {
    if let Some(m) = mermaid {
        return Ok(m.to_string());
    }

    if let Some(f) = file {
        return fs::read_to_string(f).context("Failed to read mermaid file");
    }

    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .context("Failed to read from stdin")?;

    if buffer.trim().is_empty() {
        eyre::bail!("No input provided. Use --mermaid, provide a file, or pipe to stdin.");
    }

    Ok(buffer)
}

fn render_svg(script: &str, options: &RenderOptions, server: &str) -> Result<String> {
    let rt = tokio::runtime::Runtime::new().context("Failed to create tokio runtime")?;
    let client = MermaidClient::new(Some(server.to_string()));

    rt.block_on(async { client.render_svg_from_script(script, options).await })
        .map_err(|e| eyre::eyre!("Render failed: {}", e))
}

fn render_png(script: &str, options: &RenderOptions, server: &str) -> Result<Vec<u8>> {
    let rt = tokio::runtime::Runtime::new().context("Failed to create tokio runtime")?;
    let client = MermaidClient::new(Some(server.to_string()));

    rt.block_on(async { client.render_png_from_script(script, options).await })
        .map_err(|e| eyre::eyre!("Render failed: {}", e))
}

fn output_text(content: &str, output: Option<&PathBuf>, clipboard: bool) -> Result<()> {
    if clipboard {
        copy_to_clipboard(content)?;
        eprintln!("{} Copied to clipboard", "✓".green());
    }

    if let Some(path) = output {
        fs::write(path, content).context("Failed to write output file")?;
        eprintln!("{} Saved: {}", "✓".green(), path.display());
    } else if !clipboard {
        print!("{}", content);
    }

    Ok(())
}

fn output_binary(content: &[u8], output: Option<&PathBuf>) -> Result<()> {
    if let Some(path) = output {
        fs::write(path, content).context("Failed to write output file")?;
        eprintln!("{} Saved: {}", "✓".green(), path.display());
    } else {
        io::stdout().write_all(content).context("Failed to write to stdout")?;
    }

    Ok(())
}

fn copy_to_clipboard(content: &str) -> Result<()> {
    use std::process::{Command, Stdio};

    // Try xclip first, then xsel, then wl-copy (Wayland)
    let clipboard_cmds = [
        ("xclip", vec!["-selection", "clipboard"]),
        ("xsel", vec!["--clipboard", "--input"]),
        ("wl-copy", vec![]),
    ];

    for (cmd, args) in &clipboard_cmds {
        if which::which(cmd).is_ok() {
            let mut child = Command::new(cmd)
                .args(args)
                .stdin(Stdio::piped())
                .spawn()
                .context("Failed to spawn clipboard command")?;

            if let Some(stdin) = child.stdin.as_mut() {
                stdin
                    .write_all(content.as_bytes())
                    .context("Failed to write to clipboard")?;
            }

            child.wait().context("Clipboard command failed")?;
            return Ok(());
        }
    }

    eyre::bail!("No clipboard utility found (tried xclip, xsel, wl-copy)")
}

fn open_file(path: &PathBuf) -> Result<()> {
    use std::process::Command;

    #[cfg(target_os = "linux")]
    let cmd = "xdg-open";
    #[cfg(target_os = "macos")]
    let cmd = "open";
    #[cfg(target_os = "windows")]
    let cmd = "start";

    Command::new(cmd).arg(path).spawn().context("Failed to open file")?;

    Ok(())
}

fn render_diagram<D: Diagram>(diagram: &D, format: &str, output: Option<&PathBuf>, server: &str) -> Result<()> {
    let script = diagram.build_script();

    match format.to_lowercase().as_str() {
        "mermaid" | "mmd" => {
            output_text(&script, output, false)?;
        }
        "svg" => {
            let svg = render_svg(&script, &RenderOptions::default(), server)?;
            output_text(&svg, output, false)?;
        }
        "png" => {
            let png = render_png(&script, &RenderOptions::default(), server)?;
            output_binary(&png, output)?;
        }
        _ => eyre::bail!("Unsupported format: {}. Use svg, png, or mermaid.", format),
    }

    Ok(())
}

fn load_config_or_stdin<D: FromConfig>(config: Option<&PathBuf>) -> Result<D> {
    let content = read_config_or_stdin(config)?;
    D::from_yaml(&content).map_err(|e| eyre::eyre!("Failed to parse config: {}", e))
}

fn read_config_or_stdin(config: Option<&PathBuf>) -> Result<String> {
    let content = if let Some(path) = config {
        fs::read_to_string(path).context("Failed to read config file")?
    } else {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .context("Failed to read from stdin")?;
        buffer
    };

    if content.trim().is_empty() {
        eyre::bail!("No config provided. Use --config or pipe YAML to stdin.");
    }

    Ok(content)
}

fn flowchart(
    direction: &str,
    config: Option<&PathBuf>,
    format: &str,
    output: Option<&PathBuf>,
    server: &str,
) -> Result<()> {
    let diagram: FlowChart = if let Some(path) = config {
        let content = fs::read_to_string(path).context("Failed to read config file")?;
        FlowChart::from_yaml(&content).map_err(|e| eyre::eyre!("Failed to parse config: {}", e))?
    } else {
        let dir = match direction.to_uppercase().as_str() {
            "TB" | "TD" => mermaid_rs::Direction::TopBottom,
            "BT" => mermaid_rs::Direction::BottomTop,
            "LR" => mermaid_rs::Direction::LeftRight,
            "RL" => mermaid_rs::Direction::RightLeft,
            _ => eyre::bail!("Invalid direction: {}. Use TB, BT, LR, or RL.", direction),
        };

        eprintln!(
            "{} No config provided, reading YAML from stdin (direction: {})",
            "→".blue(),
            direction
        );

        let mut content = String::new();
        io::stdin()
            .read_to_string(&mut content)
            .context("Failed to read from stdin")?;

        if content.trim().is_empty() {
            eyre::bail!("No config provided. Use --config or pipe YAML to stdin.");
        }

        let mut diagram: FlowChart =
            FlowChart::from_yaml(&content).map_err(|e| eyre::eyre!("Failed to parse config: {}", e))?;
        diagram.direction = dir;
        diagram
    };

    render_diagram(&diagram, format, output, server)
}

fn sequence(config: Option<&PathBuf>, format: &str, output: Option<&PathBuf>, server: &str) -> Result<()> {
    let diagram: SequenceDiagram = load_config_or_stdin(config)?;
    render_diagram(&diagram, format, output, server)
}

fn er(config: Option<&PathBuf>, format: &str, output: Option<&PathBuf>, server: &str) -> Result<()> {
    let content = read_config_or_stdin(config)?;
    let diagram = ERDiagram::from_yaml(&content).map_err(|e| eyre::eyre!("Failed to parse config: {}", e))?;
    render_diagram(&diagram, format, output, server)
}

fn state(config: Option<&PathBuf>, format: &str, output: Option<&PathBuf>, server: &str) -> Result<()> {
    let diagram: StateDiagram = load_config_or_stdin(config)?;
    render_diagram(&diagram, format, output, server)
}

fn mindmap(config: Option<&PathBuf>, format: &str, output: Option<&PathBuf>, server: &str) -> Result<()> {
    let content = read_config_or_stdin(config)?;
    let diagram = Mindmap::from_yaml(&content).map_err(|e| eyre::eyre!("Failed to parse config: {}", e))?;
    render_diagram(&diagram, format, output, server)
}

fn pie(
    title: Option<&str>,
    show_data: bool,
    config: Option<&PathBuf>,
    format: &str,
    output: Option<&PathBuf>,
    server: &str,
) -> Result<()> {
    let mut diagram: PieChart = load_config_or_stdin(config)?;

    if let Some(t) = title {
        diagram.title = Some(t.to_string());
    }
    if show_data {
        diagram.show_data = true;
    }

    render_diagram(&diagram, format, output, server)
}

fn journey(
    title: Option<&str>,
    config: Option<&PathBuf>,
    format: &str,
    output: Option<&PathBuf>,
    server: &str,
) -> Result<()> {
    let content = read_config_or_stdin(config)?;
    let mut diagram = Journey::from_yaml(&content).map_err(|e| eyre::eyre!("Failed to parse config: {}", e))?;

    if let Some(t) = title {
        diagram.title = Some(t.to_string());
    }

    render_diagram(&diagram, format, output, server)
}

fn list_types(format: OutputFormat) -> Result<()> {
    let types = vec![
        serde_json::json!({
            "name": "flowchart",
            "description": "Flowchart/graph diagrams with nodes and edges",
            "subcommand": "flowchart",
            "config_example": "direction: TB\nnodes:\n  - id: A\n    label: Start\nlinks:\n  - from: A\n    to: B"
        }),
        serde_json::json!({
            "name": "sequence",
            "description": "Sequence diagrams showing interactions between participants",
            "subcommand": "sequence",
            "config_example": "participants:\n  - id: Alice\n  - id: Bob\nmessages:\n  - from: Alice\n    to: Bob\n    text: Hello!"
        }),
        serde_json::json!({
            "name": "er",
            "description": "Entity-relationship diagrams for data modeling",
            "subcommand": "er",
            "config_example": "entities:\n  - name: User\n    attributes:\n      - name: id\n        type: int\n        key: PK"
        }),
        serde_json::json!({
            "name": "state",
            "description": "State diagrams showing state transitions",
            "subcommand": "state",
            "config_example": "states:\n  - id: idle\n    label: Idle\ntransitions:\n  - from: idle\n    to: running"
        }),
        serde_json::json!({
            "name": "mindmap",
            "description": "Mind map diagrams for hierarchical information",
            "subcommand": "mindmap",
            "config_example": "root:\n  text: Central Topic\n  children:\n    - text: Branch 1\n    - text: Branch 2"
        }),
        serde_json::json!({
            "name": "pie",
            "description": "Pie charts for showing proportions",
            "subcommand": "pie",
            "config_example": "title: Browser Market Share\nslices:\n  - label: Chrome\n    value: 65\n  - label: Firefox\n    value: 20"
        }),
        serde_json::json!({
            "name": "journey",
            "description": "User journey diagrams for mapping experiences",
            "subcommand": "journey",
            "config_example": "title: User Login Journey\nsections:\n  - name: Landing\n    tasks:\n      - name: Visit page\n        score: 5"
        }),
    ];

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&types)?);
        }
        OutputFormat::Yaml => {
            println!("{}", serde_yaml::to_string(&types)?);
        }
        OutputFormat::Text => {
            println!("{}", "Available Diagram Types".cyan().bold());
            println!();
            for dtype in &types {
                println!(
                    "  {} - {}",
                    dtype["name"].as_str().unwrap().green(),
                    dtype["description"].as_str().unwrap()
                );
                println!(
                    "    Command: pais diagram {}",
                    dtype["subcommand"].as_str().unwrap().yellow()
                );
                println!();
            }
            println!("{}", "Render any .mmd file:".dimmed());
            println!("  pais diagram render diagram.mmd -o output.svg");
            println!();
        }
    }

    Ok(())
}
