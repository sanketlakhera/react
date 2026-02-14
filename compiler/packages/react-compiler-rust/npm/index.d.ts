/** Result from compiling JavaScript/TypeScript code */
export interface CompileResult {
  /** The compiled output code */
  code: string;
  /** Whether compilation was successful */
  success: boolean;
  /** Error message if compilation failed */
  error: string | null;
}

/**
 * Compile JavaScript/TypeScript source code to optimized JavaScript
 * with automatic memoization (useMemoCache patterns).
 *
 * @param source - The source code to compile
 * @returns CompileResult with compiled code or error
 */
export function compile(source: string): CompileResult;

/**
 * Compile with options for file type
 *
 * @param source - The source code to compile
 * @param fileType - File type: "js", "jsx", "ts", "tsx"
 * @returns CompileResult with compiled code or error
 */
export function compileWithOptions(source: string, fileType?: string): CompileResult;

/**
 * Get version information
 */
export function version(): string;
