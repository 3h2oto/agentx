use gpui::{
    App, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div, prelude::*,
    px,
};
use gpui_component::{
    ActiveTheme, Icon, IconName, Sizable,
    button::{Button, ButtonVariants},
    h_flex, v_flex,
};
use similar::{ChangeTag, TextDiff};
use std::collections::HashMap;
use std::path::PathBuf;

use agent_client_protocol::{ToolCall, ToolCallContent};

/// Statistics for a single file's changes
#[derive(Debug, Clone, Default)]
pub struct FileChangeStats {
    pub path: PathBuf,
    pub additions: usize,
    pub deletions: usize,
    pub is_new_file: bool,
}

impl FileChangeStats {
    /// Calculate statistics from old and new text
    pub fn from_diff(path: PathBuf, old_text: Option<&str>, new_text: &str) -> Self {
        match old_text {
            Some(old) => {
                let diff = TextDiff::from_lines(old, new_text);
                let mut additions = 0;
                let mut deletions = 0;

                for change in diff.iter_all_changes() {
                    match change.tag() {
                        ChangeTag::Insert => additions += 1,
                        ChangeTag::Delete => deletions += 1,
                        ChangeTag::Equal => {}
                    }
                }

                Self {
                    path,
                    additions,
                    deletions,
                    is_new_file: false,
                }
            }
            None => {
                // New file - all lines are additions
                Self {
                    path,
                    additions: new_text.lines().count(),
                    deletions: 0,
                    is_new_file: true,
                }
            }
        }
    }

    /// Get total number of changed lines
    pub fn total_changes(&self) -> usize {
        self.additions + self.deletions
    }
}

/// Summary of all file changes in a session
#[derive(Debug, Clone, Default)]
pub struct DiffSummaryData {
    /// Map of file path to change statistics
    pub files: HashMap<PathBuf, FileChangeStats>,
}

impl DiffSummaryData {
    /// Create a new empty summary
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
        }
    }

    /// Extract diff statistics from a list of tool calls
    pub fn from_tool_calls(tool_calls: &[ToolCall]) -> Self {
        let mut summary = Self::new();

        for tool_call in tool_calls {
            for content in &tool_call.content {
                if let ToolCallContent::Diff(diff) = content {
                    let stats = FileChangeStats::from_diff(
                        diff.path.clone(),
                        diff.old_text.as_deref(),
                        &diff.new_text,
                    );
                    summary.files.insert(diff.path.clone(), stats);
                }
            }
        }

        summary
    }

    /// Get total number of files changed
    pub fn total_files(&self) -> usize {
        self.files.len()
    }

    /// Get total additions across all files
    pub fn total_additions(&self) -> usize {
        self.files.values().map(|f| f.additions).sum()
    }

    /// Get total deletions across all files
    pub fn total_deletions(&self) -> usize {
        self.files.values().map(|f| f.deletions).sum()
    }

    /// Get files sorted by total changes (descending)
    pub fn sorted_files(&self) -> Vec<&FileChangeStats> {
        let mut files: Vec<_> = self.files.values().collect();
        files.sort_by(|a, b| b.total_changes().cmp(&a.total_changes()));
        files
    }

    /// Check if there are any changes
    pub fn has_changes(&self) -> bool {
        !self.files.is_empty()
    }
}

/// UI component to display diff summary
pub struct DiffSummary {
    data: DiffSummaryData,
    collapsed: bool,
}

impl DiffSummary {
    pub fn new(data: DiffSummaryData) -> Self {
        Self {
            data,
            collapsed: false,
        }
    }

    /// Toggle collapsed state
    pub fn toggle(&mut self, cx: &mut Context<Self>) {
        self.collapsed = !self.collapsed;
        cx.notify();
    }

    /// Update the summary data
    pub fn update_data(&mut self, data: DiffSummaryData, cx: &mut Context<Self>) {
        self.data = data;
        cx.notify();
    }

