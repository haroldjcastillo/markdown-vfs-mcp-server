# Markdown VFS (Model Context Protocol Server)

A high-performance **Model Context Protocol (MCP)** server that exposes large Markdown files as a **Virtual File System (VFS)**. It uses a Rust-powered B-Tree index compiled to WebAssembly for efficient navigation and retrieval of specific document sections.

## 🚀 Key Features

- **Hierarchical Navigation**: Treats Markdown headings as a folder-like structure (e.g., `chapter-1/introduction`).
- **Rust Core (WASM)**: High-speed B-Tree indexing and searching, even for multi-megabyte Markdown files.
- **Agent Optimized**: Designed specifically for LLMs (like Claude) to explore and read large documents without hitting context limits.
- **Paginated Discovery**: Navigate deep structures without loading unnecessary content.
- **Smart Metadata**: Provides character counts and estimated tokens to help agents manage context budgets.

## 🏗️ Architecture

```text
┌────────────────┐      ┌─────────────────┐      ┌─────────────────┐
│   MCP Client   │ ◄───►│  TypeScript Srv │ ◄───►│    Rust Core    │
│ (Claude, etc.) │ stdio│  (MCP Adapter)  │ WASM │  (B-Tree Index) │
└────────────────┘      └─────────────────┘      └─────────────────┘
```

## 📦 Installation

Install the MCP server globally from npm:

```bash
npm install -g @hjco/markdown-vfs-mcp-server
```

Or use it directly with `npx` without installing:

```bash
npx @hjco/markdown-vfs-mcp-server
```

## ⚡ MCP Client Configuration

Add the server to your MCP client configuration. It communicates over **stdio**. The configuration is the same for all clients (Claude Desktop, Cursor, VS Code MCP extension, etc.).

Edit your client's MCP config file and add:

```json
{
  "mcpServers": {
    "markdown-vfs": {
      "command": "npx",
      "args": ["-y", "@hjco/markdown-vfs-mcp-server"],
      "env": {
        "MARKDOWN_PATH": "/absolute/path/to/your/document.md"
      }
    }
  }
}
```

| Client | Config file location |
|---|---|
| Claude Desktop (macOS) | `~/Library/Application Support/Claude/claude_desktop_config.json` |
| Claude Desktop (Windows) | `%APPDATA%\Claude\claude_desktop_config.json` |
| Cursor | `.cursor/mcp.json` in your project root |
| VS Code (MCP extension) | `.vscode/mcp.json` in your project root |

> **Tip:** You can omit `MARKDOWN_PATH` and call the `load_markdown` tool at runtime to load a file dynamically.

---

## 🛠️ Development Prerequisites

> Only needed if you want to build from source.

- **Node.js**: v20 or higher
- **Rust**: Stable toolchain (2021 edition)
- **wasm-pack**: For compiling the Rust core

## 🔨 Build from Source

The project uses a **Makefile** to automate the build process across the Rust and TypeScript components.

### Automated Build (Recommended)
From the project root, run:

```bash
# Full build: compiles Rust core to WASM and builds the TS server
make build
```

### Development Utilities

```bash
make check   # Run Rust clippy and cargo check
make test    # Execute Rust core tests
make lint    # Run TypeScript linting
make fmt     # Check code formatting (Rust and TS)
make clean   # Remove all build artifacts and node_modules
```

## 🖥️ Running Locally (from source)

```bash
MARKDOWN_PATH=/path/to/your/book.md node server/dist/index.js
```

## 🛠️ MCP Tools Reference

### `load_markdown`
Loads or replaces the active Markdown file.
- **Arguments**: `path` (string)
- **Use Case**: Initializing the VFS or switching documents.

### `ls_markdown`
Lists the hierarchical structure (headings) of the document.
- **Arguments**: `path` (string, default: ""), `page` (number), `size` (number), `include_stats` (boolean), `full` (boolean).
- **Use Case**: Discovering chapters or sub-sections.

If `include_stats=true` and `full=true`, the `chars` and `estimated_tokens` values are calculated from the complete section content including nested sub-sections.

### `read_section`
Retrieves the text content of a specific section.
- **Arguments**: `path` (string), `full` (boolean).
- **Use Case**: Reading the actual text of a discovered section.

### `search_index`
Performs a global search on section titles.
- **Arguments**: `query` (string).
- **Use Case**: Quickly finding specific topics in a massive document.

## 🤖 Agent Integration Instructions

This server is optimized for **iterative discovery**. Agents should follow this workflow:
1. Call `ls_markdown` with `path: ""` to see the root chapters.
2. Use `search_index` if looking for a specific keyword.
3. Call `ls_markdown` on specific paths to drill down into sub-sections.
4. Call `read_section` only when the specific content needed is identified.

## 🧪 Testing Strategy

The project follows a multi-layered testing approach to ensure the reliability of the hierarchical indexing and the MCP server interface.

### Core Logic Tests (Rust)
The most critical part of the system—the B-Tree indexing and Markdown parsing—is extensively tested in Rust. These tests verify:
- **Heading Levels**: Correct parent-child relationships between different heading depths.
- **Slug Generation**: Unique, URL-safe path generation even with duplicate titles.
- **Content Retrieval**: Precise extraction of text content excluding metadata or sub-sections.
- **Pagination**: Reliability of the `ls` tool across large documents.
- **Search Accuracy**: Case-insensitive keyword matching within the title index.

Run these tests with:
```bash
make test
```

### Server Integration
While the core logic is in Rust, the TypeScript layer acts as the MCP protocol adapter. Manual verification of the server is recommended by running it against the provided `test-data/large-book.md`:

```bash
MARKDOWN_PATH=./test-data/large-book.md node server/dist/index.js
```

## 📝 License

MIT
