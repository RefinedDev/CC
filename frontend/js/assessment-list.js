(function () {
  document.addEventListener("DOMContentLoaded", function () {
    const list = document.getElementById("assessment-list");
    if (!list || !window.CapacityApi) return;
    window.CapacityApi.get("/assessments")
      .then((items) => {
        list.innerHTML = items.length
          ? items
              .map((item) => {
                const result = item.result
                  ? `<small>Score: ${item.result.score}/${item.result.total}</small>`
                  : `<a class="btn btn-primary" href="assessment.html?id=${item.id}">Start</a>`;
                return `<div><span><strong>${item.title}</strong><small>${item.subject} · Due ${item.deadline} · ${item.duration_minutes} minutes</small></span>${result}</div>`;
              })
              .join("")
          : "<p>No assessments are available yet.</p>";
      })
      .catch((error) => {
        list.innerHTML = `<p>${error.message}</p>`;
      });
  });
})();
