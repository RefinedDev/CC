/* Capacity Connect - single, deterministic authentication layer.
   IMPORTANT: this file is loaded BEFORE app.js. */
(function () {
  "use strict";

  const USER_KEY = "thing_user";
  const TOKEN_KEY = "thing_token";
  const THEME_KEY = "thing_theme";
  window.CAPACITY_API_BASE =
    window.CAPACITY_API_BASE || "http://localhost:6969";
  const API_BASE = `${window.CAPACITY_API_BASE}/api`;

  const PUBLIC_PAGES = new Set(["", "index.html", "login.html", "signup.html"]);
  const PROTECTED_PAGES = new Set([
    "trainee-dashboard.html",
    "trainer-dashboard.html",
    "admin-dashboard.html",
    "course-catalog.html",
    "assessment-instructions.html",
    "assessment.html",
    "assessment-result.html",
    "performance.html",
    "library.html",
    "certificate.html",
    "notifications.html",
    "profile.html",
    "settings.html",
    "trainer-questionnaire.html",
    "course-management.html",
    "lectures.html",
    "enrollment.html",
  ]);

  function getUser() {
    try {
      const raw = localStorage.getItem(USER_KEY);
      if (!raw) return null;
      const user = JSON.parse(raw);
      return user && user.name && user.role ? user : null;
    } catch (_) {
      return null;
    }
  }

  function saveUser(user) {
    localStorage.setItem(USER_KEY, JSON.stringify(user));
  }
  function getToken() {
    return localStorage.getItem(TOKEN_KEY);
  }
  function apiHeaders() {
    const headers = { "Content-Type": "application/json" };
    const token = getToken();
    if (token) headers.Authorization = `Bearer ${token}`;
    return headers;
  }
  function clearAssessmentState() {
    ["thing_q", "thing_answers", "thing_assessment_end"].forEach((k) =>
      localStorage.removeItem(k),
    );
  }
  function currentPage() {
    const path = location.pathname.split("/").pop();
    return path || "index.html";
  }
  function dashboardFor(role) {
    if (String(role).toLowerCase() === "admin") return "admin-dashboard.html";
    if (String(role).toLowerCase() === "trainer")
      return "trainer-dashboard.html";
    return "trainee-dashboard.html";
  }

  function go(page) {
    location.assign(page);
  }

  function requireAuth() {
    const user = getUser();
    if (user) return true;
    // Never use a "next" parameter. It was the source of the old navigation trap.
    go("login.html");
    return false;
  }

  function enforceAccess() {
    const page = currentPage();
    if (PUBLIC_PAGES.has(page)) return true;
    if (!PROTECTED_PAGES.has(page)) return true;

    const user = getUser();
    if (!user) {
      go("login.html");
      return false;
    }

    const requiredRole = document.body.dataset.role;
    if (
      requiredRole &&
      String(user.role).toLowerCase() !== String(requiredRole).toLowerCase()
    ) {
      go(dashboardFor(user.role));
      return false;
    }
    return true;
  }

  async function login(event) {
    if (event) event.preventDefault();
    const email = (document.getElementById("login-email")?.value || "").trim();
    const password = document.getElementById("login-password")?.value || "";
    if (!email || !password) {
      alert("Please enter your email and password.");
      return false;
    }

    try {
      const response = await fetch(`${API_BASE}/auth/login`, {
        method: "POST",
        headers: apiHeaders(),
        body: JSON.stringify({ email, password }),
      });
      const result = await response.json();
      if (!response.ok) throw new Error(result.message || "Login failed.");
      localStorage.setItem(TOKEN_KEY, result.token);
      saveUser({ ...result.user, courseProgress: 65 });
      go(dashboardFor(result.user.role));
    } catch (error) {
      alert(error.message || "Unable to connect to the server.");
    }
    return false;
  }

  async function signup(event) {
    if (event) event.preventDefault();
    const name = (document.getElementById("signup-name")?.value || "").trim();
    const email = (document.getElementById("signup-email")?.value || "").trim();
    const password = document.getElementById("signup-password")?.value || "";
    if (!name || !email || !password) {
      alert("Please complete all required fields.");
      return false;
    }

    try {
      const response = await fetch(`${API_BASE}/auth/signup`, {
        method: "POST",
        headers: apiHeaders(),
        body: JSON.stringify({ name, email, password }),
      });
      const result = await response.json();
      if (!response.ok) throw new Error(result.message || "Signup failed.");
      localStorage.setItem(TOKEN_KEY, result.token);
      saveUser({ ...result.user, courseProgress: 65 });
      go(dashboardFor(result.user.role));
    } catch (error) {
      alert(error.message || "Unable to connect to the server.");
    }
    return false;
  }

  function logout(event) {
    if (event) event.preventDefault();
    localStorage.removeItem(USER_KEY);
    localStorage.removeItem(TOKEN_KEY);
    clearAssessmentState();
    localStorage.removeItem("thing_result");
    go("index.html");
    return false;
  }

  function applyTheme() {
    document.body.classList.toggle(
      "dark",
      localStorage.getItem(THEME_KEY) === "dark",
    );
    document.querySelectorAll("[data-theme-toggle]").forEach((x) => {
      x.checked = localStorage.getItem(THEME_KEY) === "dark";
    });
  }
  function toggleTheme(dark) {
    localStorage.setItem(THEME_KEY, dark ? "dark" : "light");
    applyTheme();
  }

  window.ThingAuth = {
    getUser,
    saveUser,
    getToken,
    apiHeaders,
    clearAssessmentState,
    currentPage,
    dashboardFor,
    requireAuth,
    login,
    signup,
    logout,
    applyTheme,
    toggleTheme,
    USER_KEY,
    TOKEN_KEY,
    THEME_KEY,
  };
  // Backward-compatible globals used by the existing HTML.
  window.getUser = getUser;
  window.saveUser = saveUser;
  window.requireAuth = requireAuth;
  window.login = login;
  window.signup = signup;
  window.logout = logout;
  window.toggleTheme = toggleTheme;
  window.applyTheme = applyTheme;

  // Run the access check immediately. No DOMContentLoaded redirect and no next= parameter.
  enforceAccess();
})();
