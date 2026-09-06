(function () {
  "use strict";

  if (!window.CapacityApi || !window.CapacityApi.get) {
    document.getElementById("userDistribution").textContent = "Unavailable";
    document.getElementById("distributionLegend").textContent =
      "API client failed to load. Refresh the page.";
    return;
  }

  const percent = (value, total) =>
    total ? Math.round((value / total) * 100) : 0;

  window.CapacityApi.get("/admin/stats")
    .then((stats) => {
      document.getElementById("statUsers").textContent = stats.users;
      document.getElementById("statCourses").textContent = stats.courses;
      document.getElementById("statEnrollments").textContent =
        stats.enrollments;
      document.getElementById("statCertificates").textContent =
        stats.certificates;
      document.getElementById("userDistribution").textContent =
        `${percent(stats.trainees, stats.users)}%`;
      document.getElementById("distributionLegend").innerHTML =
        `🔵 Trainee ${percent(stats.trainees, stats.users)}%<br>` +
        `🟣 Trainer ${percent(stats.trainers, stats.users)}%<br>` +
        `🟢 Admin ${percent(stats.admins, stats.users)}%`;
      document.getElementById("enrollment-overview").innerHTML =
        `<p><strong>${stats.enrollments}</strong> total enrollments across <strong>${stats.courses}</strong> courses.</p>`;
    })
    .catch((error) => {
      document.getElementById("userDistribution").textContent = "—";
      document.getElementById("distributionLegend").textContent =
        error.message || "Unable to load statistics.";
    });

  const form = document.getElementById("publish-form");
  window.CapacityApi.get("/admin/users")
    .then((users) => {
      const target = document.getElementById("publish-target");
      users.forEach((user) =>
        target.insertAdjacentHTML(
          "beforeend",
          `<option value="${user.id}">${user.name} (${user.email})</option>`,
        ),
      );
    })
    .catch((error) => {
      const target = document.getElementById("publish-target");
      if (target) target.disabled = true;
      const message = document.getElementById("publish-message");
      if (message)
        message.textContent =
          error.message || "Targeted notifications are unavailable.";
    });
  if (form)
    form.onsubmit = async (event) => {
      event.preventDefault();
      const message = document.getElementById("publish-message");
      try {
        await window.CapacityApi.post("/publishing", {
          kind: document.getElementById("publish-kind").value,
          target_user_id:
            document.getElementById("publish-target").value || null,
          title: document.getElementById("publish-title").value,
          body: document.getElementById("publish-body").value,
        });
        form.reset();
        message.textContent = "Content published.";
        loadPublished();
      } catch (error) {
        message.textContent = error.message;
      }
    };
  async function loadPublished() {
    const list = document.getElementById("published-content-list");
    if (!list) return;
    try {
      const items = await window.CapacityApi.get("/publishing/manage");
      document.getElementById("admin-activity").innerHTML = items.length
        ? items
            .slice(0, 5)
            .map(
              (item) =>
                `<div class="activity">● ${item.title} <small>${item.kind} · ${item.created_at}</small></div>`,
            )
            .join("")
        : "<p>No published activity yet.</p>";
      list.innerHTML = items.length
        ? items
            .map(
              (item) =>
                `<div><span><strong>${item.title}</strong><small>${item.kind} · ${item.target_user_id || "Everyone"} · ${item.body}</small></span><button class="edit-publication" data-id="${item.id}">Edit</button> <button class="delete-publication" data-id="${item.id}">Delete</button></div>`,
            )
            .join("")
        : "<p>No published content yet.</p>";
      list.querySelectorAll(".delete-publication").forEach(
        (button) =>
          (button.onclick = async () => {
            if (confirm("Delete this publication?")) {
              await window.CapacityApi.delete(
                `/publishing/${button.dataset.id}`,
              );
              loadPublished();
            }
          }),
      );
      list.querySelectorAll(".edit-publication").forEach(
        (button) =>
          (button.onclick = async () => {
            const item = items.find(
              (value) => String(value.id) === button.dataset.id,
            );
            const title = prompt("Title", item.title);
            const body = title === null ? null : prompt("Message", item.body);
            if (title === null || body === null) return;
            await window.CapacityApi.put(`/publishing/${item.id}`, {
              kind: item.kind,
              title,
              body,
              target_user_id: item.target_user_id,
            });
            loadPublished();
          }),
      );
    } catch (error) {
      list.innerHTML = `<p>${error.message}</p>`;
    }
  }
  loadPublished();
})();
