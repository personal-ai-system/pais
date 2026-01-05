use colored::*;
use eyre::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::cli::{ImageAction, OutputFormat};
use crate::config::Config;

// Note: Config::pais_dir() is a static method that returns the PAIS directory

/// Supported AI models for image generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Model {
    Gemini,
    Flux,
    OpenAi,
}

impl std::str::FromStr for Model {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "gemini" => Ok(Model::Gemini),
            "flux" => Ok(Model::Flux),
            "openai" | "dall-e" | "gpt-image" => Ok(Model::OpenAi),
            _ => eyre::bail!("Unknown model: {}. Supported: gemini, flux, openai", s),
        }
    }
}

impl Model {
    fn env_var(&self) -> &'static str {
        match self {
            Model::Gemini => "GOOGLE_API_KEY",
            Model::Flux => "REPLICATE_API_TOKEN",
            Model::OpenAi => "OPENAI_API_KEY",
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Model::Gemini => "Gemini",
            Model::Flux => "Flux",
            Model::OpenAi => "OpenAI",
        }
    }
}

/// Gemini API response structures
#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
}

#[derive(Debug, Deserialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiPart {
    inline_data: Option<GeminiInlineData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiInlineData {
    #[serde(rename = "mimeType")]
    _mime_type: String,
    data: String,
}

/// Gemini API request structures
#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiRequestContent>,
    #[serde(rename = "generationConfig")]
    generation_config: GeminiGenerationConfig,
}

#[derive(Debug, Serialize)]
struct GeminiRequestContent {
    parts: Vec<GeminiRequestPart>,
}

#[derive(Debug, Serialize)]
struct GeminiRequestPart {
    text: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GeminiGenerationConfig {
    response_modalities: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image_generation_config: Option<GeminiImageConfig>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GeminiImageConfig {
    aspect_ratio: String,
}

struct GenerateOptions<'a> {
    prompt: &'a str,
    model: &'a str,
    size: Option<&'a str>,
    aspect_ratio: Option<&'a str>,
    output: Option<&'a PathBuf>,
    remove_bg: bool,
    thumbnail: bool,
}

pub fn run(action: ImageAction, config: &Config) -> Result<()> {
    match action {
        ImageAction::Generate {
            prompt,
            model,
            size,
            aspect_ratio,
            output,
            remove_bg,
            thumbnail,
        } => {
            let opts = GenerateOptions {
                prompt: &prompt,
                model: &model,
                size: size.as_deref(),
                aspect_ratio: aspect_ratio.as_deref(),
                output: output.as_ref(),
                remove_bg,
                thumbnail,
            };
            generate(opts, config)
        }
        ImageAction::Models { format } => list_models(OutputFormat::resolve(format)),
    }
}

fn generate(opts: GenerateOptions, config: &Config) -> Result<()> {
    let model: Model = opts.model.parse()?;

    // Get API key
    let api_key = get_api_key(&model, config)?;

    // Determine output path
    let output_path = opts.output.cloned().unwrap_or_else(|| {
        dirs::download_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("pais-image.png")
    });

    // Ensure output directory exists
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).context("Failed to create output directory")?;
    }

    println!("{} Generating image with {}...", "→".blue(), model.name().cyan());

    // Generate based on model
    match model {
        Model::Gemini => {
            let size = opts.size.unwrap_or("2K");
            let aspect_ratio = opts.aspect_ratio.unwrap_or("16:9");
            generate_gemini(opts.prompt, size, aspect_ratio, &output_path, &api_key)?;
        }
        Model::Flux => {
            let aspect_ratio = opts.aspect_ratio.unwrap_or("16:9");
            generate_flux(opts.prompt, aspect_ratio, &output_path, &api_key)?;
        }
        Model::OpenAi => {
            let size = opts.size.unwrap_or("1024x1024");
            generate_openai(opts.prompt, size, &output_path, &api_key)?;
        }
    }

    println!("{} Saved: {}", "✓".green(), output_path.display());

    // Post-processing
    if opts.remove_bg || opts.thumbnail {
        remove_background(&output_path, config)?;
    }

    if opts.thumbnail {
        let thumb_path = output_path.with_extension("").to_string_lossy().to_string() + "-thumb.png";
        let thumb_path = PathBuf::from(thumb_path);
        add_background(&output_path, &thumb_path, "#0a0a0f")?;
        println!("{} Thumbnail: {}", "✓".green(), thumb_path.display());
    }

    Ok(())
}

