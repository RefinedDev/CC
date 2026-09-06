(function () {
  document.addEventListener("DOMContentLoaded", function () {
    const form = document.getElementById("assessment-form");
    const questions = document.getElementById("questions");
    const template = document.getElementById("question-template");
    if (!form || !questions || !template) return;
    const list = document.getElementById("trainer-assessment-list");
    const user = window.ThingAuth.getUser();
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
    async function loadAssessments() {
      if (!list) return;
      try {
        const assessments = await window.CapacityApi.get("/assessments");
        const owned = assessments.filter((item) => item.created_by === user.id);
        list.innerHTML = owned.length
          ? owned
              .map(
                (item) =>
                  `<div><span><strong>${escape(item.title)}</strong><small>${escape(item.subject)} · Due ${escape(item.deadline)}</small></span><button class="btn btn-ghost delete-assessment" data-id="${item.id}">Delete</button></div>`,
              )
              .join("")
          : "<p>You have not created any assessments yet.</p>";
        list.querySelectorAll(".delete-assessment").forEach(
          (button) =>
            (button.onclick = async () => {
              if (!confirm("Delete this assessment and its saved attempts?"))
                return;
              try {
                await window.CapacityApi.delete(
                  `/assessments/${button.dataset.id}`,
                );
                await loadAssessments();
                window.showToast("Assessment deleted.");
              } catch (error) {
                window.showToast(error.message);
              }
            }),
        );
      } catch (error) {
        list.innerHTML = `<p>${escape(error.message)}</p>`;
      }
    }
    function addQuestion() {
      const node = template.content.cloneNode(true);
      node.querySelector(".question-number").textContent =
        questions.children.length + 1;
      questions.appendChild(node);
    }
    document.getElementById("add-question").onclick = addQuestion;
    addQuestion();
    form.onsubmit = async (event) => {
      event.preventDefault();
      const payload = {
        title: document.getElementById("assessment-title").value,
        subject: document.getElementById("assessment-subject").value,
        deadline: document.getElementById("assessment-deadline").value,
        duration_minutes: Number(
          document.getElementById("assessment-duration").value,
        ),
        questions: [...questions.children].map((node) => ({
          prompt: node.querySelector(".question-prompt").value,
          options: [...node.querySelectorAll(".option")].map(
            (input) => input.value,
          ),
          correct_option: node.querySelector(".correct-option").value,
        })),
      };
      try {
        await window.CapacityApi.post("/assessments", payload);
        window.showToast("Assessment saved.");
        form.reset();
        questions.innerHTML = "";
        addQuestion();
        await loadAssessments();
      } catch (error) {
        window.showToast(error.message);
      }
    };
    loadAssessments();
  });
})();
