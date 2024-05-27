use std::error::Error;
use std::sync::mpsc::Receiver;

use color_eyre::eyre::Result;
use ratatui::widgets::ScrollbarState;
use throbber_widgets_tui::ThrobberState;

use crate::current_screen::CurrentScreen;
use crate::repository::Repository;
use crate::repository_list::RepositoryList;
use crate::trace_dbg;

pub type DependabotTrackerError = Box<dyn Error + Send + 'static>;

pub struct App {
    // the currently repository being viewed
    pub current_repository: Option<Repository>,
    // the last time that the dependabots file was updated
    pub last_updated: String,
    // the list of all repositories
    pub repositories: RepositoryList,
    // the current screen the user is looking at, and will later determine what is rendered
    pub current_screen: CurrentScreen,
    // the github api token
    pub token: String,
    // the github username
    pub username: String,
    // the state of the spinning widget
    pub spinner_state: ThrobberState,
    // the channel to receive the result of the fetching thread
    pub fetching: Option<Receiver<Result<RepositoryList, DependabotTrackerError>>>,
    // the scrollbar for viewing a repository's dependabots
    pub scrollbar: DependabotScrollbar,
    // the height of the current window chunk
    pub chunk_height: u16,
    // the last error that occurred
    pub error: Option<String>,
}

impl App {
    pub fn new() -> App {
        let loaded_repositories = load_repositories_from_file();
        let repositories = loaded_repositories.unwrap_or_else(|_| {
            trace_dbg!(level: tracing::Level::ERROR, "Failed to load repositories from file");
            vec![]
        });
        App {
            current_repository: None,
            last_updated: String::new(),
            repositories: RepositoryList::with_respositories(repositories),
            current_screen: CurrentScreen::default(),
            token: std::env::var("PAT").expect("PAT not set"),
            username: std::env::var("GH_USERNAME").expect("GH_USERNAME not set"),
            spinner_state: ThrobberState::default(),
            fetching: None,
            scrollbar: DependabotScrollbar::default(),
            chunk_height: 0,
            error: None,
        }
    }

    pub fn on_tick(&mut self) {
        self.spinner_state.calc_next();
    }
}

pub fn load_repositories_from_file() -> Result<Vec<Repository>, Box<dyn Error>> {
    let file = std::fs::File::open("data/repositories.json")?;
    let reader = std::io::BufReader::new(file);
    let repositories = serde_json::from_reader(reader)?;

    Ok(repositories)
}

pub struct DependabotScrollbar {
    state: ScrollbarState,
    length: usize,
    pub position: usize,
}

impl DependabotScrollbar {
    pub fn default() -> Self {
        DependabotScrollbar {
            state: ScrollbarState::default(),
            length: 0,
            position: 0,
        }
    }

    pub fn new(length: usize) -> Self {
        DependabotScrollbar {
            state: ScrollbarState::default()
                .content_length(length)
                .viewport_content_length(1)
                .position(0),
            length,
            position: 0,
        }
    }

    pub fn scroll_down(&mut self) {
        if self.position < self.length {
            self.position += 1;
        } else {
            self.position = 0;
        }

        self.state = self.state.position(self.position);
    }

    pub fn scroll_up(&mut self) {
        if self.position > 0 {
            self.position -= 1;
        } else {
            self.position = self.length;
        }

        self.state = self.state.position(self.position);
    }

    pub fn top(&mut self) {
        self.position = 0;
        self.state = self.state.position(self.position);
    }

    pub fn get_mut_state(&mut self) -> &mut ScrollbarState {
        &mut self.state
    }

    pub fn resize(&mut self, length: usize) {
        self.length = length;
        self.position = 0;
        self.state = self.state.content_length(length).position(0);
    }

    pub fn get_length(&self) -> usize {
        self.length
    }
}
