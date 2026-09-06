(function () {
  'use strict';

  if (!window.CapacityApi || !window.CapacityApi.get) {
    document.getElementById('userDistribution').textContent = 'Unavailable';
    document.getElementById('distributionLegend').textContent = 'API client failed to load. Refresh the page.';
    return;
  }

  const percent = (value, total) => total ? Math.round(value / total * 100) : 0;

  window.CapacityApi.get('/admin/stats').then(stats => {
    document.getElementById('statUsers').textContent = stats.users;
    document.getElementById('statCourses').textContent = stats.courses;
    document.getElementById('statEnrollments').textContent = stats.enrollments;
    document.getElementById('statCertificates').textContent = stats.certificates;
    document.getElementById('userDistribution').textContent = `${percent(stats.trainees, stats.users)}%`;
    document.getElementById('distributionLegend').innerHTML =
      `🔵 Trainee ${percent(stats.trainees, stats.users)}%<br>` +
      `🟣 Trainer ${percent(stats.trainers, stats.users)}%<br>` +
      `🟢 Admin ${percent(stats.admins, stats.users)}%`;
  }).catch(error => {
    document.getElementById('userDistribution').textContent = '—';
    document.getElementById('distributionLegend').textContent = error.message || 'Unable to load statistics.';
  });

  const form = document.getElementById('publish-form');
  window.CapacityApi.get('/admin/users').then(users => {
    const target = document.getElementById('publish-target');
    users.forEach(user => target.insertAdjacentHTML('beforeend', `<option value="${user.id}">${user.name} (${user.email})</option>`));
  }).catch(error => {
    const target = document.getElementById('publish-target');
    if (target) target.disabled = true;
    const message = document.getElementById('publish-message');
    if (message) message.textContent = error.message || 'Targeted notifications are unavailable.';
  });
  if (form) form.onsubmit = async event => {
    event.preventDefault();
    const message = document.getElementById('publish-message');
    try {
      await window.CapacityApi.post('/publishing', {
        kind: document.getElementById('publish-kind').value,
        target_user_id: document.getElementById('publish-target').value || null,
        title: document.getElementById('publish-title').value,
        body: document.getElementById('publish-body').value
      });
      form.reset();
      message.textContent = 'Content published.';
    } catch (error) { message.textContent = error.message; }
  };
})();
