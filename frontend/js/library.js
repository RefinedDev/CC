(function () {
  'use strict';
  const list = document.getElementById('resource-list');
  const upload = document.getElementById('resource-upload');
  const user = window.ThingAuth.getUser();
  const escape = value => String(value).replace(/[&<>'"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', "'": '&#39;', '"': '&quot;' }[c]));
  const formatSize = bytes => bytes < 1024 * 1024 ? `${Math.max(1, Math.round(bytes / 1024))} KB` : `${(bytes / 1024 / 1024).toFixed(1)} MB`;
  function kind(file) { return file.type.includes('pdf') ? 'PDF' : file.type.includes('presentation') ? 'PPT' : file.type.startsWith('video/') ? 'Video' : 'Document'; }
  async function load() {
    try {
      const resources = await window.CapacityApi.get('/resources');
      list.innerHTML = resources.length ? resources.map(item => `<div class="resource-row"><span>📎 ${escape(item.name)}</span><span><i class="pill blue">${escape(item.kind)}</i></span><span>${formatSize(item.size_bytes)}</span><span><button class="download-resource" data-id="${item.id}" data-name="${escape(item.name)}">Download</button>${item.created_by === user.id ? ` <button class="delete-resource" data-id="${item.id}">Delete</button>` : ''}</span></div>`).join('') : '<p>No learning resources have been uploaded yet.</p>';
      list.querySelectorAll('.download-resource').forEach(button => button.onclick = async () => {
        const data = await window.CapacityApi.get(`/resources/${button.dataset.id}/download`);
        const binary = atob(data.content_base64); const bytes = Uint8Array.from(binary, character => character.charCodeAt(0));
        const blob = new Blob([bytes]); const link = document.createElement('a'); link.href = URL.createObjectURL(blob); link.download = data.name; link.click(); URL.revokeObjectURL(link.href);
      });
      list.querySelectorAll('.delete-resource').forEach(button => button.onclick = async () => { if (confirm('Delete this resource?')) { await window.CapacityApi.delete(`/resources/${button.dataset.id}`); load(); } });
    } catch (error) { list.innerHTML = `<p>${escape(error.message)}</p>`; }
  }
  document.addEventListener('DOMContentLoaded', () => {
    if (user && ['trainer', 'admin'].includes(String(user.role).toLowerCase())) upload.style.display = 'block';
    document.getElementById('resource-file').onchange = async event => {
      const file = event.target.files[0]; if (!file) return;
      if (file.size > 25 * 1024 * 1024) return window.showToast('Files must be 25 MB or smaller.');
      const reader = new FileReader(); reader.onload = async () => {
        try { await window.CapacityApi.post('/resources', { name: file.name, kind: kind(file), size_bytes: file.size, content_base64: String(reader.result).split(',')[1] }); window.showToast('Resource uploaded.'); event.target.value = ''; load(); }
        catch (error) { window.showToast(error.message); }
      }; reader.readAsDataURL(file);
    };
    load();
  });
})();