fn get_api_key(model: &Model, _config: &Config) -> Result<String> {
    let env_var = model.env_var();

    // Check environment variable first
    if let Ok(key) = std::env::var(env_var) {
        return Ok(key);
    }

    // Check ~/.config/pais/.env file
    let pais_dir = Config::pais_dir();
    let env_file = pais_dir.join(".env");
    if env_file.exists() {
        let content = fs::read_to_string(&env_file).context("Failed to read .env file")?;
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            if let Some((key, value)) = line.split_once('=')
                && key.trim() == env_var
            {
                let value = value.trim().trim_matches('"').trim_matches('\'');
                return Ok(value.to_string());
            }
        }
    }

    eyre::bail!(
        "Missing API key: {} not found in environment or {}",
        env_var,
        env_file.display()
    )
}

fn generate_gemini(prompt: &str, _size: &str, aspect_ratio: &str, output: &PathBuf, api_key: &str) -> Result<()> {
    log::info!("Generating with Gemini, aspect_ratio={}", aspect_ratio);

    let request = GeminiRequest {
        contents: vec![GeminiRequestContent {
            parts: vec![GeminiRequestPart {
                text: prompt.to_string(),
            }],
        }],
        generation_config: GeminiGenerationConfig {
            response_modalities: vec!["TEXT".to_string(), "IMAGE".to_string()],
            image_generation_config: Some(GeminiImageConfig {
                aspect_ratio: aspect_ratio.to_string(),
            }),
        },
    };

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash-exp:generateContent?key={}",
        api_key
    );

    let request_body = serde_json::to_string(&request).context("Failed to serialize request")?;

    let mut response = ureq::post(&url)
        .header("Content-Type", "application/json")
        .send(request_body.as_bytes())
        .context("Failed to call Gemini API")?;

    let response_body = response
        .body_mut()
        .read_to_string()
        .context("Failed to read response")?;
    let response: GeminiResponse = serde_json::from_str(&response_body).context("Failed to parse Gemini response")?;

    // Find image data in response
    let image_data = response
        .candidates
        .iter()
        .flat_map(|c| &c.content.parts)
        .find_map(|p| p.inline_data.as_ref())
        .ok_or_else(|| eyre::eyre!("No image data in Gemini response"))?;

    // Decode base64 and save
    let decoded = base64_decode(&image_data.data)?;
    fs::write(output, decoded).context("Failed to write image file")?;

    Ok(())
}

fn generate_flux(prompt: &str, aspect_ratio: &str, output: &PathBuf, api_key: &str) -> Result<()> {
    log::info!("Generating with Flux, aspect_ratio={}", aspect_ratio);

    // Replicate API for Flux
    let request = serde_json::json!({
        "version": "black-forest-labs/flux-1.1-pro",
        "input": {
            "prompt": prompt,
            "aspect_ratio": aspect_ratio,
            "output_format": "png",
            "output_quality": 95
        }
    });

    let request_body = serde_json::to_string(&request).context("Failed to serialize request")?;

    let mut response = ureq::post("https://api.replicate.com/v1/predictions")
        .header("Authorization", &format!("Token {}", api_key))
        .header("Content-Type", "application/json")
        .send(request_body.as_bytes())
        .context("Failed to call Replicate API")?;

    let response_body = response
        .body_mut()
        .read_to_string()
        .context("Failed to read response")?;
    let response: serde_json::Value =
        serde_json::from_str(&response_body).context("Failed to parse Replicate response")?;

    // Get prediction ID and poll for completion
    let prediction_id = response["id"]
        .as_str()
        .ok_or_else(|| eyre::eyre!("No prediction ID in response"))?;

    let image_url = poll_replicate(prediction_id, api_key)?;

    // Download image
    let image_data = ureq::get(&image_url)
        .call()
        .context("Failed to download image")?
        .body_mut()
        .read_to_vec()
        .context("Failed to read image data")?;

    fs::write(output, image_data).context("Failed to write image file")?;

    Ok(())
}

