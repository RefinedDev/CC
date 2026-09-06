(function () {
  "use strict";

  const form = document.getElementById("courseForm");
  const message = document.getElementById("courseMessage");
  if (!form || !message) return;
  const courseSelect = document.getElementById("managedCourse");
  const lectureForm = document.getElementById("lectureForm");
  const lectureList = document.getElementById("lectureList");
  const lectureMessage = document.getElementById("lectureMessage");
  const requirements = document.getElementById("courseRequirements");
  const requirementsMessage = document.getElementById("requirementsMessage");
  const saveRequirements = document.getElementById("saveRequirements");
  let selectedCourse;

  form.addEventListener("submit", async (event) => {
    event.preventDefault();
    const button = form.querySelector("button");
    button.disabled = true;
    message.textContent = "";

    try {
      await window.CapacityApi.post("/courses", {
        title: document.getElementById("courseTitle").value.trim(),
        description: document.getElementById("courseDescription").value.trim(),
      });
      form.reset();
      message.textContent = "Course created successfully.";
      await loadCourses();
    } catch (error) {
      message.textContent = error.message || "Unable to create course.";
    } finally {
      button.disabled = false;
    }
  });

  async function loadCourses() {
    const courses = await window.CapacityApi.get("/courses");
    courseSelect.innerHTML = courses
      .filter((course) => {
        const user = window.ThingAuth.getUser();
        return user && course.created_by === user.id;
      })
      .map((course) => `<option value="${course.id}">${course.title}</option>`)
      .join("");
    selectedCourse = courseSelect.value;
    await loadRequirements();
    await loadLectures();
  }

  async function loadRequirements() {
    if (!selectedCourse || !requirements) return;
    const data = await window.CapacityApi.get(
      `/competencies/courses/${encodeURIComponent(selectedCourse)}/requirements`,
    );
    requirements.value = data.skills || "";
  }

  async function loadLectures() {
    if (!selectedCourse) {
      lectureList.innerHTML = "<p>No courses created by your account yet.</p>";
      return;
    }
    const lectures = await window.CapacityApi.get(
      `/courses/${encodeURIComponent(selectedCourse)}/lectures`,
    );
    lectureList.innerHTML = lectures
      .map(
        (lecture) => `
      <div style="display:flex;justify-content:space-between;align-items:center;gap:12px">
        <span><strong>${escapeText(lecture.title)}</strong><small>${escapeText(lecture.description)}</small></span>
        <button class="btn btn-ghost" data-delete-lecture="${lecture.id}">Delete</button>
      </div>`,
      )
      .join("");
    lectureList.querySelectorAll("[data-delete-lecture]").forEach((button) => {
      button.onclick = async () => {
        await window.CapacityApi.delete(
          `/courses/${encodeURIComponent(selectedCourse)}/lectures/${button.dataset.deleteLecture}`,
        );
        await loadLectures();
      };
    });
  }

  function escapeText(value) {
    return String(value).replace(
      /[&<>'"]/g,
      (character) =>
        ({
          "&": "&amp;",
          "<": "&lt;",
          ">": "&gt;",
          "'": "&#39;",
          '"': "&quot;",
        })[character],
    );
  }

  courseSelect.addEventListener("change", () => {
    selectedCourse = courseSelect.value;
    Promise.all([loadRequirements(), loadLectures()]).catch((error) => {
      lectureMessage.textContent = error.message;
    });
  });
  saveRequirements.addEventListener("click", async () => {
    if (!selectedCourse) return;
    try {
      await window.CapacityApi.put(
        `/competencies/courses/${encodeURIComponent(selectedCourse)}/requirements`,
        { skills: requirements.value.trim() },
      );
      requirementsMessage.textContent = "Course requirements saved.";
    } catch (error) {
      requirementsMessage.textContent =
        error.message || "Unable to save course requirements.";
    }
  });
  lectureForm.addEventListener("submit", async (event) => {
    event.preventDefault();
    try {
      await window.CapacityApi.post(
        `/courses/${encodeURIComponent(selectedCourse)}/lectures`,
        {
          title: document.getElementById("lectureTitle").value.trim(),
          description: document
            .getElementById("lectureDescription")
            .value.trim(),
        },
      );
      lectureForm.reset();
      lectureMessage.textContent = "Lecture added.";
      await loadLectures();
    } catch (error) {
      lectureMessage.textContent = error.message || "Unable to add lecture.";
    }
  });
  loadCourses().catch((error) => {
    lectureMessage.textContent = error.message || "Unable to load courses.";
  });
})();
