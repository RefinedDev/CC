(function () {
  'use strict';
  const user = window.ThingAuth.getUser();
  const list = document.getElementById('trainerCourses');
  if (!user || !list || !window.CapacityApi) return;

  const escapeText = value => String(value).replace(/[&<>'"]/g, character =>
    ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', "'": '&#39;', '"': '&quot;' }[character]));

  window.CapacityApi.get('/courses').then(courses => {
    const owned = courses.filter(course => course.created_by === user.id);
    document.getElementById('trainerCourseCount').textContent = owned.length;
    list.innerHTML = owned.length ? owned.map(course =>
      `<div>📘 <span>${escapeText(course.title)}<small>${escapeText(course.description)}</small></span></div>`
    ).join('') : '<p>No courses created yet. Use Manage Courses to create one.</p>';
  }).catch(error => {
    list.innerHTML = `<p>${escapeText(error.message || 'Unable to load your courses.')}</p>`;
  });
})();