fn poll_replicate(prediction_id: &str, api_key: &str) -> Result<String> {
    let url = format!("https://api.replicate.com/v1/predictions/{}", prediction_id);

    for _ in 0..60 {
        std::thread::sleep(std::time::Duration::from_secs(2));

        let mut response = ureq::get(&url)
            .header("Authorization", &format!("Token {}", api_key))
            .call()
            .context("Failed to poll Replicate")?;

        let response_body = response
            .body_mut()
            .read_to_string()
            .context("Failed to read response")?;
        let response: serde_json::Value =
            serde_json::from_str(&response_body).context("Failed to parse poll response")?;

        let status = response["status"].as_str().unwrap_or("");

        match status {
            "succeeded" => {
                let output = response["output"]
                    .as_str()
                    .or_else(|| {
                        response["output"]
                            .as_array()
                            .and_then(|a: &Vec<serde_json::Value>| a.first())
                            .and_then(|v: &serde_json::Value| v.as_str())
                    })
                    .ok_or_else(|| eyre::eyre!("No output URL in response"))?;
                return Ok(output.to_string());
            }
            "failed" | "canceled" => {
                let error = response["error"].as_str().unwrap_or("Unknown error");
                eyre::bail!("Replicate prediction failed: {}", error);
            }
            _ => continue,
        }
    }

    eyre::bail!("Replicate prediction timed out")
}

fn generate_openai(prompt: &str, size: &str, output: &PathBuf, api_key: &str) -> Result<()> {
    log::info!("Generating with OpenAI, size={}", size);

    let request = serde_json::json!({
        "model": "dall-e-3",
        "prompt": prompt,
        "size": size,
        "n": 1,
        "response_format": "b64_json"
    });

    let request_body = serde_json::to_string(&request).context("Failed to serialize request")?;

    let mut response = ureq::post("https://api.openai.com/v1/images/generations")
        .header("Authorization", &format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .send(request_body.as_bytes())
        .context("Failed to call OpenAI API")?;

    let response_body = response
        .body_mut()
        .read_to_string()
        .context("Failed to read response")?;
    let response: serde_json::Value =
        serde_json::from_str(&response_body).context("Failed to parse OpenAI response")?;

    let image_data = response["data"][0]["b64_json"]
        .as_str()
        .ok_or_else(|| eyre::eyre!("No image data in OpenAI response"))?;

    let decoded = base64_decode(image_data)?;
    fs::write(output, decoded).context("Failed to write image file")?;

    Ok(())
}

fn base64_decode(data: &str) -> Result<Vec<u8>> {
    // Simple base64 decoder
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let data = data.replace(['\n', '\r', ' '], "");
    let data = data.trim_end_matches('=');

    let mut result = Vec::with_capacity(data.len() * 3 / 4);
    let mut buffer = 0u32;
    let mut bits = 0;

    for byte in data.bytes() {
        let value = ALPHABET
            .iter()
            .position(|&c| c == byte)
            .ok_or_else(|| eyre::eyre!("Invalid base64 character"))? as u32;

        buffer = (buffer << 6) | value;
        bits += 6;

        if bits >= 8 {
            bits -= 8;
            result.push((buffer >> bits) as u8);
        }
    }

    Ok(result)
}

fn remove_background(image_path: &Path, _config: &Config) -> Result<()> {
    let pais_dir = Config::pais_dir();
    let api_key = std::env::var("REMOVEBG_API_KEY")
        .or_else(|_| {
            let env_file = pais_dir.join(".env");
            if env_file.exists() {
                let content = fs::read_to_string(&env_file)?;
                for line in content.lines() {
                    if let Some((key, value)) = line.split_once('=')
                        && key.trim() == "REMOVEBG_API_KEY"
                    {
                        return Ok(value.trim().trim_matches('"').to_string());
                    }
                }
            }
            Err(eyre::eyre!("REMOVEBG_API_KEY not found"))
        })
        .context("--remove-bg requires REMOVEBG_API_KEY")?;

    println!("{} Removing background...", "→".blue());

    // Use curl for multipart form upload (simpler than implementing in Rust)
    let output = Command::new("curl")
        .args([
            "-s",
            "-X",
            "POST",
            "-H",
            &format!("X-Api-Key: {}", api_key),
            "-F",
            &format!("image_file=@{}", image_path.display()),
            "-F",
            "size=auto",
            "https://api.remove.bg/v1.0/removebg",
            "-o",
            &image_path.to_string_lossy(),
        ])
        .output()
        .context("Failed to run curl for background removal")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eyre::bail!("Background removal failed: {}", stderr);
    }

    println!("{} Background removed", "✓".green());
    Ok(())
}

