(function () {
  'use strict';

  const params = new URLSearchParams(location.search);
  const courseId = params.get('course');
  const enrollCard = document.getElementById('enrollCard');
  const success = document.getElementById('success');
  const form = document.getElementById('enrollForm');

  if (!courseId || !enrollCard || !success || !form) return;

  function showError(message) {
    let error = document.getElementById('enrollmentError');
    if (!error) {
      error = document.createElement('p');
      error.id = 'enrollmentError';
      error.setAttribute('role', 'alert');
      error.style.color = 'var(--danger, #c0392b)';
      form.prepend(error);
    }
    error.textContent = message;
  }

  async function loadCourse() {
    try {
      const course = await window.CapacityApi.get(`/courses/${encodeURIComponent(courseId)}`);
      document.getElementById('courseTitle').textContent = course.title;
      document.getElementById('courseDescription').textContent = course.description;
      document.getElementById('successCourse').textContent = course.title;
      document.getElementById('startLearning').href = `lectures.html?course=${encodeURIComponent(course.id)}`;
      const user = window.ThingAuth.getUser();
      if (user && Array.isArray(course.enrolled_users) && course.enrolled_users.includes(user.id)) {
        enrollCard.style.display = 'none';
        success.style.display = 'block';
        document.querySelector('#success p').firstChild.textContent = 'You are already enrolled in ';
        form.remove();
      }
    } catch (error) {
      showError(error.message || 'Unable to load this course.');
      form.querySelector('button').disabled = true;
    }
  }

  form.addEventListener('submit', async event => {
    event.preventDefault();
    const button = form.querySelector('button');
    button.disabled = true;
    showError('');

    try {
      await window.CapacityApi.post(`/courses/${encodeURIComponent(courseId)}/enroll`);
      enrollCard.style.display = 'none';
      success.style.display = 'block';
    } catch (error) {
      showError(error.message || 'Unable to enroll in this course.');
      button.disabled = false;
    }
  });

  loadCourse();
})();
