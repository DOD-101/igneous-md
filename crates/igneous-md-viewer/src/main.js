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
                    type: "ChangeCssNext",
                }),
            );
            break;

        case "C":
            ws.send(
                JSON.stringify({
                    type: "ChangeCssPrev",
                }),
            );
            break;

        case "e":
            ws.send(
                JSON.stringify({
                    type: "ExportHtml",
                }),
            );
            break;

        case "r":
            ws.send(
                JSON.stringify({
                    type: "RedirectDefault",
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

        default:
            break;
    }

    lastKey = event.key;
});

function handle_redirect(href) {
    ws.send(
        JSON.stringify({
            type: "Redirect",
            body: href,
        }),
    );

    window.scrollTo(0, 0);

    return false;
}

const url = new URL(window.location.href);

const params = new URLSearchParams(url.search);

const ws = new WebSocket(
    // HACK: SET PORT VIA URL PARAM
    `ws://localhost:2323/ws/?path=${params.get("path")}&update_rate=${params.get("update_rate")}`,
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

    switch (data.type) {
        case "CurrentCss": {
            console.log(styleSheet, data);
            styleSheet.textContent = data.body;
            break;
        }
        case "CssChange":
            styleSheet.href = `${data.body}?_noise=${Math.random()}`;
            break;
        case "CssUpdate":
            styleSheet.href = `${styleSheet.href.split("?")[0]}?_noise=${Math.random()}`;
            break;
        case "HtmlUpdate":
            {
                const main = document.body;

                main.innerHTML = data.body;

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
        default:
            console.warn("Unknown message type:", data.type);
            break;
    }
};

ws.onopen = () => {
    ws.send(
        JSON.stringify({
            type: "CurrentCss",
        }),
    );
};
