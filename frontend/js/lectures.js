(function () {
  "use strict";

  const courseId = new URLSearchParams(location.search).get("course");
  const lectureList = document.getElementById("lectureList");

  if (!courseId || !lectureList) return;

  let course;
  let lessons = [];
  const progress = { completedLectures: [], startedLectures: [] };

  function render() {
    const completed = progress.completedLectures;
    const started = progress.startedLectures;
    const total = lessons.length;
    const done = completed.length;
    const percent = Math.round((done / total) * 100);

    document.getElementById("courseTitle").textContent = course.title;
    document.getElementById("heroTitle").textContent = started.length
      ? "Continue your learning journey"
      : "Start your learning journey";
    document.getElementById("heroDescription").textContent = course.description;
    document.getElementById("courseCategory").textContent = "COURSE";
    document.getElementById("progressPercent").textContent = `${percent}%`;
    document.getElementById("progressCount").textContent =
      `${done} / ${total} completed`;
    document.getElementById("remainingCount").textContent =
      `${total - done} remaining`;
    document.getElementById("progressBar").style.width = `${percent}%`;
    document.getElementById("totalLectures").textContent = total;
    document.getElementById("completedCount").textContent = done;
    document.getElementById("remainingStat").textContent = total - done;
    document.getElementById("lessonBadge").textContent =
      `${total} LESSON${total === 1 ? "" : "S"}`;

    lectureList.innerHTML = lessons
      .map((lecture, index) => {
        const title = lecture.title;
        const isComplete = completed.includes(index);
        const isStarted = started.includes(index) && !isComplete;
        return `<div class="lecture-item ${isComplete ? "completed" : ""} ${isStarted ? "started" : ""}">
        <div class="lecture-number">01</div><div class="lecture-icon">▶</div>
        <div class="lecture-details"><h3>${title}</h3><p>Course introduction</p></div>
        <span class="lecture-status ${isComplete ? "completed" : "pending"}">${isComplete ? "✓ Completed" : isStarted ? "▶ In progress" : "○ Not completed"}</span>
        <button class="btn ${isComplete ? "btn-ghost" : "btn-primary"} lecture-action" onclick="watchLecture(${index})">${isComplete ? "Review" : isStarted ? "Continue" : "Start"}</button>
      </div>`;
      })
      .join("");
  }

  window.watchLecture = function (index) {
    progress.startedLectures = [
      ...new Set([...progress.startedLectures, index]),
    ];
    document.getElementById("watchBox").classList.add("active");
    document.getElementById("watchTitle").textContent = lessons[index].title;
    document.getElementById("completeBtn").onclick = function () {
      window.CapacityApi.post(
        `/courses/${encodeURIComponent(courseId)}/progress/${lessons[index].id}`,
      )
        .then(() => {
          progress.completedLectures = [
            ...new Set([...progress.completedLectures, index]),
          ];
          document.getElementById("watchBox").classList.remove("active");
          render();
        })
        .catch((error) => {
          document.getElementById("watchTitle").textContent =
            error.message || "Unable to save progress.";
        });
    };
    render();
  };

  document.getElementById("closeWatch").onclick = () =>
    document.getElementById("watchBox").classList.remove("active");

  Promise.all([
    window.CapacityApi.get(`/courses/${encodeURIComponent(courseId)}`),
    window.CapacityApi.get(`/courses/${encodeURIComponent(courseId)}/lectures`),
    window.CapacityApi.get(`/courses/${encodeURIComponent(courseId)}/progress`),
  ])
    .then(([result, courseLectures, courseProgress]) => {
      course = result;
      lessons = courseLectures;
      progress.completedLectures = courseProgress.completed_lectures
        .map((lectureId) =>
          lessons.findIndex((lecture) => lecture.id === lectureId),
        )
        .filter((index) => index >= 0);
      render();
    })
    .catch((error) => {
      lectureList.innerHTML = `<div class="panel"><p>${error.message || "Unable to load this course."}</p></div>`;
    });
})();
