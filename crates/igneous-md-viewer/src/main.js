MathJax = {
    options: {
        skipHtmlTags: ["script", "noscript", "style", "textarea"],
    },
    tex: {
        inlineMath: [["$", "$"]],
        displayMath: [["$$", "$$"]],
    },
    startup: {
        ready() {
            MathJax.startup.defaultReady();
            // Re-process code.language-math elements
            document.querySelectorAll("code.language-math").forEach((el) => {
                const isDisplay = el.classList.contains("math-display");
                const wrapper = document.createElement("span");
                wrapper.textContent = isDisplay
                    ? "$$" + el.textContent + "$$"
                    : "$" + el.textContent + "$";
                el.replaceWith(wrapper);
            });
            MathJax.typesetPromise();
        },
    },
};

let lastKey = "";

const styleSheet = document.getElementById("md-style");

document.addEventListener("keydown", (event) => {
    switch (event.key) {
        case "c":
            ws.send(
                JSON.stringify({
                    t: "ChangeCss",
                    c: { index: 1, relative: true },
                }),
            );
            break;

        case "C":
            ws.send(
                JSON.stringify({
                    t: "ChangeCss",
                    c: { index: -1, relative: true },
                }),
            );
            break;

        case "e":
            ws.send(
                JSON.stringify({
                    t: "ExportHtml",
                    c: {},
                }),
            );
            break;

        case "r":
            ws.send(
                JSON.stringify({
                    t: "RedirectDefault",
                }),
            );

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
            ws.send(
                JSON.stringify({
                    t: "RequestExport",
                }),
            );
            break;

        default:
            break;
    }

    lastKey = event.key;
});

function handle_redirect(href) {
    ws.send(
        JSON.stringify({
            t: "Redirect",
            c: {
                path: href,
            },
        }),
    );

    window.scrollTo(0, 0);

    return false;
}

const url = new URL(window.location.href);

const params = new URLSearchParams(url.search);

const ws = new WebSocket(
    `ws://${window.location.host}/ws/?md_path=${params.get("path")}&update_rate=${params.get("update_rate")}`,
);

function safeParse(jsonString) {
    try {
        return JSON.parse(jsonString);
    } catch (error) {
        console.error("Failed to parse ServerMsg:", error);
        return null; // Return null if parsing fails
    }
}

ws.onmessage = (event) => {
    const data = safeParse(event.data);

    if (!data) return;

    // TODO: It would be nice to use the cached css file if it hasn't changed

    const tag = data.t;
    const content = data.c;

    switch (tag) {
        case "CssUpdate":
            styleSheet.textContent = content.css;
            break;
        case "HtmlUpdate":
            {
                const main = document.body;

                main.innerHTML = content.html;

                console.log("Markdown updated");
                hljs.configure({
                    cssSelector: 'code[class*="language-"]', // Highlight.js configuration
                });
                hljs.highlightAll();

                document
                    .querySelectorAll("code.language-math")
                    .forEach((el) => {
                        {
                            const isDisplay =
                                el.classList.contains("math-display");
                            const wrapper = document.createElement("span");
                            wrapper.textContent = isDisplay
                                ? "$$" + el.textContent + "$$"
                                : "$" + el.textContent + "$";
                            el.replaceWith(wrapper);
                        }
                    });
                MathJax.typeset();
            }
            break;
        case "Export":
            window.webkit.messageHandlers.exportPDF.postMessage(content.path);
            break;
        // case "Exit":
        //     window.webkit.messageHandlers.exit.postMessage("");
        //     break;
        default:
            console.warn("Unknown message type:", tag);
            break;
    }
};

ws.onopen = () => {
    ws.send(
        JSON.stringify({
            t: "ChangeCss",
            c: { index: 0, relative: true },
        }),
    );
};
