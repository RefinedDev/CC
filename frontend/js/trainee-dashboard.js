(function () {
  'use strict';
  const user = window.ThingAuth.getUser();
  const target = document.getElementById('continueCourse');
  if (!user || !target || !window.CapacityApi) return;

  const escapeText = value => String(value).replace(/[&<>'"]/g, character =>
    ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', "'": '&#39;', '"': '&quot;' }[character]));

  window.CapacityApi.get('/courses').then(async courses => {
    const enrolled = courses.filter(course => Array.isArray(course.enrolled_users) && course.enrolled_users.includes(user.id));
    document.getElementById('traineeCourseCount').textContent = enrolled.length;
    if (!enrolled.length) {
      target.innerHTML = '<p>You are not enrolled in any courses yet. <a href="course-catalog.html">Browse courses</a>.</p>';
      return;
    }
    const course = enrolled[0];
    const [lectures, progress] = await Promise.all([
      window.CapacityApi.get(`/courses/${encodeURIComponent(course.id)}/lectures`),
      window.CapacityApi.get(`/courses/${encodeURIComponent(course.id)}/progress`)
    ]);
    const completed = progress.completed_lectures.length;
    const percent = lectures.length ? Math.round(completed / lectures.length * 100) : 0;
    target.innerHTML = `<div class="course-thumb">📘</div><div>
      <h3>${escapeText(course.title)}</h3><p>${completed} / ${lectures.length} lessons completed</p>
      <div class="progress"><i style="width:${percent}%"></i></div>
      <a class="btn btn-primary continue-course-btn" href="lectures.html?course=${encodeURIComponent(course.id)}">Continue</a>
    </div>`;
  }).catch(error => {
    target.innerHTML = `<p>${escapeText(error.message || 'Unable to load your courses.')}</p>`;
  });
})();
