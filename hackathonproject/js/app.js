/* The Thing - application UI. Authentication is handled only by auth.js. */
(function () {
  'use strict';
  const Auth = window.ThingAuth;
  if (!Auth) return;

  function escapeHTML(s) {
    return String(s).replace(/[&<>'"]/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;',"'":'&#39;','"':'&quot;'}[c]));
  }
  function showToast(msg) {
    const t = document.createElement('div'); t.className = 'toast'; t.textContent = msg;
    document.body.appendChild(t); setTimeout(() => t.remove(), 2600);
  }
  window.showToast = showToast;
  window.toggleSidebar = () => document.querySelector('.sidebar')?.classList.toggle('open');

  function setupShell() {
    Auth.applyTheme();
    const u = Auth.getUser();
    document.querySelectorAll('[data-user-name]').forEach(e => e.textContent = u ? u.name : 'User');
    document.querySelectorAll('[data-user-role]').forEach(e => e.textContent = u ? u.role : 'Guest');
    document.querySelectorAll('[data-user-avatar]').forEach(e => e.textContent = u ? u.name.trim().charAt(0).toUpperCase() : 'U');
    document.querySelectorAll('[data-logout]').forEach(e => e.onclick = Auth.logout);
    document.querySelectorAll('[data-menu]').forEach(e => e.onclick = window.toggleSidebar);
    document.querySelectorAll('[data-theme-toggle]').forEach(e => e.onchange = () => Auth.toggleTheme(e.checked));
    const current = Auth.currentPage();
    document.querySelectorAll('.side-links a').forEach(a => {
      if (a.getAttribute('href') === current) a.classList.add('active');
      if (u && a.dataset.role && a.dataset.role !== u.role) a.style.display = 'none';
    });
  }

  function initThemeButton() {
    if (document.querySelector('.floating-theme')) return;
    const b = document.createElement('button');
    b.className = 'floating-theme'; b.title = 'Toggle theme';
    Object.assign(b.style,{position:'fixed',right:'18px',bottom:'18px',width:'42px',height:'42px',borderRadius:'50%',border:'1px solid var(--border)',background:'var(--surface)',color:'var(--text)',zIndex:'50',boxShadow:'0 6px 18px rgba(0,0,0,.15)'});
    const sync = () => { b.innerHTML = localStorage.getItem(Auth.THEME_KEY) === 'dark' ? '☀' : '☾'; };
    b.onclick = () => { Auth.toggleTheme(localStorage.getItem(Auth.THEME_KEY) !== 'dark'); sync(); };
    document.body.appendChild(b); sync();
  }

  const questions = [
    {q:'Which HTML tag is used for the largest heading?',o:['<h1>','<heading>','<h6>','<head>'],a:'A'},
    {q:'Which CSS property changes the text color?',o:['font-color','color','text-color','foreground-color'],a:'B'},
    {q:'Which language is mainly used to add interactivity to web pages?',o:['SQL','Python','JavaScript','C++'],a:'C'},
    {q:'Which symbol starts an ID selector in CSS?',o:['.','#','@','*'],a:'B'},
    {q:'Which HTML element creates a hyperlink?',o:['<link>','<a>','<href>','<url>'],a:'B'},
    {q:'Which method converts a JSON string into a JavaScript object?',o:['JSON.parse()','JSON.stringify()','JSON.object()','JSON.convert()'],a:'A'},
    {q:'Which property controls the space inside an element?',o:['margin','padding','spacing','inside-space'],a:'B'},
    {q:'Which HTML element is used to display an image?',o:['<picture>','<src>','<img>','<image>'],a:'C'}
  ];
  let qi = Number(localStorage.getItem('thing_q') || 0);
  let answers = {};
  try { answers = JSON.parse(localStorage.getItem('thing_answers') || '{}') || {}; } catch (_) { answers = {}; }

  function renderQuestion() {
    const q = document.getElementById('question'); if (!q) return;
    const item = questions[qi];
    document.getElementById('qnum').textContent = qi + 1;
    document.getElementById('totalq').textContent = questions.length;
    q.textContent = item.q;
    document.getElementById('options').innerHTML = item.o.map((x,i) => {
      const letter = String.fromCharCode(65+i);
      return `<label><input type="radio" name="q" value="${letter}" ${answers[qi]===letter?'checked':''}><span>${letter}. ${escapeHTML(x)}</span></label>`;
    }).join('');
    document.getElementById('qprogress').style.width = ((qi+1)/questions.length*100)+'%';
    renderGrid();
    document.querySelectorAll('#options input').forEach(r => r.onchange = () => {
      answers[qi] = r.value; localStorage.setItem('thing_answers', JSON.stringify(answers)); renderGrid();
    });
  }
  function renderGrid() {
    const g = document.getElementById('qgrid'); if (!g) return;
    g.innerHTML = questions.map((_,i) => `<button class="${i===qi?'current':''} ${answers[i]?'answered':''}" onclick="goQuestion(${i})">${i+1}</button>`).join('');
  }
  window.goQuestion = i => { qi = i; localStorage.setItem('thing_q', qi); renderQuestion(); };
  window.nextQ = () => { if (!answers[qi]) return showToast('Please select an answer first.'); qi < questions.length-1 ? window.goQuestion(qi+1) : submitAssessment(); };
  window.prevQ = () => { if (qi > 0) window.goQuestion(qi-1); };
  function submitAssessment() {
    const score = questions.reduce((n,x,i) => n + (answers[i] === x.a ? 1 : 0), 0);
    localStorage.setItem('thing_result', JSON.stringify({score,total:questions.length,title:'Web Development Basics Assessment',date:new Date().toLocaleDateString()}));
    Auth.clearAssessmentState();
    location.assign('assessment-result.html');
  }
  window.submitAssessment = submitAssessment;

  function initAssessment() {
    if (!document.getElementById('question')) return;
    renderQuestion();
    let end = Number(localStorage.getItem('thing_assessment_end') || 0);
    if (!end) { end = Date.now()+30*60*1000; localStorage.setItem('thing_assessment_end', end); }
    const timer = document.getElementById('timer');
    const interval = setInterval(() => {
      const left = Math.max(0, Math.floor((end-Date.now())/1000));
      timer.textContent = String(Math.floor(left/60)).padStart(2,'0')+':'+String(left%60).padStart(2,'0');
      if (left===0) { clearInterval(interval); submitAssessment(); }
    },1000);
  }

  function initCertificate() {
    const u=Auth.getUser(), box=document.getElementById('certificate-state'); if(!u||!box)return;
    const p=Number(u.courseProgress||0);
    if(p<80){box.innerHTML=`<div class="panel" style="text-align:center"><div style="font-size:60px">🔒</div><h2>Certificate unavailable</h2><p>This course must be at least <strong>80% complete</strong> before a certificate can be issued.</p><div class="progress"><i style="width:${p}%"></i></div><p>Current progress: <strong>${p}%</strong></p><a class="btn btn-primary" href="course-catalog.html">Continue Course</a></div>`;return;}
    document.getElementById('certificate-template').style.display='block'; document.querySelector('[data-cert-name]').textContent=u.name;
  }

  function initResult() {
    const el=document.getElementById('result-score'); if(!el)return;
    const r=JSON.parse(localStorage.getItem('thing_result')||'null'), u=Auth.getUser();
    if(!r||!u){ location.assign(Auth.dashboardFor(u?.role||'Trainee')); return; }
    document.getElementById('result-name').textContent=u.name;
    document.getElementById('result-title').textContent=r.title;
    document.getElementById('result-score').textContent=`${r.score}/${r.total}`;
    document.getElementById('result-percent').textContent=Math.round(r.score/r.total*100)+'%';
    document.getElementById('result-message').textContent=r.score/r.total>=.8?'Excellent work! You passed the assessment.':'Keep practicing and try again to improve your score.';
  }

  window.saveProfile = function(){const u=Auth.getUser(),name=document.getElementById('profile-name')?.value.trim();if(u&&name){u.name=name;Auth.saveUser(u);setupShell();showToast('Profile saved!');}};

  document.addEventListener('DOMContentLoaded', () => {
    // auth.js has already performed the access check synchronously.
    if (!Auth.getUser() && !new Set(['index.html','login.html','signup.html']).has(Auth.currentPage())) return;
    setupShell(); initThemeButton(); initAssessment(); initCertificate(); initResult();
  });
})();
