(function () {
  const escape = value => String(value).replace(/[&<>'"]/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;',"'":'&#39;','"':'&quot;'}[c]));
  document.addEventListener('DOMContentLoaded', async () => {
    try {
      const [assessments, courses] = await Promise.all([window.CapacityApi.get('/assessments'), window.CapacityApi.get('/courses')]);
      const results = assessments.filter(item => item.result).map(item => ({ ...item, percent: Math.round(item.result.score / item.result.total * 100) }))
        .sort((a, b) => String(a.result.submitted_at).localeCompare(String(b.result.submitted_at)));
      const average = results.length ? Math.round(results.reduce((sum, item) => sum + item.percent, 0) / results.length) : 0;
      const best = results.reduce((top, item) => !top || item.percent > top.percent ? item : top, null);
      document.getElementById('assessment-count').textContent = results.length;
      document.getElementById('average-score').textContent = `${average}%`;
      document.getElementById('pass-rate').textContent = results.length ? `${Math.round(results.filter(item => item.percent >= 80).length / results.length * 100)}%` : '—';
      document.getElementById('best-score').textContent = best ? `${best.percent}%` : '—';
      document.getElementById('best-assessment').textContent = best ? best.title : 'No results yet';
      document.getElementById('overall-score').firstChild.textContent = average;
      document.getElementById('overall-message').textContent = results.length ? (average >= 80 ? 'Good progress!' : 'Keep practicing') : 'No results yet';
      document.getElementById('overall-detail').textContent = results.length ? 'Based on your submitted assessments.' : 'Complete an assessment to see performance.';
      document.getElementById('assessment-bars').innerHTML = results.length ? results.slice(-6).map(item => `<div class="pbar"><i style="height:${item.percent}%"></i><b>${item.percent}%</b><small>${escape(item.subject)}</small></div>`).join('') : '<p>No assessment results yet.</p>';
      document.getElementById('assessment-history').innerHTML = '<div class="p-row head"><span>Assessment</span><span>Submitted</span><span>Score</span><span>Status</span></div>' + (results.length ? results.map(item => `<div class="p-row"><span>${escape(item.title)}</span><span>${escape(item.result.submitted_at)}</span><span><b>${item.percent}%</b></span><span class="status ${item.percent >= 80 ? 'pass' : 'improve'}">${item.percent >= 80 ? 'Passed' : 'Improve'}</span></div>`).join('') : '<p>No assessment history yet.</p>');
      const trendValues = results.slice(-7).map(item => item.percent);
      const labels = results.slice(-7).map(item => escape(item.subject || item.title));
      const trendLabels = document.getElementById('trend-labels');
      if (trendValues.length) {
        const points = trendValues.map((value, index) => {
          const x = trendValues.length === 1 ? 300 : index * 600 / (trendValues.length - 1);
          return `${x},${220 - value * 1.8}`;
        });
        const line = points.join(' ');
        document.getElementById('trend-line').setAttribute('points', line);
        document.getElementById('trend-area').setAttribute('d', `M${line} L600 220 L0 220 Z`);
        const last = points[points.length - 1].split(',');
        document.getElementById('trend-point').setAttribute('cx', last[0]);
        document.getElementById('trend-point').setAttribute('cy', last[1]);
        trendLabels.innerHTML = labels.map(label => `<span>${label}</span>`).join('');
      } else {
        document.getElementById('trend-line').setAttribute('points', '');
        document.getElementById('trend-area').setAttribute('d', '');
        document.getElementById('trend-point').setAttribute('cx', '0');
        document.getElementById('trend-point').setAttribute('cy', '220');
        trendLabels.innerHTML = '<span>No results yet</span>';
      }
      const enrolled = courses.filter(course => Array.isArray(course.enrolled_users) && course.enrolled_users.includes(window.ThingAuth.getUser().id));
      let completed = 0; let total = 0;
      for (const course of enrolled) { const [lectures, progress] = await Promise.all([window.CapacityApi.get(`/courses/${encodeURIComponent(course.id)}/lectures`), window.CapacityApi.get(`/courses/${encodeURIComponent(course.id)}/progress`)]); total += lectures.length; completed += progress.completed_lectures.length; }
      const completion = total ? Math.round(completed / total * 100) : 0;
      document.querySelector('#course-completion strong').textContent = `${completion}%`;
      document.getElementById('course-completion-detail').innerHTML = `<span><i class="legend-dot done"></i>Completed <b>${completed}</b></span><span><i class="legend-dot remaining"></i>Remaining <b>${Math.max(0, total - completed)}</b></span>`;
    } catch (error) { document.getElementById('assessment-history').innerHTML = `<p>${escape(error.message)}</p>`; }
  });
})();
