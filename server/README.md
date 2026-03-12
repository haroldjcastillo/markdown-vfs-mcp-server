# @hjco/markdown-vfs-mcp-server

A **Model Context Protocol (MCP)** server that exposes large Markdown files as a Virtual File System (VFS), powered by a Rust/WASM B-Tree index for efficient navigation.

## Installation

```bash
npm install -g @hjco/markdown-vfs-mcp-server
```

Or run directly without installing:

```bash
npx @hjco/markdown-vfs-mcp-server
```

## MCP Configuration (stdio)

Add to your MCP client config file. The configuration is identical for Claude Desktop, Cursor, VS Code, and any other MCP-compatible client:

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

| Client | Config file |
|---|---|
| Claude Desktop (macOS) | `~/Library/Application Support/Claude/claude_desktop_config.json` |
| Claude Desktop (Windows) | `%APPDATA%\Claude\claude_desktop_config.json` |
| Cursor | `.cursor/mcp.json` in your project |
| VS Code (MCP extension) | `.vscode/mcp.json` in your project |

> You can omit `MARKDOWN_PATH` and call the `load_markdown` tool at runtime instead.

## Available Tools

| Tool | Description |
|---|---|
| `load_markdown` | Load or replace the active Markdown file |
| `ls_markdown` | List the hierarchical section structure (with pagination) |
| `read_section` | Read the text content of a specific section |
| `search_index` | Search section titles by keyword |

## License

MIT
