use std::{cell::RefCell, collections::HashSet, rc::Rc};

use gpui::SharedString;
use gpui_component::{list::ListState, IndexPath};

use crate::task_schema::AgentTask;

pub struct TaskListDelegate {
    pub industries: Vec<SharedString>,
    pub _agent_tasks: Vec<Rc<AgentTask>>,
    pub matched_agent_tasks: Vec<Vec<Rc<AgentTask>>>,
    pub selected_index: Option<IndexPath>,
    pub confirmed_index: Option<IndexPath>,
    pub query: SharedString,
    pub loading: bool,
    pub eof: bool,
    pub lazy_load: bool,
    // Track which sections are collapsed (using RefCell for interior mutability)
    pub collapsed_sections: Rc<RefCell<HashSet<usize>>>,
    // Store weak reference to list state to notify on collapse toggle
    pub list_state: Option<gpui::WeakEntity<ListState<TaskListDelegate>>>,
}

impl TaskListDelegate {
    pub fn new() -> Self {
        Self {
            industries: vec![],
            matched_agent_tasks: vec![],
            _agent_tasks: vec![],
            selected_index: Some(IndexPath::default()),
            confirmed_index: None,
            query: "".into(),
            loading: false,
            eof: false,
            lazy_load: false,
            collapsed_sections: Rc::new(RefCell::new(HashSet::new())),
            list_state: None,
        }
    }

    pub fn is_section_collapsed(&self, section: usize) -> bool {
        self.collapsed_sections.borrow().contains(&section)
    }

    pub fn prepare(&mut self, query: impl Into<SharedString>) {
        self.query = query.into();
        // Clear previous data before rebuilding
        self.industries.clear();
        self.matched_agent_tasks.clear();

        let agent_tasks: Vec<Rc<AgentTask>> = self
            ._agent_tasks
            .iter()
            .filter(|agent_task| {
                agent_task
                    .name
                    .to_lowercase()
                    .contains(&self.query.to_lowercase())
            })
            .cloned()
            .collect();
        for agent_task in agent_tasks.into_iter() {
            if let Some(ix) = self
                .industries
                .iter()
                .position(|s| s.as_ref() == agent_task.task_type.as_str())
            {
                self.matched_agent_tasks[ix].push(agent_task);
            } else {
                self.industries.push(agent_task.task_type.clone().into());
                self.matched_agent_tasks.push(vec![agent_task]);
            }
        }
    }

    pub fn load_all_tasks(&mut self) {
        let tasks = crate::task_data::load_mock_tasks();
        self._agent_tasks = tasks.into_iter().map(Rc::new).collect();
        self.prepare(self.query.clone());
    }

    pub fn extend_more(&mut self, _len: usize) {
        // For mock data, we just use the initial JSON load
        // If we want to support pagination/lazy loading, we could cycle through tasks
        // For now, just do nothing as all tasks are loaded initially
    }

    pub fn selected_agent_task(&self) -> Option<Rc<AgentTask>> {
        let Some(ix) = self.selected_index else {
            return None;
        };

        self.matched_agent_tasks
            .get(ix.section)
            .and_then(|c| c.get(ix.row))
            .cloned()
    }
}
