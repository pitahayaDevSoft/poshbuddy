use crate::app::{AppMessage, FontAsset};
use std::path::PathBuf;
use tokio::sync::mpsc;

/// Main synchronization task that fetches themes and fonts from GitHub repositories in the background
pub async fn setup_app_task(tx: mpsc::Sender<AppMessage>, themes_dir: PathBuf) {
    let themes_url = "https://api.github.com/repos/JanDeDobbeleer/oh-my-posh/contents/themes";
    let fonts_url = "https://api.github.com/repos/ryanoasis/nerd-fonts/contents/patched-fonts";
    setup_app_task_with_urls(tx, themes_dir, themes_url, fonts_url).await;
}

pub async fn setup_app_task_with_urls(
    tx: mpsc::Sender<AppMessage>,
    _themes_dir: PathBuf,
    _themes_url: &str,
    _fonts_url: &str,
) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;
    use std::path::PathBuf;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_setup_app_task_success() {
        let mut server = Server::new_async().await;

        let theme_mock = server
            .mock("GET", "/themes")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"
            [
                {"name": "theme1.omp.json", "type": "file"},
                {"name": "theme2.omp.json", "type": "file"}
            ]
            "#,
            )
            .create_async()
            .await;

        let font_mock = server
            .mock("GET", "/fonts")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"
            [
                {"name": "FiraCode", "type": "dir"},
                {"name": "Hack", "type": "dir"}
            ]
            "#,
            )
            .create_async()
            .await;

        let themes_url = format!("{}/themes", server.url());
        let fonts_url = format!("{}/fonts", server.url());

        let (tx, mut rx) = mpsc::channel(10);

        setup_app_task_with_urls(tx, PathBuf::from("dummy"), &themes_url, &fonts_url).await;

        let msg1 = rx.recv().await.unwrap();
        if let AppMessage::ThemesLoaded(themes) = msg1 {
            assert_eq!(themes.len(), 2);
            assert_eq!(themes[0], "theme1.omp.json");
            assert_eq!(themes[1], "theme2.omp.json");
        } else {
            panic!("Expected ThemesLoaded message");
        }

        let msg2 = rx.recv().await.unwrap();
        if let AppMessage::FontsLoaded(fonts) = msg2 {
            assert_eq!(fonts.len(), 2);
            assert_eq!(fonts[0].name, "FiraCode");
            assert_eq!(fonts[1].name, "Hack");
        } else {
            panic!("Expected FontsLoaded message");
        }

        theme_mock.assert_async().await;
        font_mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_setup_app_task_ignores_invalid_themes() {
        let mut server = Server::new_async().await;

        let _theme_mock = server
            .mock("GET", "/themes")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"
            [
                {"name": "theme1.omp.json", "type": "file"},
                {"name": "readme.md", "type": "file"},
                {"name": "theme2.omp.json", "type": "file"}
            ]
            "#,
            )
            .create_async()
            .await;

        let _font_mock = server
            .mock("GET", "/fonts")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"[]"#)
            .create_async()
            .await;

        let themes_url = format!("{}/themes", server.url());
        let fonts_url = format!("{}/fonts", server.url());

        let (tx, mut rx) = mpsc::channel(10);

        setup_app_task_with_urls(tx, PathBuf::from("dummy"), &themes_url, &fonts_url).await;

        let msg1 = rx.recv().await.unwrap();
        if let AppMessage::ThemesLoaded(themes) = msg1 {
            assert_eq!(themes.len(), 2);
            assert_eq!(themes[0], "theme1.omp.json");
            assert_eq!(themes[1], "theme2.omp.json");
        } else {
            panic!("Expected ThemesLoaded message");
        }
    }

    #[tokio::test]
    async fn test_setup_app_task_ignores_invalid_fonts() {
        let mut server = Server::new_async().await;

        let _theme_mock = server
            .mock("GET", "/themes")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"[]"#)
            .create_async()
            .await;

        let _font_mock = server
            .mock("GET", "/fonts")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"
            [
                {"name": "FiraCode", "type": "dir"},
                {"name": "SomeFile.ttf", "type": "file"},
                {"name": "Hack", "type": "dir"}
            ]
            "#,
            )
            .create_async()
            .await;

        let themes_url = format!("{}/themes", server.url());
        let fonts_url = format!("{}/fonts", server.url());

        let (tx, mut rx) = mpsc::channel(10);

        setup_app_task_with_urls(tx, PathBuf::from("dummy"), &themes_url, &fonts_url).await;

        // Wait for ThemesLoaded (empty)
        let _ = rx.recv().await.unwrap();

        let msg2 = rx.recv().await.unwrap();
        if let AppMessage::FontsLoaded(fonts) = msg2 {
            assert_eq!(fonts.len(), 2);
            assert_eq!(fonts[0].name, "FiraCode");
            assert_eq!(fonts[1].name, "Hack");
        } else {
            panic!("Expected FontsLoaded message");
        }
    }

    #[tokio::test]
    async fn test_setup_app_task_handles_http_errors() {
        let mut server = Server::new_async().await;

        let _theme_mock = server
            .mock("GET", "/themes")
            .with_status(500)
            .create_async()
            .await;

        let _font_mock = server
            .mock("GET", "/fonts")
            .with_status(404)
            .create_async()
            .await;

        let themes_url = format!("{}/themes", server.url());
        let fonts_url = format!("{}/fonts", server.url());

        let (tx, mut rx) = mpsc::channel(10);

        setup_app_task_with_urls(tx, PathBuf::from("dummy"), &themes_url, &fonts_url).await;

        // Channel should be dropped without sending any messages
        assert!(rx.recv().await.is_none());
    }

    #[tokio::test]
    async fn test_setup_app_task_handles_invalid_json() {
        let mut server = Server::new_async().await;

        let _theme_mock = server
            .mock("GET", "/themes")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"invalid": json}"#)
            .create_async()
            .await;

        let _font_mock = server
            .mock("GET", "/fonts")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"<not>json</not>"#)
            .create_async()
            .await;

        let themes_url = format!("{}/themes", server.url());
        let fonts_url = format!("{}/fonts", server.url());

        let (tx, mut rx) = mpsc::channel(10);

        setup_app_task_with_urls(tx, PathBuf::from("dummy"), &themes_url, &fonts_url).await;

        // Channel should be dropped without sending any messages
        assert!(rx.recv().await.is_none());
    }
}
