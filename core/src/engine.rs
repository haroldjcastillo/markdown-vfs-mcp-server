use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A paginated collection of section paths returned by discovery tools.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PaginatedResult {
    /// List of hierarchical paths (e.g., ["chapter-1", "chapter-1/intro"]).
    pub items: Vec<String>,
    /// Current 0-indexed page number.
    pub page: usize,
    /// Number of items requested per page.
    pub size: usize,
    /// Total number of items available for the given parent path.
    pub total: usize,
}

/// Represents a specific section of the Markdown document identified by a heading.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MarkdownNode {
    /// The original text of the heading.
    pub title: String,
    /// The URL-safe version of the title used for path building.
    pub slug: String,
    /// The depth level of the heading (1 for #, 2 for ##, etc.).
    pub level: usize,
    /// The raw text content belonging directly to this section (before the next heading).
    pub content: String,
    /// Hierarchical paths to direct sub-sections.
    pub children: Vec<String>,
}

/// The core indexing engine that transforms Markdown text into a Virtual File System.
/// It uses a BTreeMap to maintain an ordered, searchable index of all sections.
pub struct MarkdownEngine {
    /// Map of full hierarchical paths to their corresponding document nodes.
    pub(crate) nodes: BTreeMap<String, MarkdownNode>,
    /// Map of Markdown reference IDs (e.g., [link_id]: url) to their target URLs.
    pub(crate) references: BTreeMap<String, String>,
    /// List of paths for top-level headings (Level 1 or the first encountered).
    pub(crate) root_nodes: Vec<String>,
}

/// Generates a URL-safe slug from a heading title.
pub fn slugify(title: &str) -> String {
    let mut out = String::new();
    let mut previous_dash = false;

    for ch in title.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            previous_dash = false;
        } else if !previous_dash {
            out.push('-');
            previous_dash = true;
        }
    }

    let normalized = out.trim_matches('-').to_string();
    if normalized.is_empty() {
        "section".to_string()
    } else {
        normalized
    }
}

/// Attempts to parse a Markdown reference definition (e.g., [id]: url).
fn parse_reference(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();
    if !trimmed.starts_with('[') {
        return None;
    }

    let close = trimmed.find("]:")?;
    if close <= 1 {
        return None;
    }

    let ref_id = trimmed[1..close].trim();
    let url = trimmed[(close + 2)..].trim();

    if ref_id.is_empty() || url.is_empty() {
        return None;
    }

    Some((ref_id.to_string(), url.to_string()))
}

/// Parses a standard Markdown heading (e.g., ### My Title).
fn parse_markdown_heading(line: &str) -> Option<(usize, String)> {
    let trimmed = line.trim();
    if !trimmed.starts_with('#') {
        return None;
    }

    let mut level = 0usize;
    for ch in trimmed.chars() {
        if ch == '#' {
            level += 1;
        } else {
            break;
        }
    }

    if !(1..=6).contains(&level) {
        return None;
    }

    if trimmed
        .chars()
        .nth(level)
        .is_none_or(|c| !c.is_whitespace())
    {
        return None;
    }

    let title = trimmed[level..].trim();
    if title.is_empty() {
        return None;
    }

    Some((level, title.to_string()))
}

/// Parses an HTML-style heading (e.g., <h2>My Title</h2>).
fn parse_html_heading(line: &str) -> Option<(usize, String)> {
    let trimmed = line.trim();
    let lower = trimmed.to_ascii_lowercase();

    if !lower.starts_with("<h") {
        return None;
    }

    let bytes = lower.as_bytes();
    if bytes.len() < 5 {
        return None;
    }

    let level_char = bytes[2] as char;
    if !('1'..='6').contains(&level_char) {
        return None;
    }

    let level = level_char.to_digit(10)? as usize;
    let open_end = trimmed.find('>')?;
    let close_tag = format!("</h{}>", level);
    let close_index = lower.rfind(&close_tag)?;

    if close_index <= open_end {
        return None;
    }

    let title = trimmed[(open_end + 1)..close_index].trim();
    if title.is_empty() {
        return None;
    }

    Some((level, title.to_string()))
}

/// Unified heading parser that supports both Markdown and HTML heading formats.
pub(crate) fn parse_heading(line: &str) -> Option<(usize, String)> {
    parse_markdown_heading(line).or_else(|| parse_html_heading(line))
}

