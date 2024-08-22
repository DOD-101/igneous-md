let nth_css = 0;

document.addEventListener("keydown", (event) => {
	if (event.key === "c") {
		nth_css++;
		get_css(nth_css);
		return;
	}

	if (event.key === "C") {
		nth_css--;
		get_css(nth_css);
		return;
	}
});

function get_css(n) {
	fetch(`${window.location.origin}/api/get-css-path?n=${n}`)
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

function postProcessHtml(htmlString) {
	const parser = new DOMParser();
	const doc = parser.parseFromString(htmlString, "text/html");
	const listItems = doc.querySelectorAll('li>p>input[type="checkbox"]');

	// apply missing classes for checkbox lists
	listItems.forEach((input) => {
		const li = input.parentNode.parentNode;
		const ul = li.parentNode;

		li.classList.add("task-list-item");
		input.classList.add("task-list-item-checkbox");
		ul.classList.add("contains-task-list");
	});

	return doc.documentElement.outerHTML;
}

const socket = new WebSocket(
	`ws://${window.location.host}/ws?path=${window.location.pathname.slice(1)}`,
	"md-data",
);

socket.onmessage = (event) => {
	// NOTE: post-processing of the HTML only occurs on the first ws message
	// this means that for a few milliseconds the document is formatted
	// slightly incorrectly
	document.getElementById("body").innerHTML = postProcessHtml(event.data);
	console.log("Markdown updated");
	hljs.configure({
		// Stop hljs for detecting languages on code blocks with none specified
		cssSelector: 'code[class*="language-"]',
	});
	hljs.highlightAll();
};
