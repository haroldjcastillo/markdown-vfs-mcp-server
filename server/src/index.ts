#!/usr/bin/env node
import fs from "node:fs";
import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
  CallToolRequestSchema,
  ErrorCode,
  ListToolsRequestSchema,
  McpError,
} from "@modelcontextprotocol/sdk/types.js";
import { CoreFacade } from "./core_facade.js";

type LoadedState = {
  facade: CoreFacade;
  sourcePath: string;
};

/**
 * Validates that a value is a string.
 * @throws {McpError} If the value is not a string.
 */
function ensureString(value: unknown, name: string): string {
  if (typeof value !== "string") {
    throw new McpError(ErrorCode.InvalidParams, `Expected string for '${name}'.`);
  }
  return value;
}

/**
 * Parses an optional path string, defaulting to an empty string for root.
 */
function parseOptionalPath(value: unknown): string {
  if (value === undefined) {
    return "";
  }

  if (typeof value !== "string") {
    throw new McpError(ErrorCode.InvalidParams, "Expected string for 'path'.");
  }

  return value;
}

/**
 * Parses pagination parameters with fallback values.
 */
function parsePagination(value: unknown, fallback: number, name: string): number {
  if (value === undefined) {
    return fallback;
  }

  if (typeof value !== "number" || !Number.isFinite(value)) {
    throw new McpError(ErrorCode.InvalidParams, `Expected number for '${name}'.`);
  }

  return Math.max(0, Math.floor(value));
}

/**
 * Parses boolean flags with fallback values.
 */
function parseBoolean(value: unknown, fallback: boolean, name: string): boolean {
  if (value === undefined) {
    return fallback;
  }

  if (typeof value !== "boolean") {
    throw new McpError(ErrorCode.InvalidParams, `Expected boolean for '${name}'.`);
  }

  return value;
}

/**
 * Simple heuristic for token estimation (roughly 4 chars per token).
 */
function estimateTokens(text: string): number {
  if (text.length === 0) {
    return 0;
  }

  return Math.ceil(text.length / 4);
}

/**
 * Generates a human-readable title from a hierarchical path.
 */
function titleFromPath(path: string): string {
  const raw = path.split("/").pop() ?? path;
  return raw.replace(/-/g, " ");
}

type SectionItem = {
  path: string;
  title: string;
};

type SectionWithStats = SectionItem & {
  chars: number;
  estimated_tokens: number;
};

/**
 * Main entry point for the Markdown VFS MCP Server.
 */
