use crate::app::{AppMessage, FontAsset, RemoteTheme};
use std::path::PathBuf;
use tokio::sync::mpsc;

/// Main synchronization task that fetches themes and fonts from GitHub repositories in the background
pub async fn setup_app_task(tx: mpsc::Sender<AppMessage>, themes_dir: PathBuf) {
    let themes_url = "https://api.github.com/repos/JanDeDobbeleer/oh-my-posh/contents/themes";
    let fonts_url = "https://api.github.com/repos/ryanoasis/nerd-fonts/contents/patched-fonts";
    setup_app_task_with_urls(tx, themes_dir, themes_url, fonts_url).await;
}

pub fn get_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent("poshbuddy")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

/// Checks if the system has an active internet connection by attempting a fast resolve
pub fn check_internet_connectivity() -> bool {
    // Attempting to resolve a reliable host or connecting to a public DNS
    use std::net::{TcpStream, ToSocketAddrs};
    let timeout = std::time::Duration::from_millis(1500);
    
    // We try to connect to a public DNS (Cloudflare) on port 53
    match "1.1.1.1:53".to_socket_addrs() {
        Ok(mut addrs) => {
            if let Some(addr) = addrs.next() {
                return TcpStream::connect_timeout(&addr, timeout).is_ok();
            }
        }
        Err(_) => return false,
    }
    false
}

/// Downloads a remote theme file to the local themes directory
pub async fn download_theme_file(name: &str, url: &str, target_dir: &std::path::Path) -> Result<std::path::PathBuf, String> {
    let client = get_client();
    let file_path = target_dir.join(format!("{}.omp.json", name));
    
    match client.get(url).send().await {
        Ok(resp) => {
            if !resp.status().is_success() {
                return Err(format!("Download failed: HTTP {}", resp.status()));
            }
            match resp.bytes().await {
                Ok(bytes) => {
                    if let Err(e) = tokio::fs::write(&file_path, &bytes).await {
                        return Err(format!("Disk write failed: {}", e));
                    }
                    Ok(file_path)
                }
                Err(e) => Err(format!("Network transfer failed: {}", e)),
            }
        }
        Err(e) => Err(format!("Network request failed: {}", e)),
    }
}

/// Downloads a remote theme file to a temporary location for previewing
pub async fn download_to_temp(name: &str, url: &str) -> Result<std::path::PathBuf, String> {
    let client = get_client();
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Failed to download theme for preview: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Server returned error: {}", response.status()));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read theme content: {}", e))?;

    let temp_dir = std::env::temp_dir();
    let temp_name = format!("poshbuddy_preview_{}.omp.json", name);
    let temp_path = temp_dir.join(temp_name);

    tokio::fs::write(&temp_path, &bytes).await.map_err(|e| format!("Failed to write preview file: {}", e))?;

    Ok(temp_path)
}

pub async fn setup_app_task_with_urls(
    tx: mpsc::Sender<AppMessage>,
    _themes_dir: std::path::PathBuf,
    themes_url: &str,
    fonts_url: &str,
) {
    let client = get_client();

    // 1. Fetching available themes from the official Oh My Posh repository
    let resp = client
        .get(themes_url)
        .header("User-Agent", "poshbuddy")
        .send()
        .await;

    if let Ok(r) = resp {
        if let Ok(json) = r.json::<serde_json::Value>().await {
            // Processing JSON response to extract filenames and download URLs
            let themes: Vec<RemoteTheme> = json
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|v| {
                    let name = v["name"].as_str()?.to_string();
                    if !name.ends_with(".omp.json") { return None; }
                    let download_url = v["download_url"].as_str()?.to_string();
                    let sha = v["sha"].as_str()?.to_string();
                    let clean_name = name.replace(".omp.json", "");
                    Some(RemoteTheme { name: clean_name, download_url, sha })
                })
                .collect();

            // Sending the remote themes metadata back to the main UI loop
            let _ = tx.send(AppMessage::RemoteThemesLoaded(themes)).await;
        }
    }

    // 2. Fetching Nerd Fonts metadata from the Nerd Fonts repository (patched fonts list)
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
                    let name = v["name"].as_str()?.to_string();
                    let download_url = v["html_url"].as_str().unwrap_or("").to_string();
                    Some(FontAsset { name, download_url })
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
                {"name": "theme1.omp.json", "type": "file", "download_url": "http://example.com/t1", "sha": "s1"},
                {"name": "theme2.omp.json", "type": "file", "download_url": "http://example.com/t2", "sha": "s2"}
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
        if let AppMessage::RemoteThemesLoaded(themes) = msg1 {
            assert_eq!(themes.len(), 2);
            assert_eq!(themes[0].name, "theme1");
            assert_eq!(themes[1].name, "theme2");
        } else {
            panic!("Expected RemoteThemesLoaded message");
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
                {"name": "theme1.omp.json", "type": "file", "download_url": "http://example.com/t1", "sha": "s1"},
                {"name": "readme.md", "type": "file", "download_url": "http://example.com/r", "sha": "sr"},
                {"name": "theme2.omp.json", "type": "file", "download_url": "http://example.com/t2", "sha": "s2"}
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
        if let AppMessage::RemoteThemesLoaded(themes) = msg1 {
            assert_eq!(themes.len(), 2);
            assert_eq!(themes[0].name, "theme1");
            assert_eq!(themes[1].name, "theme2");
        } else {
            panic!("Expected RemoteThemesLoaded message");
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
