use std::fmt::{self, Display, Formatter};

use ratatui::{style::{Color, Style}, text::{Line, Span}};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependabotState {
    AutoDismissed,
    Dismissed,
    Fixed,
    Open,
}

impl Display for DependabotState {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DependabotSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl Display for DependabotSeverity {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct GithubDependabot {
    pub number: u32,
    pub state: DependabotState,
    pub security_vulnerability: SecurityVulnerability,
    pub html_url: String,
    pub created_at: String,
    pub updated_at: String,
    pub dismissed_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SecurityVulnerability {
    pub severity: DependabotSeverity,
    pub package: Package,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Package {
    pub ecosystem: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependabot {
    pub number: u32,
    pub state: DependabotState,
    pub severity: DependabotSeverity,
    pub html_url: String,
    pub created_at: String,
    pub updated_at: String,
    pub dismissed_at: Option<String>,
    pub dependency_ecosystem: String,
    pub dependency_name: String,
}

impl Dependabot {
    pub fn to_text(&self) -> Vec<Line> {
        let mut lines = Vec::<Line>::new();
        lines.push(Line::from(vec![
            Span::styled("-".repeat(20), Style::default().fg(Color::Green))
        ]));
        lines.push(Line::from(vec![
            Span::styled(format!("Number: {}", self.number), Style::default().fg(Color::Blue))
        ]));
        lines.push(Line::from(vec![
            Span::styled(format!("State: {}", self.state), Style::default().fg(Color::Blue))
        ]));
        lines.push(Line::from(vec![
            Span::styled(format!("Severity: {}", self.severity), Style::default().fg(Color::Blue))
        ]));
        lines.push(Line::from(vec![
            Span::styled(format!("URL: {}", self.html_url), Style::default().fg(Color::Blue))
        ]));
        lines.push(Line::from(vec![
            Span::styled(format!("Created At: {}", self.created_at), Style::default().fg(Color::Blue))
        ]));
        lines.push(Line::from(vec![
            Span::styled(format!("Updated At: {}", self.updated_at), Style::default().fg(Color::Blue))
        ]));
        lines.push(Line::from(vec![
            Span::styled(format!("Dismissed At: {}", self.dismissed_at.clone().unwrap_or_else(|| "N/A".to_string())), Style::default().fg(Color::Blue))
        ]));
        lines.push(Line::from(vec![
            Span::styled(format!("Dependency Ecosystem: {}", self.dependency_ecosystem), Style::default().fg(Color::Blue))
        ]));
        lines.push(Line::from(vec![
            Span::styled(format!("Dependency Name: {}", self.dependency_name), Style::default().fg(Color::Blue))
        ]));

        lines
    }
}
