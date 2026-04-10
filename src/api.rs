use crate::app::{AppMessage, FontAsset};
use std::path::PathBuf;
use tokio::sync::mpsc;

/// Main synchronization task that fetches themes and fonts from GitHub repositories in the background
pub async fn setup_app_task(tx: mpsc::Sender<AppMessage>, _themes_dir: PathBuf) {
    let client = reqwest::Client::new();

    // 1. Fetching available themes from the official Oh My Posh repository
    let themes_url = "https://api.github.com/repos/JanDeDobbeleer/oh-my-posh/contents/themes";
    let resp = client
        .get(themes_url)
        .header("User-Agent", "poshbuddy")
        .send()
        .await;

    if let Ok(r) = resp {
        if let Ok(json) = r.json::<serde_json::Value>().await {
            // Processing JSON response to extract filenames ending in .omp.json
            let names: Vec<String> = json
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|v| v["name"].as_str().map(|s| s.to_string()))
                .filter(|s| s.ends_with(".omp.json"))
                .collect();

            // Sending the theme list back to the main UI loop
            let _ = tx.send(AppMessage::ThemesLoaded(names)).await;
        }
    }

    // 2. Fetching Nerd Fonts metadata from the Nerd Fonts repository (patched fonts list)
    let fonts_url = "https://api.github.com/repos/ryanoasis/nerd-fonts/contents/patched-fonts";
    let resp_fonts = client
        .get(fonts_url)
        .header("User-Agent", "poshbuddy")
        .send()
        .await;

    if let Ok(r) = resp_fonts {
        if let Ok(json) = r.json::<serde_json::Value>().await {
            // Filtering for directories that represent different font families
            let fonts: Vec<FontAsset> = json
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter(|v| v["type"] == "dir")
                .filter_map(|v| {
                    v["name"].as_str().map(|s| FontAsset {
                        name: s.to_string(),
                    })
                })
                .collect();

            // Sending the font metadata back to the main UI loop
            let _ = tx.send(AppMessage::FontsLoaded(fonts)).await;
        }
    }
}
