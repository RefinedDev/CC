(function () {
  'use strict';
  const user = window.ThingAuth.getUser();
  const target = document.getElementById('continueCourse');
  if (!user || !target || !window.CapacityApi) return;

  const escapeText = value => String(value).replace(/[&<>'"]/g, character =>
    ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', "'": '&#39;', '"': '&quot;' }[character]));

  Promise.all([
    window.CapacityApi.get('/courses'),
    window.CapacityApi.get('/resources'),
    window.CapacityApi.get('/assessments'),
    window.CapacityApi.get('/publishing')
  ]).then(async ([courses, resources, assessments, publications]) => {
    const enrolled = courses.filter(course => Array.isArray(course.enrolled_users) && course.enrolled_users.includes(user.id));
    document.getElementById('traineeCourseCount').textContent = enrolled.length;
    const assessmentResults = assessments.filter(item => item.result);
    const pendingAssessments = Math.max(0, assessments.length - assessmentResults.length);
    document.getElementById('traineeAssessmentCount').textContent = assessments.length;
    document.getElementById('traineeAssessmentDetail').textContent = `${pendingAssessments} pending`;
    const resourceList = document.getElementById('traineeResources');
    resourceList.innerHTML = resources.length ? resources.slice(0, 3).map(resource =>
      `<div>📎 <span>${escapeText(resource.name)}<small>${escapeText(resource.kind)}</small></span></div>`
    ).join('') : '<p>No resources uploaded yet. <a href="library.html">Browse the library</a>.</p>';
    document.getElementById('traineeActivity').innerHTML = publications.length
      ? publications.slice(0, 4).map(item => `<div>🔔 <span>${escapeText(item.title)}<small>${escapeText(item.kind)} · ${escapeText(item.created_at)}</small></span></div>`).join('')
      : '<p>No recent activity yet.</p>';
    const courseData = await Promise.all(enrolled.map(async course => {
      const [lectures, progress] = await Promise.all([
        window.CapacityApi.get(`/courses/${encodeURIComponent(course.id)}/lectures`),
        window.CapacityApi.get(`/courses/${encodeURIComponent(course.id)}/progress`)
      ]);
      const completed = progress.completed_lectures.length;
      return { course, lectures, completed, percent: lectures.length ? Math.round(completed / lectures.length * 100) : 0 };
    }));
    const completedCourses = courseData.filter(item => item.lectures.length > 0 && item.completed === item.lectures.length);
    const lessonsCompleted = courseData.reduce((total, item) => total + item.completed, 0);
    document.getElementById('traineeCertificateCount').textContent = completedCourses.length;
    document.getElementById('traineeLessonCount').textContent = lessonsCompleted;
    const achievements = document.getElementById('traineeAchievements');
    const earned = [];
    if (assessmentResults.length >= 5) earned.push(['⭐', 'Quick Learner', `Completed ${assessmentResults.length} assessments`]);
    if (completedCourses.length > 0) earned.push(['🏆', 'Course Finisher', `Completed ${completedCourses.length} course${completedCourses.length === 1 ? '' : 's'}`]);
    const highScores = assessmentResults.filter(item => item.result.score / item.result.total >= 0.9).length;
    if (highScores >= 3) earned.push(['🎯', 'Top Performer', `Scored 90%+ in ${highScores} assessments`]);
    achievements.innerHTML = earned.length ? earned.map(item =>
      `<div>${item[0]} <b>${escapeText(item[1])}</b><small>${escapeText(item[2])}</small></div>`
    ).join('') : '<p>Complete courses and assessments to earn achievements.</p>';
    if (!courseData.length) {
      target.innerHTML = '<p>You are not enrolled in any courses yet. <a href="course-catalog.html">Browse courses</a>.</p>';
      return;
    }
    const current = courseData.find(item => item.percent < 100) || courseData[0];
    const { course, lectures, completed, percent } = current;
    target.innerHTML = `<div class="course-thumb">📘</div><div>
      <h3>${escapeText(course.title)}</h3><p>${completed} / ${lectures.length} lessons completed</p>
      <div class="progress"><i style="width:${percent}%"></i></div>
      <a class="btn btn-primary continue-course-btn" href="lectures.html?course=${encodeURIComponent(course.id)}">Continue</a>
    </div>`;
  }).catch(error => {
    target.innerHTML = `<p>${escapeText(error.message || 'Unable to load your courses.')}</p>`;
  });
})();
