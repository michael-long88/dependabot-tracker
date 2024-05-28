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
        let index = match self.state.selected() {
            Some(index) => {
                if index >= self.repos.len() - 1 {
                    0
                } else {
                    index + 1
                }
            }
            None => self.selected.unwrap_or(0),
        };
        self.state.select(Some(index));
    }

    pub fn previous(&mut self) {
        let index = match self.state.selected() {
            Some(index) => {
                if index == 0 {
                    self.repos.len() - 1
                } else {
                    index - 1
                }
            }
            None => self.selected.unwrap_or(0),
        };
        self.state.select(Some(index));
    }

    pub fn get_selected_repository(&self) -> Option<&Repository> {
        self.state.selected().map(|index| &self.repos[index])
    }

    pub fn get_mut_state(&mut self) -> &mut ListState {
        &mut self.state
    }
}