impl MarkdownEngine {
    /// Parses the provided Markdown content and builds a hierarchical Virtual File System index.
    pub fn new(content: &str) -> MarkdownEngine {
        let mut engine = MarkdownEngine {
            nodes: BTreeMap::new(),
            references: BTreeMap::new(),
            root_nodes: Vec::new(),
        };

        let mut stack: Vec<(usize, String)> = Vec::new();
        let mut current_path: Option<String> = None;

        for line in content.lines() {
            if let Some((ref_id, url)) = parse_reference(line) {
                engine.references.insert(ref_id, url);
                continue;
            }

            if let Some((level, title)) = parse_heading(line) {
                while let Some((last_level, _)) = stack.last() {
                    if *last_level >= level {
                        stack.pop();
                    } else {
                        break;
                    }
                }

                let parent_path = stack.last().map(|(_, path)| path.clone());
                let base_slug = slugify(&title);
                let unique_slug = engine.make_unique_slug(parent_path.as_deref(), &base_slug);

                let full_path = match &parent_path {
                    Some(parent) => format!("{}/{}", parent, unique_slug),
                    None => unique_slug.clone(),
                };

                let node = MarkdownNode {
                    title,
                    slug: unique_slug,
                    level,
                    content: String::new(),
                    children: Vec::new(),
                };

                if let Some(parent) = parent_path {
                    if let Some(parent_node) = engine.nodes.get_mut(&parent) {
                        parent_node.children.push(full_path.clone());
                    }
                } else {
                    engine.root_nodes.push(full_path.clone());
                }

                engine.nodes.insert(full_path.clone(), node);
                stack.push((level, full_path.clone()));
                current_path = Some(full_path);
                continue;
            }

            engine.push_content_line(current_path.as_ref(), line);
        }

        engine
    }

    /// Generates a unique slug for a section by checking for collisions with existing paths.
    fn make_unique_slug(&self, parent_path: Option<&str>, base_slug: &str) -> String {
        let mut candidate = base_slug.to_string();
        let mut suffix = 1usize;

        loop {
            let full_path = match parent_path {
                Some(parent) => format!("{}/{}", parent, candidate),
                None => candidate.clone(),
            };

            if !self.nodes.contains_key(&full_path) {
                return candidate;
            }

            candidate = format!("{}-{}", base_slug, suffix);
            suffix += 1;
        }
    }

    /// Internal helper to append content to the currently active section during parsing.
    fn push_content_line(&mut self, current_path: Option<&String>, line: &str) {
        if let Some(path) = current_path {
            if let Some(node) = self.nodes.get_mut(path) {
                if !node.content.is_empty() {
                    node.content.push('\n');
                }
                node.content.push_str(line);
            }
        }
    }

    /// Lists direct sub-sections for a given hierarchical path with pagination support.
    /// Use an empty path ("") to retrieve root-level headings.
    pub fn ls(&self, path: &str, page: usize, size: usize) -> PaginatedResult {
        let entries = if path.is_empty() {
            self.root_nodes.clone()
        } else {
            self.nodes
                .get(path)
                .map(|node| node.children.clone())
                .unwrap_or_default()
        };

        let safe_size = if size == 0 { 20 } else { size };
        let start = page.saturating_mul(safe_size);
        let end = start.saturating_add(safe_size).min(entries.len());

        let items = if start >= entries.len() {
            Vec::new()
        } else {
            entries[start..end].to_vec()
        };

        PaginatedResult {
            items,
            page,
            size: safe_size,
            total: entries.len(),
        }
    }

    /// Retrieves the human-readable title of the section at the specified path.
    pub fn get_title(&self, path: &str) -> Option<String> {
        self.nodes.get(path).map(|n| n.title.clone())
    }

    /// Reads the direct text content of a section (non-recursive).
    pub fn read(&self, path: &str) -> Option<String> {
        self.nodes.get(path).map(|node| node.content.clone())
    }

    /// Reads the full content of a section, including all its sub-sections recursively.
    pub fn read_full(&self, path: &str) -> Option<String> {
        if !self.nodes.contains_key(path) {
            return None;
        }
        let mut buf = String::new();
        self.collect_full(path, &mut buf);
        Some(buf)
    }

    /// Recursive helper to collect content and headings from a node and its entire subtree.
    fn collect_full(&self, path: &str, buf: &mut String) {
        let Some(node) = self.nodes.get(path) else {
            return;
        };
        let content = node.content.clone();
        let children = node.children.clone();

        if !content.is_empty() {
            if !buf.is_empty() {
                buf.push('\n');
            }
            buf.push_str(&content);
        }

        for child_path in children {
            let (child_level, child_title) = match self.nodes.get(&child_path) {
                Some(n) => (n.level, n.title.clone()),
                None => continue,
            };
            if !buf.is_empty() {
                buf.push('\n');
            }
            buf.push_str(&"#".repeat(child_level));
            buf.push(' ');
            buf.push_str(&child_title);
            self.collect_full(&child_path, buf);
        }
    }

    /// Resolves a Markdown reference ID to its corresponding URL.
    pub fn get_reference(&self, ref_id: &str) -> Option<String> {
        self.references.get(ref_id).cloned()
    }

    /// Performs a case-insensitive search across all section titles.
    /// Returns a list of all matching hierarchical paths.
    pub fn search(&self, query: &str) -> Vec<String> {
        let q = query.to_ascii_lowercase();
        self.nodes
            .iter()
            .filter_map(|(path, node)| {
                if node.title.to_ascii_lowercase().contains(&q) {
                    Some(path.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}
