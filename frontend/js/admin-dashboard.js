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
})();
