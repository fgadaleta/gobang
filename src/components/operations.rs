use super::{Component, EventState, StatefulDrawableComponent};
use crate::clipboard::copy_to_clipboard;
use crate::components::command::{self, CommandInfo};
use crate::components::TableComponent;
use crate::config::KeyConfig;
use crate::database::Pool;
use crate::event::Key;
use anyhow::Result;
use async_trait::async_trait;
use database_tree::{Database, Table};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

#[derive(Debug, PartialEq)]
pub enum Focus {
    NewUser,
    DelUser,
    NewGraph,
    DelGraph,
    // TODO add others
}

impl std::fmt::Display for Focus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct OperationsComponent {

    new_user: TableComponent,
    del_user: TableComponent,
    new_graph: TableComponent,
    del_graph: TableComponent,
    // TODO add other operations
    focus: Focus,
    key_config: KeyConfig,
}

impl OperationsComponent {
    pub fn new(key_config: KeyConfig) -> Self {
        Self {
            new_user: TableComponent::new(key_config.clone()),
            del_user: TableComponent::new(key_config.clone()),
            new_graph: TableComponent::new(key_config.clone()),
            del_graph: TableComponent::new(key_config.clone()),
            focus: Focus::NewUser,
            key_config,
        }
    }

    fn focused_component(&mut self) -> &mut TableComponent {
        match self.focus {
            Focus::NewUser => &mut self.new_user,
            Focus::DelUser => &mut self.del_user,
            Focus::NewGraph => &mut self.new_graph,
            Focus::DelGraph => &mut self.del_graph,
        }
    }

    pub async fn update(
        &mut self,
        database: Database,
        table: Table,
        pool: &Box<dyn Pool>,
    ) -> Result<()> {
        self.new_user.reset();
        let columns = pool.get_columns(&database, &table).await?;
        if !columns.is_empty() {

            self.new_user.update(
                columns
                    .iter()
                    .map(|c| c.columns())
                    .collect::<Vec<Vec<String>>>(),
                columns.get(0).unwrap().fields(),
                database.clone(),
                table.clone(),
            );
        }
        self.del_user.reset();
        let constraints = pool.get_constraints(&database, &table).await?;
        if !constraints.is_empty() {
            self.del_user.update(
                constraints
                    .iter()
                    .map(|c| c.columns())
                    .collect::<Vec<Vec<String>>>(),
                constraints.get(0).unwrap().fields(),
                database.clone(),
                table.clone(),
            );
        }
        self.new_graph.reset();
        let foreign_keys = pool.get_foreign_keys(&database, &table).await?;
        if !foreign_keys.is_empty() {
            self.new_graph.update(
                foreign_keys
                    .iter()
                    .map(|c| c.columns())
                    .collect::<Vec<Vec<String>>>(),
                foreign_keys.get(0).unwrap().fields(),
                database.clone(),
                table.clone(),
            );
        }
        self.del_graph.reset();
        let indexes = pool.get_indexes(&database, &table).await?;
        if !indexes.is_empty() {
            self.del_graph.update(
                indexes
                    .iter()
                    .map(|c| c.columns())
                    .collect::<Vec<Vec<String>>>(),
                indexes.get(0).unwrap().fields(),
                database.clone(),
                table.clone(),
            );
        }
        Ok(())
    }

    fn tab_names(&self) -> Vec<(Focus, String)> {
        vec![
            (Focus::NewUser, command::tab_new_user(&self.key_config).name),
            (
                Focus::DelUser,
                command::tab_del_user(&self.key_config).name,
            ),
            (
                Focus::NewGraph,
                command::tab_new_graph(&self.key_config).name,
            ),
            (Focus::DelGraph, command::tab_del_graph(&self.key_config).name),
        ]
    }
}

impl StatefulDrawableComponent for OperationsComponent {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect, focused: bool) -> Result<()> {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Length(20), Constraint::Min(1)])
            .split(area);

        let tab_names = self
            .tab_names()
            .iter()
            .map(|(f, c)| {
                ListItem::new(c.to_string()).style(if *f == self.focus {
                    Style::default().bg(Color::Blue)
                } else {
                    Style::default()
                })
            })
            .collect::<Vec<ListItem>>();

        let tab_list = List::new(tab_names)
            .block(Block::default().borders(Borders::ALL).style(if focused {
                Style::default()
            } else {
                Style::default().fg(Color::DarkGray)
            }))
            .style(Style::default());

        f.render_widget(tab_list, layout[0]);

        self.focused_component().draw(f, layout[1], focused)?;
        Ok(())
    }
}

#[async_trait]
impl Component for OperationsComponent {
    fn commands(&self, out: &mut Vec<CommandInfo>) {
        out.push(CommandInfo::new(command::toggle_property_tabs(
            &self.key_config,
        )));
    }

    fn event(&mut self, key: Key) -> Result<EventState> {
        self.focused_component().event(key)?;

        if key == self.key_config.copy {
            if let Some(text) = self.focused_component().selected_cells() {
                copy_to_clipboard(text.as_str())?
            }
        } else if key == self.key_config.tab_columns {
            self.focus = Focus::NewUser;
        } else if key == self.key_config.tab_constraints {
            self.focus = Focus::DelUser;
        } else if key == self.key_config.tab_foreign_keys {
            self.focus = Focus::NewGraph;
        } else if key == self.key_config.tab_indexes {
            self.focus = Focus::DelGraph;
        }
        Ok(EventState::NotConsumed)
    }
}
