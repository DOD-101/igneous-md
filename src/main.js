document.addEventListener("keydown", (event) => {
	if (event.key === "c") {
		get_css("next");
		return;
	}

	if (event.key === "C") {
		get_css("prev");
		return;
	}
});

document.addEventListener("keydown", (event) => {
	if (event.key === "e") {
		post_html(document.documentElement.outerHTML);
	}

	if (event.key === "E") {
		post_html(document.body.outerHTML);
	}
});

function get_css(direction) {
	fetch(`${window.location.origin}/api/get-css-path/${direction}`)
		.then((response) => {
			if (!response.ok) {
				throw new Error("Network response was not ok");
			}
			return response.text(); // Use response.json() for JSON data
		})
		.then((data) => {
			console.log("New Css path:", data);
			const oldStyleSheet = document.getElementById("md-stylesheet");
			const newStyleSheet = document.createElement("link");
			newStyleSheet.rel = "stylesheet";
			newStyleSheet.href = data;
			newStyleSheet.id = "md-stylesheet";
			document.head.appendChild(newStyleSheet);
			oldStyleSheet.parentNode.removeChild(oldStyleSheet);
		})
		.catch((error) => console.error("Fetch error:", error));
}

function post_html(htmlString) {
	fetch(`${window.location.origin}/api/post-html`, {
		method: "POST",
		headers: {
			"Content-Type": "text/html",
		},
		body: htmlString,
	})
		.then((response) => response.text())
		.then((data) => console.log(data))
		.catch((error) => console.error(error));
}

const socket = new WebSocket(
	`ws://${window.location.host}/ws/${window.location.pathname}`,
);

socket.onmessage = (event) => {
	document.getElementById("body").innerHTML = event.data;
	console.log("Markdown updated");
	hljs.configure({
		// Stop hljs for detecting languages on code blocks with none specified
		cssSelector: 'code[class*="language-"]',
	});
	hljs.highlightAll();
};
