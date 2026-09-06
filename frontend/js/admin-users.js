(function () {
  "use strict";

  const list = document.getElementById("userList");
  const message = document.getElementById("userMessage");
  if (!list || !message) return;
  if (!window.CapacityApi || !window.CapacityApi.get) {
    list.innerHTML = "<p>API client failed to load. Refresh the page.</p>";
    return;
  }

  const escapeText = (value) =>
    String(value).replace(
      /[&<>'"]/g,
      (character) =>
        ({
          "&": "&amp;",
          "<": "&lt;",
          ">": "&gt;",
          "'": "&#39;",
          '"': "&quot;",
        })[character],
    );

  async function loadUsers() {
    const users = await window.CapacityApi.get("/admin/users");
    list.innerHTML = users
      .map(
        (user) => `
      <div style="display:flex;align-items:center;justify-content:space-between;gap:12px">
        <span><strong>${escapeText(user.name)}</strong><small>${escapeText(user.email)}</small></span>
        <select data-user-id="${escapeText(user.id)}">
          ${["trainee", "trainer", "admin"]
            .map(
              (role) =>
                `<option value="${role}" ${role === user.role ? "selected" : ""}>${role}</option>`,
            )
            .join("")}
        </select>
      </div>
    `,
      )
      .join("");
    list.querySelectorAll("select[data-user-id]").forEach((select) => {
      select.onchange = async () => {
        try {
          await window.CapacityApi.post(
            `/admin/users/${encodeURIComponent(select.dataset.userId)}/role`,
            { role: select.value },
          );
          message.textContent = "Role updated.";
        } catch (error) {
          message.textContent = error.message || "Unable to update role.";
        }
      };
    });
  }

  loadUsers().catch((error) => {
    list.innerHTML = `<p>${escapeText(error.message || "Unable to load users.")}</p>`;
  });
})();
