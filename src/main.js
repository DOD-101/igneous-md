let nth_css = 0;
setInterval(() => {
	console.log("Fetching");
	fetch(`${window.location.href}?update=true`)
		.then((response) => {
			if (!response.ok) {
				throw new Error("Network response was not ok");
			}
			return response.text();
		})
		.then((data) => {
			// WARN: processing of the HTML only happens on the first fetch
			// this means that for the first second the document is formatted
			// slightly incorrectly
			document.getElementById("body").innerHTML = processHtml(data);
			hljs.configure({
				// Stop hljs for detecting languages on code blocks with none specified
				cssSelector: 'code[class*="language-"]',
			});
			hljs.highlightAll();
		})
		.catch((error) => console.error("Fetch error:", error));
}, 1000);

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

function processHtml(htmlString) {
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
