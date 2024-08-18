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
			console.log("Response received: ", data);
			document.getElementById("body").innerHTML = data;
		})
		.catch((error) => console.error("Fetch error:", error));
}, 1000);
