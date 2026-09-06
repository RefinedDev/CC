(function () {
  const target = document.getElementById("public-content");
  if (!target) return;
  fetch(
    `${window.CAPACITY_API_BASE || "http://localhost:6969"}/api/publishing?kind=content`,
  )
    .then((response) => response.json())
    .then((items) => {
      target.innerHTML = items.length
        ? items
            .slice(0, 3)
            .map(
              (item) =>
                `<div class="activity"><strong>${item.title}</strong><p>${item.body}</p></div>`,
            )
            .join("")
        : "<p>No new learning content has been published yet.</p>";
    })
    .catch(() => {
      target.innerHTML = "<p>Latest learning content is unavailable.</p>";
    });
})();
