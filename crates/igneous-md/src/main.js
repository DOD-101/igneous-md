let lastKey = "";

document.addEventListener("keydown", (event) => {
	switch (event.key) {
		case "c":
			socket.send(
				JSON.stringify({
					type: "ChangeCssNext",
				}),
			);
			break;

		case "C":
			socket.send(
				JSON.stringify({
					type: "ChangeCssPrev",
				}),
			);
			break;

		case "e":
			socket.send(
				JSON.stringify({
					type: "ExportHtml",
				}),
			);
			break;

		case "r":
			socket.send(
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
			window.scrollTo({ top: document.body.scrollHeight, behavior: "smooth" });
			break;

		default:
			break;
	}

	lastKey = event.key;
});

function handle_redirect(href) {
	socket.send(
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

const socket = new WebSocket(
	`ws://${window.location.host}/ws/?path=${params.get("path")}`,
);

function safeParse(jsonString) {
	try {
		return JSON.parse(jsonString);
	} catch (error) {
		console.error("Failed to parse ServerMsg:", error);
		return null; // Return null if parsing fails
	}
}

socket.onmessage = (event) => {
	const data = safeParse(event.data);
	if (!data) return;

	// TODO: It would be nice to use the cached css file if it hasn't changed

	switch (data.type) {
		case "CssChange":
			{
				const styleSheet = document.getElementById("md-stylesheet");

				styleSheet.href = `${data.body}?_noise=${Math.random()}`;
			}
			break;
		case "CssUpdate":
			{
				const styleSheet = document.getElementById("md-stylesheet");

				styleSheet.href = `${styleSheet.href.split("?")[0]}?_noise=${Math.random()}`;
			}
			break;
		case "HtmlUpdate":
			document.getElementById("body").innerHTML = data.body;
			console.log("Markdown updated");
			hljs.configure({
				cssSelector: 'code[class*="language-"]', // Highlight.js configuration
			});
			hljs.highlightAll();
			break;
		default:
			console.warn("Unknown message type:", data.type);
			break;
	}
};
