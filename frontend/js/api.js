(function () {
  "use strict";

  const API_BASE = `${window.CAPACITY_API_BASE || "http://localhost:6969"}/api`;

  async function request(path, options = {}) {
    const response = await fetch(`${API_BASE}${path}`, {
      ...options,
      headers: {
        ...window.ThingAuth.apiHeaders(),
        ...(options.headers || {}),
      },
    });
    const text = await response.text();
    let data = {};
    if (text) {
      try {
        data = JSON.parse(text);
      } catch {
        data = { message: text };
      }
    }
    if (!response.ok) throw new Error(data.message || "Request failed.");
    return data;
  }

  window.CapacityApi = {
    get: (path) => request(path),
    post: (path, body) =>
      request(path, {
        method: "POST",
        body: JSON.stringify(body),
      }),
    put: (path, body) =>
      request(path, {
        method: "PUT",
        body: JSON.stringify(body),
      }),
    delete: (path) => request(path, { method: "DELETE" }),
  };
})();
