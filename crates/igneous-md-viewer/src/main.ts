import type { ClientMsg } from "../../igneous-md-protocol/bindings/ClientMsg";
import type { ServerMsg } from "../../igneous-md-protocol/bindings/ServerMsg";

window.MathJax = {
  options: {
    skipHtmlTags: ["script", "noscript", "style", "textarea"],
  },
  tex: {
    inlineMath: [["$", "$"]],
    displayMath: [["$$", "$$"]],
  },
  startup: {
    ready() {
      MathJax.startup!.defaultReady!();
      processMathCodeElements();
      MathJax.typeset!();
    },
  },
};

/** Wrap raw `code.language-math` elements in `<span>` with the correct delimiters. */
function processMathCodeElements(): void {
  document.querySelectorAll<HTMLElement>("code.language-math").forEach((el) => {
    const isDisplay = el.classList.contains("math-display");
    const wrapper = document.createElement("span");
    wrapper.textContent = isDisplay
      ? "$$" + el.textContent + "$$"
      : "$" + el.textContent + "$";
    el.replaceWith(wrapper);
  });
}

// State
let lastKey = "";

const styleSheet = document.getElementById("md-style") as HTMLStyleElement;

const url = new URL(window.location.href);
const params = new URLSearchParams(url.search);
const exportPath: string | null = params.get("export");

let exportStarted = false;
let contentReady = false;
let cssReady = false;

// WebSocket

const ws = new WebSocket(
  `ws://${window.location.host}/ws/?md_path=${params.get("path")}&update_rate=${params.get("update_rate")}`,
);

/** Send a typed client message over the WebSocket. */
function send(msg: ClientMsg): void {
  ws.send(JSON.stringify(msg));
}

// Message handling
function maybeStartExport(): void {
  if (!exportPath || exportStarted || !contentReady || !cssReady) return;

  exportStarted = true;

  // Give MathJax a chance to finish typesetting before printing
  MathJax.typesetPromise!().then(() => {
    window.webkit.messageHandlers.exportPDF.postMessage(exportPath!);
  });
}

ws.onmessage = (event: MessageEvent) => {
  let data: ServerMsg;
  try {
    data = JSON.parse(event.data) as ServerMsg;
  } catch (error) {
    console.error("Failed to parse ServerMsg:", error);
    return;
  }

  switch (data.t) {
    case "CssUpdate":
      styleSheet.textContent = data.c.css;
      cssReady = true;
      maybeStartExport();
      break;

    case "HtmlUpdate":
      document.body.innerHTML = data.c.html;

      console.log("Markdown updated");
      hljs.configure({
        cssSelector: 'code[class*="language-"]',
      });
      hljs.highlightAll();

      processMathCodeElements();
      MathJax.typeset!();
      contentReady = true;
      maybeStartExport();
      break;

    case "Export":
      window.webkit.messageHandlers.exportPDF.postMessage(data.c.path);
      break;

    // case "Exit":
    //     window.webkit.messageHandlers.exit.postMessage("");
    //     break;

    default:
      console.warn("Unknown message type:", (data as { t: string }).t);
      break;
  }
};

ws.onopen = () => {
  send({ t: "ChangeCss", c: { index: 0, relative: true } });
};

// Keyboard shortcuts
document.addEventListener("keydown", (event: KeyboardEvent) => {
  switch (event.key) {
    case "c":
      send({ t: "ChangeCss", c: { index: 1, relative: true } });
      break;

    case "C":
      send({ t: "ChangeCss", c: { index: -1, relative: true } });
      break;

    case "e":
      send({ t: "RequestExport" });
      break;

    case "r":
      send({ t: "RedirectDefault" });
      window.scrollTo(0, 0);
      break;

    case "j":
      window.scrollBy({ top: 150, behavior: "smooth" });
      break;

    case "k":
      window.scrollBy({ top: -150, behavior: "smooth" });
      break;

    case "g":
      if (lastKey === "g") {
        window.scrollTo({ top: 0, behavior: "smooth" });
        lastKey = "";
      }
      break;

    case "G":
      window.scrollTo({
        top: document.body.scrollHeight,
        behavior: "smooth",
      });
      break;

    case "p":
      send({ t: "RequestExport" });
      break;

    default:
      break;
  }

  lastKey = event.key;
});

// Internal link handling (called from onclick attributes in rendered HTML)
// eslint-disable-next-line @typescript-eslint/no-unused-vars
function handle_redirect(href: string): boolean {
  send({ t: "Redirect", c: { path: href } });
  window.scrollTo(0, 0);
  return false;
}
