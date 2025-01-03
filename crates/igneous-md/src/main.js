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

		default:
			break;
	}
});

function update_css(css_path) {
	console.log("New Css path:", css_path);
	const oldStyleSheet = document.getElementById("md-stylesheet");
	const newStyleSheet = document.createElement("link");
	newStyleSheet.rel = "stylesheet";
	newStyleSheet.href = css_path;
	newStyleSheet.id = "md-stylesheet";
	document.head.appendChild(newStyleSheet);
	oldStyleSheet.parentNode.removeChild(oldStyleSheet);
}

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
	if (!data) return; // Exit if parsing failed

	switch (data.type) {
		case "CssUpdate":
			update_css(data.body);
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