async function main(): Promise<void> {
  let loadedState: LoadedState | undefined;

  /**
   * Loads a Markdown file into the Rust core engine.
   */
  const loadMarkdown = (rawPath: string): LoadedState => {
    const inputPath = rawPath.trim();
    if (inputPath.length === 0) {
      throw new McpError(ErrorCode.InvalidParams, "'path' cannot be empty.");
    }

    if (!fs.existsSync(inputPath)) {
      throw new McpError(ErrorCode.InvalidParams, `File not found: ${inputPath}`);
    }

    const markdownContent = fs.readFileSync(inputPath, "utf-8");
    const facade = new CoreFacade(markdownContent);
    const rootResult = facade.ls("", 0, 1);
    if (rootResult.total === 0) {
      throw new McpError(
        ErrorCode.InvalidParams,
        "The markdown file has no sections (no headings found).",
      );
    }

    return {
      facade,
      sourcePath: inputPath,
    };
  };

  const markdownPath = process.env.MARKDOWN_PATH;
  if (markdownPath) {
    loadedState = loadMarkdown(markdownPath);
  }

  const server = new Server(
    {
      name: "markdown-vfs-mcp-server",
      version: "0.1.0",
    },
    {
      capabilities: {
        tools: {},
      },
    },
  );

  server.setRequestHandler(ListToolsRequestSchema, async () => ({
    tools: [
      {
        name: "load_markdown",
        description:
          "Loads or replaces the active Markdown document into the VFS. This should be the first tool used if no environment variable was provided. Use absolute or relative paths.",
        inputSchema: {
          type: "object",
          properties: {
            path: {
              type: "string",
              description: "The file system path to the .md file.",
            },
          },
          required: ["path"],
          additionalProperties: false,
        },
      },
      {
        name: "ls_markdown",
        description:
          "Explores the document hierarchy. Returns child sections for a given path using pagination. Use this to 'browse' the document like a directory tree. Use an empty path ('') to see the root chapters.",
        inputSchema: {
          type: "object",
          properties: {
            path: {
              type: "string",
              default: "",
              description: "Hierarchical path (e.g. 'chapter-1/introduction'). Leave empty for root.",
            },
            page: {
              type: "number",
              default: 0,
              description: "Page number (starting from 0).",
            },
            size: {
              type: "number",
              default: 20,
              description: "Items per page (max 50 recommended).",
            },
            include_stats: {
              type: "boolean",
              default: false,
              description:
                "Include character count and token estimates for each section. Useful for managing context limits.",
            },
            full: {
              type: "boolean",
              default: false,
              description:
                "When used with include_stats=true, estimates section size using full section text including nested sub-sections.",
            },
          },
          additionalProperties: false,
        },
      },
      {
        name: "read_section",
        description:
          "Retrieves the content of a specific section by its path. Use this tool only after identifying the correct section path via 'ls_markdown' or 'search_index'.",
        inputSchema: {
          type: "object",
          properties: {
            path: {
              type: "string",
              description: "The exact hierarchical path of the section to read.",
            },
            full: {
              type: "boolean",
              default: false,
              description:
                "If true, returns the content of this section PLUS all its sub-sections (be careful with large chapters).",
            },
          },
          required: ["path"],
          additionalProperties: false,
        },
      },
      {
        name: "search_index",
        description:
          "Performs a keyword search across all section titles in the document. Returns a list of matching paths which can then be read using 'read_section'.",
        inputSchema: {
          type: "object",
          properties: {
            query: {
              type: "string",
              description: "Keyword or phrase to search for in titles.",
            },
          },
          required: ["query"],
          additionalProperties: false,
        },
      },
    ],
  }));

  server.setRequestHandler(CallToolRequestSchema, async (request) => {
    const { name, arguments: args } = request.params;

    if (name === "load_markdown") {
      const path = ensureString(args?.path, "path");
      loadedState = loadMarkdown(path);
      return {
        content: [
          {
            type: "text",
            text: JSON.stringify(
              {
                loaded: true,
                path: loadedState.sourcePath,
                root_sections: loadedState.facade.ls("", 0, 1).total,
              },
              null,
              2,
            ),
          },
        ],
      };
    }

    if (!loadedState) {
      throw new McpError(
        ErrorCode.InvalidRequest,
        "No markdown loaded. Call 'load_markdown' first or set MARKDOWN_PATH before startup.",
      );
    }

    const activeFacade = loadedState.facade;

    if (name === "ls_markdown") {
      const path = parseOptionalPath(args?.path);
      const page = parsePagination(args?.page, 0, "page");
      const size = parsePagination(args?.size ?? args?.page_size, 20, "size");
      const includeStats = parseBoolean(args?.include_stats, false, "include_stats");
      const full = parseBoolean(args?.full, false, "full");

      if (path !== "" && activeFacade.read(path) === undefined) {
        throw new McpError(ErrorCode.InvalidParams, `Path not found: ${path}`);
      }

      const result = activeFacade.ls(path, page, size);

      if (includeStats) {
        const sections: SectionWithStats[] = result.items.map((itemPath) => {
          const content = full
            ? (activeFacade.readFull(itemPath) ?? "")
            : (activeFacade.read(itemPath) ?? "");
          return {
            path: itemPath,
            title: activeFacade.getTitle(itemPath) ?? titleFromPath(itemPath),
            chars: content.length,
            estimated_tokens: estimateTokens(content),
          };
        });

        return {
          content: [
            {
              type: "text",
              text: JSON.stringify(
                {
                  sections,
                  page: result.page,
                  size: result.size,
                  total: result.total,
                },
                null,
                2,
              ),
            },
          ],
        };
      }

      const sections: SectionItem[] = result.items.map((itemPath) => ({
        path: itemPath,
        title: activeFacade.getTitle(itemPath) ?? titleFromPath(itemPath),
      }));

      return {
        content: [
          {
            type: "text",
            text: JSON.stringify(
              {
                sections,
                page: result.page,
                size: result.size,
                total: result.total,
              },
              null,
              2,
            ),
          },
        ],
      };
    }

    if (name === "read_section") {
      const path = ensureString(args?.path, "path");
      const full = parseBoolean(args?.full, false, "full");
      const content = full ? activeFacade.readFull(path) : activeFacade.read(path);
      if (content === undefined) {
        throw new McpError(ErrorCode.InvalidParams, `Path not found: ${path}`);
      }

      return {
        content: [{ type: "text", text: content }],
      };
    }

    if (name === "search_index") {
      const query = ensureString(args?.query, "query");
      const result = activeFacade.search(query);
      return {
        content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
      };
    }

    throw new McpError(ErrorCode.MethodNotFound, `Unknown tool: ${name}`);
  });

  const transport = new StdioServerTransport();
  await server.connect(transport);
}

main().catch((error) => {
  console.error("Failed to start MCP server:", error);
  process.exit(1);
});