fn add_background(input: &Path, output: &Path, color: &str) -> Result<()> {
    // Use ImageMagick
    let status = Command::new("magick")
        .args([
            input.to_str().unwrap(),
            "-background",
            color,
            "-flatten",
            output.to_str().unwrap(),
        ])
        .status()
        .context("Failed to run ImageMagick (is it installed?)")?;

    if !status.success() {
        eyre::bail!("ImageMagick failed to add background");
    }

    Ok(())
}

fn list_models(format: OutputFormat) -> Result<()> {
    let models = vec![
        serde_json::json!({
            "name": "gemini",
            "provider": "Google",
            "env_var": "GOOGLE_API_KEY",
            "sizes": ["1K", "2K", "4K"],
            "aspect_ratios": ["1:1", "16:9", "3:2", "9:16", "21:9"],
            "notes": "Best quality, recommended default"
        }),
        serde_json::json!({
            "name": "flux",
            "provider": "Replicate (Black Forest Labs)",
            "env_var": "REPLICATE_API_TOKEN",
            "sizes": ["1:1", "16:9", "3:2", "2:3", "3:4", "4:3", "4:5", "5:4", "9:16", "21:9"],
            "notes": "Alternative aesthetic"
        }),
        serde_json::json!({
            "name": "openai",
            "provider": "OpenAI",
            "env_var": "OPENAI_API_KEY",
            "sizes": ["1024x1024", "1536x1024", "1024x1536"],
            "notes": "DALL-E 3"
        }),
    ];

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&models)?);
        }
        OutputFormat::Yaml => {
            println!("{}", serde_yaml::to_string(&models)?);
        }
        OutputFormat::Text => {
            println!("{}", "Available Models".cyan().bold());
            println!();
            for model in &models {
                println!(
                    "  {} ({})",
                    model["name"].as_str().unwrap().green(),
                    model["provider"].as_str().unwrap()
                );
                println!("    API Key: {}", model["env_var"].as_str().unwrap().yellow());
                println!(
                    "    Sizes: {}",
                    model["sizes"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_str().unwrap())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                if let Some(ar) = model["aspect_ratios"].as_array() {
                    println!(
                        "    Aspect Ratios: {}",
                        ar.iter().map(|v| v.as_str().unwrap()).collect::<Vec<_>>().join(", ")
                    );
                }
                println!("    {}", model["notes"].as_str().unwrap().dimmed());
                println!();
            }
        }
    }

    Ok(())
}
