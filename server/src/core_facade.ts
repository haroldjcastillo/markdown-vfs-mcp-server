import { createRequire } from "node:module";

export type PaginatedResult = {
  items: string[];
  page: number;
  size: number;
  total: number;
};

type WasmMarkdownDBInstance = {
  ls(path: string, page: number, size: number): PaginatedResult;
  get_title(path: string): string | undefined;
  read(path: string): string | undefined;
  read_full(path: string): string | undefined;
  get_reference(refId: string): string | undefined;
  search(query: string): string[];
};

type WasmModule = { MarkdownDB: new (content: string) => WasmMarkdownDBInstance };

const require = createRequire(import.meta.url);
const { MarkdownDB: WasmMarkdownDB } = require("@hjco/markdown-btree-core") as WasmModule;

/**
 * Facade class that provides a high-level TypeScript API for interacting
 * with the underlying Rust B-Tree index compiled to WebAssembly.
 */
export class CoreFacade {
  private db: WasmMarkdownDBInstance;

  /**
   * Initializes a new Markdown VFS index from the provided content.
   * @param content The raw Markdown string to index.
   */
  constructor(content: string) {
    this.db = new WasmMarkdownDB(content);
  }

  /**
   * Lists child sections for a given path with pagination.
   * @param path The parent path (e.g., "chapter-1"). Use "" for root.
   * @param page The 0-indexed page number.
   * @param size The number of items per page.
   */
  ls(path: string, page: number, size: number): PaginatedResult {
    return this.db.ls(path, page, size);
  }

  /**
   * Retrieves the human-readable title of a section.
   */
  getTitle(path: string): string | undefined {
    return this.db.get_title(path);
  }

  /**
   * Reads the text content of a single section (excluding sub-sections).
   */
  read(path: string): string | undefined {
    return this.db.read(path);
  }

  /**
   * Reads the full text content of a section including all its sub-sections.
   */
  readFull(path: string): string | undefined {
    return this.db.read_full(path);
  }

  /**
   * Resolves a Markdown reference (e.g., [ref]) if indexed.
   */
  getReference(refId: string): string | undefined {
    return this.db.get_reference(refId);
  }

  /**
   * Searches for a query string in section titles.
   * @returns An array of matching section paths.
   */
  search(query: string): string[] {
    return this.db.search(query);
  }
}
