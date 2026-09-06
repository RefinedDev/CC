(function () {
  'use strict';
  const user = window.ThingAuth.getUser();
  const target = document.getElementById('continueCourse');
  if (!user || !target || !window.CapacityApi) return;

  const escapeText = value => String(value).replace(/[&<>'"]/g, character =>
    ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', "'": '&#39;', '"': '&quot;' }[character]));

  Promise.all([window.CapacityApi.get('/courses'), window.CapacityApi.get('/resources')]).then(async ([courses, resources]) => {
    const enrolled = courses.filter(course => Array.isArray(course.enrolled_users) && course.enrolled_users.includes(user.id));
    document.getElementById('traineeCourseCount').textContent = enrolled.length;
    const resourceList = document.getElementById('traineeResources');
    resourceList.innerHTML = resources.length ? resources.slice(0, 3).map(resource =>
      `<div>📎 <span>${escapeText(resource.name)}<small>${escapeText(resource.kind)}</small></span></div>`
    ).join('') : '<p>No resources uploaded yet. <a href="library.html">Browse the library</a>.</p>';
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
