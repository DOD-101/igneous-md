/** Ambient type declarations for globals provided by vendored scripts. */

interface HighlightJs {
  configure(opts: { cssSelector: string }): void;
  highlightAll(): void;
}

declare const hljs: HighlightJs;

/**
 * MathJax global.
 *
 * Before the MathJax library loads, this is a config object set by our code.
 * After MathJax loads, it populates additional runtime methods (typeset,
 * startup.defaultReady, etc.). All properties are optional to reflect this
 * two-phase lifecycle.
 */
interface MathJax {
  options?: { skipHtmlTags?: string[] };
  tex?: {
    inlineMath?: [string, string][];
    displayMath?: [string, string][];
  };
  startup?: {
    ready?: () => void;
    defaultReady?: () => void;
  };
  typeset?(): void;
  typesetPromise?(): Promise<void>;
}

declare let MathJax: MathJax;

interface Window {
  MathJax: MathJax;
  webkit: {
    messageHandlers: {
      exportPDF: {
        postMessage(path: string): void;
      };
    };
  };
}
