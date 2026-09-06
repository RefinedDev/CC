(function () {
  'use strict';
  let assessment; let current = 0; const answers = {};
  const id = new URLSearchParams(location.search).get('id');
  const escape = value => String(value).replace(/[&<>'"]/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;',"'":'&#39;','"':'&quot;'}[c]));
  function render() {
    const question = assessment.questions[current];
    document.getElementById('qnum').textContent = current + 1;
    document.getElementById('totalq').textContent = assessment.questions.length;
    document.getElementById('question').textContent = question.prompt;
    document.getElementById('options').innerHTML = question.options.map((option, index) => {
      const letter = String.fromCharCode(65 + index);
      return `<label><input type="radio" name="answer" value="${letter}" ${answers[question.id] === letter ? 'checked' : ''}><span>${letter}. ${escape(option)}</span></label>`;
    }).join('');
    document.getElementById('qprogress').style.width = `${((current + 1) / assessment.questions.length) * 100}%`;
    document.querySelectorAll('#options input').forEach(input => input.onchange = () => { answers[question.id] = input.value; renderGrid(); });
    renderGrid();
  }
  function startTimer() {
    let remaining = Math.max(0, Number(assessment.duration_minutes) * 60);
    const timer = document.getElementById('timer');
    const tick = () => {
      timer.textContent = `${String(Math.floor(remaining / 60)).padStart(2, '0')}:${String(remaining % 60).padStart(2, '0')}`;
      if (remaining === 0) return window.submitAssessment();
      remaining -= 1;
      setTimeout(tick, 1000);
    };
    tick();
  }
  function renderGrid() {
    document.getElementById('qgrid').innerHTML = assessment.questions.map((question, index) =>
      `<button class="${index === current ? 'current' : ''} ${answers[question.id] ? 'answered' : ''}" onclick="window.goQuestion(${index})">${index + 1}</button>`).join('');
  }
  window.goQuestion = index => { current = index; render(); };
  window.prevQ = () => { if (current > 0) window.goQuestion(current - 1); };
  window.nextQ = () => { if (!answers[assessment.questions[current].id]) return window.showToast('Please select an answer first.'); if (current < assessment.questions.length - 1) window.goQuestion(current + 1); else window.submitAssessment(); };
  window.submitAssessment = async () => {
    if (Object.keys(answers).length !== assessment.questions.length && !confirm('Some questions are unanswered. Submit anyway?')) return;
    try {
      const result = await window.CapacityApi.post(`/assessments/${assessment.id}/attempt`, { answers });
      location.assign(`assessment-result.html?id=${assessment.id}&score=${result.score}&total=${result.total}`);
    } catch (error) { window.showToast(error.message); }
  };
  document.addEventListener('DOMContentLoaded', async () => {
    if (!id || !window.CapacityApi) return;
    try { assessment = await window.CapacityApi.get(`/assessments/${id}`); document.title = `${assessment.title} | Capacity Connect`; render(); startTimer(); }
    catch (error) { document.getElementById('question').textContent = error.message; }
  });
})();
