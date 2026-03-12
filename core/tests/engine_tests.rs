use markdown_btree_core::engine::MarkdownEngine;

/// Verifies that top-level headings are correctly identified as root nodes in the VFS.
#[test]
fn test_root_nodes_parsed() {
    let content = "# Chapter 1\n\nSome content.\n\n# Chapter 2\n\nMore content.";
    let engine = MarkdownEngine::new(content);
    let result = engine.ls("", 0, 20);
    assert_eq!(result.total, 2);
    assert_eq!(result.items[0], "chapter-1");
    assert_eq!(result.items[1], "chapter-2");
}

/// Ensures that sub-headings (##) are correctly associated with their parent headings (#).
#[test]
fn test_nested_nodes() {
    let content = "# Chapter 1\n\n## Section 1\n\nContent.\n\n## Section 2\n\nMore.";
    let engine = MarkdownEngine::new(content);
    let result = engine.ls("chapter-1", 0, 20);
    assert_eq!(result.total, 2);
    assert!(result.items[0].ends_with("section-1"));
    assert!(result.items[1].ends_with("section-2"));
}

/// Tests that identical titles result in unique hierarchical paths using a numeric suffix.
#[test]
fn test_unique_slug_dedup() {
    let content = "# Chapter\n\n## Section\n\n## Section\n\n## Section";
    let engine = MarkdownEngine::new(content);
    let result = engine.ls("chapter", 0, 20);
    assert_eq!(result.total, 3);
    assert!(result.items[0].ends_with("section"));
    assert!(result.items[1].ends_with("section-1"));
    assert!(result.items[2].ends_with("section-2"));
}

/// Checks that section content is correctly isolated from its sub-sections.
#[test]
fn test_read_content() {
    let content = "# Chapter 1\n\nHello world.";
    let engine = MarkdownEngine::new(content);
    assert_eq!(engine.read("chapter-1"), Some("Hello world.".to_string()));
}

/// Verifies that Markdown link/image references (e.g., [id]: url) are parsed into the index
/// and omitted from the actual text content of a section.
#[test]
fn test_references_excluded_from_content() {
    let content = "# Chapter\n\n[img]: https://example.com/img.png\n\nSome text.";
    let engine = MarkdownEngine::new(content);
    let node_content = engine.read("chapter").unwrap_or_default();
    assert!(!node_content.contains("[img]"));
    assert_eq!(
        engine.get_reference("img"),
        Some("https://example.com/img.png".to_string())
    );
}

/// Tests the case-insensitive search capability across section titles.
#[test]
fn test_search_by_title() {
    let content = "# Introduction\n\n## Advanced Topics\n\n### Basic Concepts";
    let engine = MarkdownEngine::new(content);
    let results = engine.search("advanced");
    assert_eq!(results.len(), 1);
    assert!(results[0].ends_with("advanced-topics"));
}

/// Validates the pagination logic in the `ls` method, ensuring it correctly calculates
/// offsets and item counts for large documents.
#[test]
fn test_pagination() {
    let headings: String = (1..=10).map(|i| format!("# Chapter {i}\n\n")).collect();
    let engine = MarkdownEngine::new(&headings);
    let page0 = engine.ls("", 0, 3);
    assert_eq!(page0.items.len(), 3);
    assert_eq!(page0.page, 0);
    assert_eq!(page0.size, 3);
    assert_eq!(page0.total, 10);

    let page3 = engine.ls("", 3, 3);
    assert_eq!(page3.items.len(), 1);
    assert_eq!(page3.page, 3);
    assert_eq!(page3.size, 3);
}

/// Ensures compatibility with HTML heading tags (e.g., <h1>), which are sometimes used
/// in place of standard Markdown headings.
#[test]
fn test_html_headings() {
    let content = "<h1>Title</h1>\n\n<h2>Subtitle</h2>\n\nContent.";
    let engine = MarkdownEngine::new(content);
    let result = engine.ls("", 0, 20);
    assert_eq!(result.total, 1);
    assert_eq!(result.items[0], "title");
    let children = engine.ls("title", 0, 20);
    assert_eq!(children.total, 1);
    assert!(children.items[0].ends_with("subtitle"));
}
