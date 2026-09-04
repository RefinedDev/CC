(function () {
  'use strict';

  const courseId = new URLSearchParams(location.search).get('course');
  const progressKey = `thing_course_progress_${courseId}`;
  const lectureList = document.getElementById('lectureList');

  if (!courseId || !lectureList) return;

  let course;
  let progress = JSON.parse(localStorage.getItem(progressKey) || '{"completedLectures":[],"startedLectures":[]}');
  progress.completedLectures = progress.completedLectures || [];
  progress.startedLectures = progress.startedLectures || [];

  function save() {
    localStorage.setItem(progressKey, JSON.stringify(progress));
  }

  function render() {
    const lessons = [`${course.title} Overview`];
    const completed = progress.completedLectures;
    const started = progress.startedLectures;
    const total = lessons.length;
    const done = completed.length;
    const percent = Math.round(done / total * 100);

    document.getElementById('courseTitle').textContent = course.title;
    document.getElementById('heroTitle').textContent = started.length ? 'Continue your learning journey' : 'Start your learning journey';
    document.getElementById('heroDescription').textContent = course.description;
    document.getElementById('courseCategory').textContent = 'COURSE';
    document.getElementById('progressPercent').textContent = `${percent}%`;
    document.getElementById('progressCount').textContent = `${done} / ${total} completed`;
    document.getElementById('remainingCount').textContent = `${total - done} remaining`;
    document.getElementById('progressBar').style.width = `${percent}%`;
    document.getElementById('totalLectures').textContent = total;
    document.getElementById('completedCount').textContent = done;
    document.getElementById('remainingStat').textContent = total - done;
    document.getElementById('lessonBadge').textContent = `${total} LESSON`;

    lectureList.innerHTML = lessons.map((title, index) => {
      const isComplete = completed.includes(index);
      const isStarted = started.includes(index) && !isComplete;
      return `<div class="lecture-item ${isComplete ? 'completed' : ''} ${isStarted ? 'started' : ''}">
        <div class="lecture-number">01</div><div class="lecture-icon">▶</div>
        <div class="lecture-details"><h3>${title}</h3><p>Course introduction</p></div>
        <span class="lecture-status ${isComplete ? 'completed' : 'pending'}">${isComplete ? '✓ Completed' : isStarted ? '▶ In progress' : '○ Not completed'}</span>
        <button class="btn ${isComplete ? 'btn-ghost' : 'btn-primary'} lecture-action" onclick="watchLecture(0)">${isComplete ? 'Review' : isStarted ? 'Continue' : 'Start'}</button>
      </div>`;
    }).join('');
  }

  window.watchLecture = function (index) {
    progress.startedLectures = [...new Set([...progress.startedLectures, index])];
    save();
    document.getElementById('watchBox').classList.add('active');
    document.getElementById('watchTitle').textContent = `${course.title} Overview`;
    document.getElementById('completeBtn').onclick = function () {
      progress.completedLectures = [...new Set([...progress.completedLectures, index])];
      save();
      document.getElementById('watchBox').classList.remove('active');
      render();
    };
    render();
  };

  document.getElementById('closeWatch').onclick = () => document.getElementById('watchBox').classList.remove('active');

  window.CapacityApi.get(`/courses/${encodeURIComponent(courseId)}`)
    .then(result => {
      course = result;
      render();
    })
    .catch(error => {
      lectureList.innerHTML = `<div class="panel"><p>${error.message || 'Unable to load this course.'}</p></div>`;
    });
})();
