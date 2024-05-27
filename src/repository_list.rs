use ratatui::widgets::ListState;

use crate::repository::Repository;

pub struct RepositoryList {
    state: ListState,
    pub repos: Vec<Repository>,
    selected: Option<usize>,
}

impl RepositoryList {
    pub fn with_respositories(repos: Vec<Repository>) -> RepositoryList {
        let mut state = ListState::default();
        if repos.is_empty() {
            state.select(None);
        } else {
            state.select(Some(0));
        }
        RepositoryList {
            state,
            repos,
            selected: None,
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.repos.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => self.selected.unwrap_or(0),
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.repos.len() - 1
                } else {
                    i - 1
                }
            }
            None => self.selected.unwrap_or(0),
        };
        self.state.select(Some(i));
    }

    pub fn get_selected_repository(&self) -> Option<&Repository> {
        self.state.selected().map(|i| &self.repos[i])
    }

    pub fn get_mut_state(&mut self) -> &mut ListState {
        &mut self.state
    }
}
