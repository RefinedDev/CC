/* The Thing - single, deterministic authentication layer.
   IMPORTANT: this file is loaded BEFORE app.js. */
(function () {
  'use strict';

  const USER_KEY = 'thing_user';
  const THEME_KEY = 'thing_theme';

  const PUBLIC_PAGES = new Set(['', 'index.html', 'login.html', 'signup.html']);
  const PROTECTED_PAGES = new Set([
    'trainee-dashboard.html', 'trainer-dashboard.html', 'admin-dashboard.html',
    'course-catalog.html', 'assessment-instructions.html', 'assessment.html',
    'assessment-result.html', 'performance.html', 'library.html', 'certificate.html',
    'feedback.html', 'notifications.html', 'profile.html', 'settings.html',
    'trainer-questionnaire.html', 'lectures.html', 'enrollment.html'
  ]);

  function getUser() {
    try {
      const raw = localStorage.getItem(USER_KEY);
      if (!raw) return null;
      const user = JSON.parse(raw);
      return user && user.name && user.role ? user : null;
    } catch (_) { return null; }
  }

  function saveUser(user) { localStorage.setItem(USER_KEY, JSON.stringify(user)); }
  function clearAssessmentState() {
    ['thing_q', 'thing_answers', 'thing_assessment_end'].forEach(k => localStorage.removeItem(k));
  }
  function currentPage() {
    const path = location.pathname.split('/').pop();
    return path || 'index.html';
  }
  function dashboardFor(role) {
    if (role === 'Admin') return 'admin-dashboard.html';
    if (role === 'Trainer') return 'trainer-dashboard.html';
    return 'trainee-dashboard.html';
  }

  function go(page) { location.assign(page); }

  function requireAuth() {
    const user = getUser();
    if (user) return true;
    // Never use a "next" parameter. It was the source of the old navigation trap.
    go('login.html');
    return false;
  }

  function enforceAccess() {
    const page = currentPage();
    if (PUBLIC_PAGES.has(page)) return true;
    if (!PROTECTED_PAGES.has(page)) return true;

    const user = getUser();
    if (!user) {
      go('login.html');
      return false;
    }

    const requiredRole = document.body.dataset.role;
    if (requiredRole && user.role !== requiredRole) {
      go(dashboardFor(user.role));
      return false;
    }
    return true;
  }

  function login(event) {
    if (event) event.preventDefault();
    const name = (document.getElementById('login-name')?.value || '').trim();
    const role = document.getElementById('login-role')?.value || 'Trainee';
    if (!name) {
      alert('Please enter your name.');
      return false;
    }

    // Remove any stale data from a previous assessment/session.
    clearAssessmentState();
    localStorage.removeItem('thing_result');

    saveUser({ name, role, courseProgress: 65 });
    // Login has exactly ONE destination: the selected role dashboard.
    go(dashboardFor(role));
    return false;
  }

  function signup(event) {
    if (event) event.preventDefault();
    const name = (document.getElementById('signup-name')?.value || '').trim();
    const role = document.getElementById('signup-role')?.value || 'Trainee';
    if (!name) {
      alert('Please enter your name.');
      return false;
    }

    clearAssessmentState();
    localStorage.removeItem('thing_result');
    saveUser({ name, role, courseProgress: 65 });
    go(dashboardFor(role));
    return false;
  }

  function logout(event) {
    if (event) event.preventDefault();
    localStorage.removeItem(USER_KEY);
    clearAssessmentState();
    localStorage.removeItem('thing_result');
    go('index.html');
    return false;
  }

  function applyTheme() {
    document.body.classList.toggle('dark', localStorage.getItem(THEME_KEY) === 'dark');
    document.querySelectorAll('[data-theme-toggle]').forEach(x => {
      x.checked = localStorage.getItem(THEME_KEY) === 'dark';
    });
  }
  function toggleTheme(dark) {
    localStorage.setItem(THEME_KEY, dark ? 'dark' : 'light');
    applyTheme();
  }

  window.ThingAuth = { getUser, saveUser, clearAssessmentState, currentPage, dashboardFor, requireAuth, enforceAccess, login, signup, logout, applyTheme, toggleTheme, USER_KEY, THEME_KEY };
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
