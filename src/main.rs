use std::{error::Error, io, panic::{set_hook, take_hook}, sync::mpsc::{self, TryRecvError}, thread};

use color_eyre::eyre::Result;
use crossterm::{event::{self, Event, KeyCode}, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};
use dotenv::dotenv;
use ratatui::{backend::{Backend, CrosstermBackend},Terminal};
use repository_list::RepositoryList;

mod app;
mod repository_list;
mod ui;
mod logging;
mod current_screen;
mod dependabot;
mod repository;
use crate::app::{App, DependabotTrackerError, DependabotScrollbar};
use crate::repository::fetch_github_repos;
use crate::current_screen::CurrentScreen;
use crate::logging::initialize_logging;

fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    initialize_logging()?;
    init_panic_hook();
    let mut tui = init_tui()?;
    let mut app = App::new();
    let res = run_app(&mut tui, &mut app);
    let _ = restore_tui();

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

pub fn init_panic_hook() {
    let original_hook = take_hook();
    set_hook(Box::new(move |panic_info| {
        // intentionally ignore errors here since we're already in a panic
        let _ = restore_tui();
        original_hook(panic_info);
    }));
}

pub fn init_tui() -> io::Result<Terminal<impl Backend>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    
    Terminal::new(backend)
}

pub fn restore_tui() -> io::Result<()> {
    disable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        LeaveAlternateScreen,
    )?;

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<(), DependabotTrackerError> {
    loop {
        terminal.draw(|f| ui::ui(f, app))
        .map_err(|e| Box::new(e) as DependabotTrackerError)?;

        if let Event::Key(key) = event::read()
        .map_err(|e| Box::new(e) as DependabotTrackerError)? {
            if key.kind == event::KeyEventKind::Release {
                // Skip events that are not KeyEventKind::Press
                continue;
            }
            match app.current_screen {
                CurrentScreen::Overview => match key.code {
                    KeyCode::Char('r') => {
                        app.current_screen = CurrentScreen::ProjectList;
                    }
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    KeyCode::Char('u') => {
                        app.current_screen = CurrentScreen::Update;
                    }
                    _ => {}
                },
                CurrentScreen::Update => match key.code {
                    KeyCode::Char('y') => {
                        app.current_screen = CurrentScreen::Updating;
                        let (tx, rx) = mpsc::channel();
                        let username = app.username.clone();
                        let token = app.token.clone();

                        thread::spawn(move || {
                            let result: Result<RepositoryList, DependabotTrackerError> = fetch_github_repos(&username, &token);
                            tx.send(result).unwrap();
                        });

                        app.current_screen = CurrentScreen::Updating;
                        app.fetching = Some(rx);
                        
                    }
                    KeyCode::Char('n') => {
                        app.current_screen = CurrentScreen::ProjectList;
                    }
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    _ => {}
                },
                CurrentScreen::ProjectList => match key.code {
                    KeyCode::Enter => {
                        if let Some(repo) = app.repositories.get_selected_repository() {
                            app.current_repository = Some(repo.clone());
                            app.current_screen = CurrentScreen::Project;
                            app.scrollbar = DependabotScrollbar::new(repo.total_active_alerts * 10);

                            trace_dbg!(level: tracing::Level::INFO, app.scrollbar.get_length());
                        }
                    }
                    KeyCode::Up => {
                        app.repositories.previous();
                    },
                    KeyCode::Down => {
                        app.repositories.next();
                    }
                    KeyCode::Char('o') => {
                        app.current_screen = CurrentScreen::Overview;
                    }
                    KeyCode::Char('u') => {
                        app.current_screen = CurrentScreen::Update;
                    }
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    _ => {}
                },
                CurrentScreen::Project => match key.code {
                    KeyCode::Char('r') => {
                        app.current_screen = CurrentScreen::ProjectList;
                    }
                    KeyCode::Tab => {
                        app.current_screen = CurrentScreen::DependabotDetails;
                    }
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    _ => {}
                },
                CurrentScreen::DependabotDetails => match key.code {
                    KeyCode::Up => {
                        app.scrollbar.scroll_up();
                    }
                    KeyCode::Down => {
                        app.scrollbar.scroll_down();
                    }
                    KeyCode::Tab => {
                        app.current_screen = CurrentScreen::Project;
                    }
                    KeyCode::Char('o') => {
                        app.current_screen = CurrentScreen::Overview;
                    }
                    KeyCode::Char('t') => {
                        app.scrollbar.top();
                    }
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        while let Some(rx) = &app.fetching {
            match rx.try_recv() {
                Ok(result) => {
                    app.repositories = result?;
                    app.fetching = None;
                    app.current_screen = CurrentScreen::Overview;
                }
                Err(TryRecvError::Empty) => {
                    // The fetch is still in progress, update the UI as usual
                    app.on_tick();
                    terminal.draw(|f| ui::ui(f, app))
                        .map_err(|e| Box::new(e) as DependabotTrackerError)?;
                }
                Err(TryRecvError::Disconnected) => {
                    // The fetch thread has panicked or been unexpectedly terminated
                    return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Fetch thread terminated unexpectedly")));
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    }
}

