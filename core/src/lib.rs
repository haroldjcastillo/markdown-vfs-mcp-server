pub mod engine;

use engine::MarkdownEngine;
use wasm_bindgen::prelude::*;

/// The primary interface for the Markdown VFS exposed to WebAssembly.
/// It wraps a high-performance B-Tree index of headings and content.
#[wasm_bindgen]
pub struct MarkdownDB {
    inner: MarkdownEngine,
}

#[wasm_bindgen]
impl MarkdownDB {
    /// Creates a new instance by parsing and indexing the provided Markdown string.
    #[wasm_bindgen(constructor)]
    pub fn new(content: &str) -> MarkdownDB {
        MarkdownDB {
            inner: MarkdownEngine::new(content),
        }
    }

    /// Lists child sections for a given path with pagination.
    /// Returns a `JsValue` containing the paginated results.
    pub fn ls(&self, path: &str, page: usize, page_size: usize) -> JsValue {
        let result = self.inner.ls(path, page, page_size);
        serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
    }

    /// Returns the original title of a section at the specified path.
    pub fn get_title(&self, path: &str) -> Option<String> {
        self.inner.get_title(path)
    }

    /// Retrieves the content of a section (excluding nested sub-sections).
    pub fn read(&self, path: &str) -> Option<String> {
        self.inner.read(path)
    }

    /// Retrieves the content of a section INCLUDING all nested sub-sections.
    pub fn read_full(&self, path: &str) -> Option<String> {
        self.inner.read_full(path)
    }

    /// Resolves a reference ID from the Markdown document.
    pub fn get_reference(&self, ref_id: &str) -> Option<String> {
        self.inner.get_reference(ref_id)
    }

    /// Searches for matching section titles and returns their paths.
    pub fn search(&self, query: &str) -> JsValue {
        let result = self.inner.search(query);
        serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
    }
}