    /// Render a single file change row
    fn render_file_row(
        &self,
        stats: &FileChangeStats,
        cx: &Context<Self>,
    ) -> gpui::AnyElement {
        let filename = stats
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown")
            .to_string();

        let file_ext = stats
            .path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        // Choose icon based on file type
        let icon = match file_ext {
            "rs" => IconName::File,
            "js" | "jsx" | "ts" | "tsx" => IconName::File,
            "py" => IconName::File,
            "html" | "htm" => IconName::File,
            "css" | "scss" => IconName::File,
            "json" | "yaml" | "yml" | "toml" => IconName::File,
            "md" | "txt" => IconName::File,
            _ => IconName::File,
        };

        h_flex()
            .w_full()
            .items_center()
            .gap_3()
            .p_2()
            .rounded(px(4.))
            .hover(|this| this.bg(cx.theme().muted.opacity(0.3)))
            .cursor_pointer()
            .child(
                Icon::new(icon)
                    .size(px(16.))
                    .text_color(cx.theme().muted_foreground),
            )
            .child(
                div()
                    .flex_1()
                    .text_size(px(13.))
                    .text_color(cx.theme().foreground)
                    .child(filename),
            )
            .when(stats.is_new_file, |this| {
                this.child(
                    div()
                        .px_2()
                        .py(px(2.))
                        .rounded(px(4.))
                        .bg(cx.theme().green.opacity(0.2))
                        .text_size(px(11.))
                        .text_color(cx.theme().green)
                        .child("NEW"),
                )
            })
            .when(stats.additions > 0, |this| {
                this.child(
                    div()
                        .text_size(px(12.))
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(cx.theme().green)
                        .child(format!("+{}", stats.additions)),
                )
            })
            .when(stats.deletions > 0, |this| {
                this.child(
                    div()
                        .text_size(px(12.))
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(cx.theme().red)
                        .child(format!("-{}", stats.deletions)),
                )
            })
            .child(
                Icon::new(IconName::ChevronRight)
                    .size(px(14.))
                    .text_color(cx.theme().muted_foreground),
            )
            .into_any_element()
    }
}

impl Render for DiffSummary {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.data.has_changes() {
            return div().into_any_element();
        }

        let total_files = self.data.total_files();
        let total_additions = self.data.total_additions();
        let total_deletions = self.data.total_deletions();
        let is_collapsed = self.collapsed;

        v_flex()
            .w_full()
            .gap_2()
            .p_4()
            .rounded(cx.theme().radius)
            .bg(cx.theme().secondary)
            .border_1()
            .border_color(cx.theme().border)
            // Header
            .child(
                h_flex()
                    .w_full()
                    .items_center()
                    .gap_3()
                    .child(
                        Icon::new(IconName::Asterisk)
                            .size(px(16.))
                            .text_color(cx.theme().accent),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_size(px(14.))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(cx.theme().foreground)
                            .child(format!(
                                "{} file{} changed",
                                total_files,
                                if total_files == 1 { "" } else { "s" }
                            )),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .when(total_additions > 0, |this| {
                                this.child(
                                    div()
                                        .text_size(px(12.))
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .text_color(cx.theme().green)
                                        .child(format!("+{}", total_additions)),
                                )
                            })
                            .when(total_deletions > 0, |this| {
                                this.child(
                                    div()
                                        .text_size(px(12.))
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .text_color(cx.theme().red)
                                        .child(format!("-{}", total_deletions)),
                                )
                            }),
                    )
                    .child(
                        Button::new("diff-summary-toggle")
                            .icon(if is_collapsed {
                                IconName::ChevronDown
                            } else {
                                IconName::ChevronUp
                            })
                            .ghost()
                            .small()
                            .on_click(cx.listener(|this, _ev, _window, cx| {
                                this.toggle(cx);
                            })),
                    ),
            )
            // File list (only shown when not collapsed)
            .when(!is_collapsed, |this| {
                this.child(
                    v_flex()
                        .w_full()
                        .gap_1()
                        .children(
                            self.data
                                .sorted_files()
                                .into_iter()
                                .map(|stats| self.render_file_row(stats, cx)),
                        ),
                )
            })
            .into_any_element()
    }
}

/// A wrapper view for DiffSummary that can be used as a GPUI entity
pub struct DiffSummaryView {
    summary: Entity<DiffSummary>,
}

impl DiffSummaryView {
    pub fn new(data: DiffSummaryData, _window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| {
            let summary = cx.new(|_| DiffSummary::new(data));
            Self { summary }
        })
    }

    /// Update the summary data
    pub fn update_data(&mut self, data: DiffSummaryData, cx: &mut Context<Self>) {
        self.summary.update(cx, |summary, cx| {
            summary.update_data(data, cx);
        });
        cx.notify();
    }
}

impl Render for DiffSummaryView {
    fn render(&mut self, _: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.summary.clone()
    }
}
