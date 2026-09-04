(function () {
  'use strict';

  const courseGrid = document.getElementById('courseGrid');
  if (!courseGrid) return;

  const escapeText = value => String(value).replace(/[&<>'"]/g, character =>
    ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', "'": '&#39;', '"': '&quot;' }[character]));

  function render(courses) {
    if (!courses.length) {
      courseGrid.innerHTML = '<div class="panel"><p>No courses are available yet.</p></div>';
      return;
    }

    courseGrid.innerHTML = courses.map(course => `
      <article class="course-card">
        <div class="big-thumb blue">📘</div>
        <div>
          <span class="tag">Course</span>
          <h3>${escapeText(course.title)}</h3>
          <p>${escapeText(course.description)}</p>
          <a class="btn btn-primary course-action" href="enrollment.html?course=${encodeURIComponent(course.id)}">Enroll</a>
        </div>
      </article>
    `).join('');
  }

  async function loadCourses() {
    try {
      const courses = await window.CapacityApi.get('/courses');
      render(courses);
    } catch (error) {
      courseGrid.innerHTML = `<div class="panel"><p>${escapeText(error.message || 'Unable to load courses.')}</p></div>`;
    }
  }

  loadCourses();
})();
