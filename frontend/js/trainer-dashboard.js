(function () {
  'use strict';
  const user = window.ThingAuth.getUser();
  const list = document.getElementById('trainerCourses');
  if (!user || !list || !window.CapacityApi) return;

  const escapeText = value => String(value).replace(/[&<>'"]/g, character =>
    ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', "'": '&#39;', '"': '&quot;' }[character]));

  Promise.all([window.CapacityApi.get('/courses/analytics'), window.CapacityApi.get('/resources')]).then(([analytics, resources]) => {
    const courses = analytics.courses;
    document.getElementById('trainerCourseCount').textContent = courses.length;
    document.getElementById('trainerAssessmentCount').textContent = analytics.assessments.length;
    document.getElementById('trainerResourceCount').textContent = resources.filter(resource => resource.created_by === user.id).length;
    const scores = analytics.assessments.filter(item => item.attempts > 0);
    document.getElementById('trainerAverageScore').textContent = scores.length ? `${Math.round(scores.reduce((sum, item) => sum + item.average_score, 0) / scores.length)}%` : '—';
    list.innerHTML = courses.length ? courses.map(course =>
      `<div>📘 <span>${escapeText(course.title)}<small>${course.enrollments} trainees · ${course.completion_rate}% completion</small></span></div>`
    ).join('') : '<p>No courses created yet. Use Manage Courses to create one.</p>';
    const chart = document.getElementById('trainerCourseChart');
    chart.innerHTML = courses.length ? courses.map(course => `<i style="height:${Math.max(5, course.completion_rate)}%"><span>${escapeText(course.title.slice(0, 8))}</span></i>`).join('') : '<p>No course analytics available.</p>';
    const assessmentList = document.getElementById('trainerAssessmentAnalytics');
    assessmentList.innerHTML = analytics.assessments.length ? analytics.assessments.map(item =>
      `<div>📝 <span>${escapeText(item.title)}<small>${item.attempts} attempts · ${item.attempts ? `${item.average_score}% average` : 'No submissions yet'}</small></span></div>`
    ).join('') : '<p>No assessments created yet.</p>';
  }).catch(error => {
    list.innerHTML = `<p>${escapeText(error.message || 'Unable to load your courses.')}</p>`;
  });
})();
