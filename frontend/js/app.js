/* Capacity Connect - application UI. Authentication is handled only by auth.js. */
(function () {
  'use strict';
  const Auth = window.ThingAuth;
  if (!Auth) return;

  function escapeHTML(s) {
    return String(s).replace(/[&<>'"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', "'": '&#39;', '"': '&quot;' }[c]));
  }
  function showToast(msg) {
    const t = document.createElement('div'); t.className = 'toast'; t.textContent = msg;
    document.body.appendChild(t); setTimeout(() => t.remove(), 2600);
  }
  window.showToast = showToast;
  window.toggleSidebar = () => document.querySelector('.sidebar')?.classList.toggle('open');

  function setupShell() {
    Auth.applyTheme();
    const u = Auth.getUser();
    document.querySelectorAll('[data-user-name]').forEach(e => e.textContent = u ? u.name : 'User');
    document.querySelectorAll('[data-user-role]').forEach(e => e.textContent = u ? u.role : 'Guest');
    document.querySelectorAll('[data-user-avatar]').forEach(e => e.textContent = u ? u.name.trim().charAt(0).toUpperCase() : 'U');
    if (u) document.querySelectorAll('.brand').forEach(brand => brand.setAttribute('href', Auth.dashboardFor(u.role)));
    document.querySelectorAll('[data-logout]').forEach(e => e.onclick = Auth.logout);
    document.querySelectorAll('[data-menu]').forEach(e => e.onclick = window.toggleSidebar);
    document.querySelectorAll('[data-theme-toggle]').forEach(e => e.onchange = () => Auth.toggleTheme(e.checked));
    const current = Auth.currentPage();
    const pageRoles = {
      'trainee-dashboard.html': 'trainee',
      'trainer-dashboard.html': 'trainer',
      'admin-dashboard.html': 'admin',
      'course-management.html': 'trainer',
      'trainer-questionnaire.html': 'trainer',
      'performance.html': 'trainee',
      'feedback.html': 'trainee'
    };
    document.querySelectorAll('.side-links a').forEach(a => {
      if (a.getAttribute('href') === current) a.classList.add('active');
      const target = (a.getAttribute('href') || '').split('?')[0];
      const requiredRole = a.dataset.role || pageRoles[target];
      if (u && requiredRole && String(requiredRole).toLowerCase() !== String(u.role).toLowerCase()) {
        a.style.display = 'none';
      }
    });
  }

  function initThemeButton() {
    if (document.querySelector('.floating-theme')) return;
    const b = document.createElement('button');
    b.className = 'floating-theme'; b.title = 'Toggle theme';
    Object.assign(b.style, { position: 'fixed', right: '18px', bottom: '18px', width: '42px', height: '42px', borderRadius: '50%', border: '1px solid var(--border)', background: 'var(--surface)', color: 'var(--text)', zIndex: '50', boxShadow: '0 6px 18px rgba(0,0,0,.15)' });
    const sync = () => { b.innerHTML = localStorage.getItem(Auth.THEME_KEY) === 'dark' ? '☀' : '☾'; };
    b.onclick = () => { Auth.toggleTheme(localStorage.getItem(Auth.THEME_KEY) !== 'dark'); sync(); };
    document.body.appendChild(b); sync();
  }

  function initCertificate() {
    const u = Auth.getUser(), box = document.getElementById('certificate-state'); if (!u || !box) return;
    const p = Number(u.courseProgress || 0);
    if (p < 80) { box.innerHTML = `<div class="panel" style="text-align:center"><div style="font-size:60px">🔒</div><h2>Certificate unavailable</h2><p>This course must be at least <strong>80% complete</strong> before a certificate can be issued.</p><div class="progress"><i style="width:${p}%"></i></div><p>Current progress: <strong>${p}%</strong></p><a class="btn btn-primary" href="course-catalog.html">Continue Course</a></div>`; return; }
    document.getElementById('certificate-template').style.display = 'block'; document.querySelector('[data-cert-name]').textContent = u.name;
  }

  async function loadProfile() {
    const profileName = document.getElementById('profile-name');
    if (!profileName) return;

    try {
      const profile = await window.CapacityApi.get('/users/me');

      profileName.value = profile.name || '';
      const profileEmail = document.getElementById('profile-email');
      if (profileEmail) profileEmail.value = profile.email || '';
      Auth.saveUser({ ...Auth.getUser(), ...profile });
      setupShell();
    } catch (error) {
      showToast(error.message || 'Unable to load profile.');
    }
  }

  window.saveProfile = async function () {
    const name = document.getElementById('profile-name')?.value.trim();
    const email = document.getElementById('profile-email')?.value.trim();
    if (!name || !email) {
      showToast('Name and email are required.');
      return;
    }

    try {
      const profile = await window.CapacityApi.put('/users/me', { name, email });

      Auth.saveUser({ ...Auth.getUser(), ...profile });
      setupShell();
      showToast('Profile saved!');
    } catch (error) {
      showToast(error.message || 'Unable to save profile.');
    }
  };

  document.addEventListener('DOMContentLoaded', () => {
    // auth.js has already performed the access check synchronously.
    if (!Auth.getUser() && !new Set(['index.html', 'login.html', 'signup.html']).has(Auth.currentPage())) return;
    setupShell(); initThemeButton(); initCertificate(); loadProfile();
  });
})();
