(function () {
  const escape = (value) =>
    String(value).replace(
      /[&<>'"]/g,
      (c) =>
        ({
          "&": "&amp;",
          "<": "&lt;",
          ">": "&gt;",
          "'": "&#39;",
          '"': "&quot;",
        })[c],
    );
  document.addEventListener("DOMContentLoaded", async () => {
    const box = document.getElementById("certificate-state");
    const list = document.getElementById("certificate-list");
    try {
      const user = window.ThingAuth.getUser();
      const courses = await window.CapacityApi.get("/courses");
      const enrolled = courses.filter((course) =>
        course.enrolled_users.includes(user.id),
      );
      const completed = [];
      for (const course of enrolled) {
        const [lectures, progress] = await Promise.all([
          window.CapacityApi.get(`/courses/${course.id}/lectures`),
          window.CapacityApi.get(`/courses/${course.id}/progress`),
        ]);
        if (
          lectures.length &&
          progress.completed_lectures.length === lectures.length
        )
          completed.push(course);
      }
      if (!completed.length) {
        box.innerHTML =
          '<div class="panel" style="text-align:center"><h2>No certificates yet</h2><p>Complete all lectures in an enrolled course to earn a certificate.</p></div>';
        return;
      }
      const date = new Date().toLocaleDateString();
      list.innerHTML = completed
        .map(
          (course) => `<article class="certificate">
        <div class="certificate-inner"><img src="assets/logo.svg" class="cert-logo">
          <p>CERTIFICATE</p><h2>OF COMPLETION</h2>
          <span>This is to certify that</span><h1>${escape(user.name)}</h1>
          <span>has successfully completed</span><h3>${escape(course.title)}</h3>
          <div class="medal">🏅</div><div class="cert-footer">
            <span>Date: ${date}</span><span>Capacity Connect</span>
          </div>
        </div>
      </article>`,
        )
        .join("");
    } catch (error) {
      box.innerHTML = `<div class="panel"><p>${escape(error.message)}</p></div>`;
    }
  });
})();
