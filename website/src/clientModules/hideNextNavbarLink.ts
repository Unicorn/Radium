/**
 * Client module to hide "Next" link in the navbar
 * This runs in the browser after the page loads
 */

export default function hideNextNavbarLink() {
	// Function to hide navbar items with "Next" text
	const hideNextLink = () => {
		// Find the navbar container
		const navbar = document.querySelector(".navbar");
		if (!navbar) return;

		// Find all links within the navbar (using multiple selectors)
		const allLinks = navbar.querySelectorAll("a");

		allLinks.forEach((link) => {
			const text = link.textContent?.trim();
			const href = link.getAttribute("href");

			// Check if the link text is exactly "Next" OR if it points to /docs/introduction
			if (
				text === "Next" ||
				href === "/docs/introduction" ||
				href?.includes("/docs/introduction")
			) {
				// Hide the link itself
				(link as HTMLElement).style.display = "none";
				(link as HTMLElement).style.visibility = "hidden";
				(link as HTMLElement).style.opacity = "0";
				(link as HTMLElement).style.width = "0";
				(link as HTMLElement).style.height = "0";
				(link as HTMLElement).style.overflow = "hidden";

				// Hide parent elements (item, div, etc.)
				let parent = link.parentElement;
				let depth = 0;
				while (parent && parent !== navbar && depth < 5) {
					// Check if this is a navbar item or similar container
					if (
						parent.classList.contains("navbar__item") ||
						parent.classList.contains("navbar__link") ||
						(parent.tagName === "DIV" && parent.parentElement === navbar)
					) {
						(parent as HTMLElement).style.display = "none";
						(parent as HTMLElement).style.visibility = "hidden";
					}
					parent = parent.parentElement;
					depth++;
				}
			}
		});

		// Also check navbar items directly
		const navbarItems = navbar.querySelectorAll(".navbar__item");
		navbarItems.forEach((item) => {
			const link = item.querySelector("a");
			if (link) {
				const text = link.textContent?.trim();
				const href = link.getAttribute("href");
				if (
					text === "Next" ||
					href === "/docs/introduction" ||
					href?.includes("/docs/introduction")
				) {
					(item as HTMLElement).style.display = "none";
					(item as HTMLElement).style.visibility = "hidden";
				}
			}
		});
	};

	// Run when DOM is ready
	if (document.readyState === "loading") {
		document.addEventListener("DOMContentLoaded", hideNextLink);
	} else {
		hideNextLink();
	}

	// Also run after delays to catch dynamically added items
	setTimeout(hideNextLink, 100);
	setTimeout(hideNextLink, 500);
	setTimeout(hideNextLink, 1000);

	// Use MutationObserver to catch items added after initial load
	const observer = new MutationObserver(() => {
		hideNextLink();
	});

	const navbar = document.querySelector(".navbar");
	if (navbar) {
		observer.observe(navbar, {
			childList: true,
			subtree: true,
		});
	}
}
