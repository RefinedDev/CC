(function () {
  document.addEventListener("DOMContentLoaded", async function () {
    const id = new URLSearchParams(location.search).get("id");
    const scoreParam = new URLSearchParams(location.search).get("score");
    const totalParam = new URLSearchParams(location.search).get("total");
    if (!id || !window.CapacityApi) return;
    try {
      const assessment = await window.CapacityApi.get(`/assessments/${id}`);
      const result = scoreParam
        ? { score: Number(scoreParam), total: Number(totalParam) }
        : await window.CapacityApi.get(`/assessments/${id}/result`);
      document.getElementById("result-name").textContent =
        window.ThingAuth.getUser().name;
      document.getElementById("result-title").textContent =
        `${assessment.title} · ${assessment.subject}`;
      document.getElementById("result-score").textContent =
        `${result.score}/${result.total}`;
      document.getElementById("result-percent").textContent =
        `${Math.round((result.score / result.total) * 100)}%`;
      document.getElementById("result-message").textContent =
        result.score / result.total >= 0.8
          ? "Excellent work! You passed the assessment."
          : "Keep practicing and try again to improve your score.";
    } catch (error) {
      document.getElementById("result-message").textContent = error.message;
    }
  });
})();
