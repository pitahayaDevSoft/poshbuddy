use super::*;
use ratatui::Terminal;
use ratatui::backend::TestBackend;

fn setup_test_app(state: AppState) -> App {
    let mut app = App::new();
    app.state = state;
    app
}

#[test]
fn test_ui_welcome_state() {
    let mut app = setup_test_app(AppState::Welcome);
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|f| ui(f, &mut app)).unwrap();

    let buffer = terminal.backend().buffer();
    assert_eq!(buffer.area.width, 80);
    assert_eq!(buffer.area.height, 24);
}

#[test]
fn test_ui_dependency_missing_state() {
    let mut app = setup_test_app(AppState::DependencyMissing);
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|f| ui(f, &mut app)).unwrap();

    let buffer = terminal.backend().buffer();
    assert_eq!(buffer.area.width, 80);
}

#[test]
fn test_ui_loading_state() {
    let mut app = setup_test_app(AppState::Loading);
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|f| ui(f, &mut app)).unwrap();

    let buffer = terminal.backend().buffer();
    assert_eq!(buffer.area.width, 80);
}

#[test]
fn test_ui_installing_dependency_state() {
    let mut app = setup_test_app(AppState::InstallingDependency {
        log: vec!["Downloading".to_string()],
        current_action: "Installing".to_string(),
    });
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|f| ui(f, &mut app)).unwrap();

    let buffer = terminal.backend().buffer();
    assert_eq!(buffer.area.width, 80);
}

#[test]
fn test_ui_main_state() {
    let mut app = setup_test_app(AppState::Main);
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|f| ui(f, &mut app)).unwrap();

    let buffer = terminal.backend().buffer();
    assert_eq!(buffer.area.width, 80);
}
