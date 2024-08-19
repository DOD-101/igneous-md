let nth_css = 0;
setInterval(() => {
	console.log("Fetching");
	fetch(`${window.location.href}?update=true`)
		.then((response) => {
			if (!response.ok) {
				throw new Error("Network response was not ok");
			}
			return response.text(); // Use response.json() for JSON data
		})
		.then((data) => {
			// console.log("Response received: ", data);
			document.getElementById("body").innerHTML = data;
		})
		.catch((error) => console.error("Fetch error:", error));
}, 1000);

document.addEventListener("keydown", (event) => {
	if (event.key !== "c") {
		return;
	}
	nth_css++;
	fetch(`${window.location.origin}/api/get-css?n=${nth_css}`)
		.then((response) => {
			if (!response.ok) {
				throw new Error("Network response was not ok");
			}
			return response.text(); // Use response.json() for JSON data
		})
		.then((data) => {
			console.log("New Css path:", data);
			const oldStyleSheet = document.querySelector('link[rel="stylesheet"]');
			const newStyleSheet = document.createElement("link");
			newStyleSheet.rel = "stylesheet";
			newStyleSheet.href = data;
			document.head.appendChild(newStyleSheet);
			oldStyleSheet.parentNode.removeChild(oldStyleSheet);
		})
		.catch((error) => console.error("Fetch error:", error));
});
