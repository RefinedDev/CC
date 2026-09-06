(function () {
  document.addEventListener('DOMContentLoaded', async () => {
    const list = document.getElementById('notification-list');
    if (!list || !window.CapacityApi) return;
    try {
      const items = await window.CapacityApi.get('/publishing');
      const escape = value => String(value).replace(/[&<>"']/g, char => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[char]));
      list.innerHTML = items.length ? items.map(item => `<div class="notification${item.read ? '' : ' unread'}" data-id="${item.id}"><div class="icon">${item.kind === 'achievement' ? '🏆' : item.kind === 'content' ? '📚' : '🔔'}</div><div><b>${escape(item.title)}</b><p>${escape(item.body)}</p><small>${escape(item.created_at)}</small></div></div>`).join('') : '<p>No updates have been published yet.</p>';
      list.querySelectorAll('.notification.unread').forEach(item => item.addEventListener('click', async () => {
        await window.CapacityApi.post(`/publishing/${item.dataset.id}/read`, {});
        item.classList.remove('unread');
      }));
      document.getElementById('mark-all-read')?.addEventListener('click', async () => {
        await window.CapacityApi.post('/publishing/read-all', {});
        list.querySelectorAll('.notification').forEach(item => item.classList.remove('unread'));
      });
    } catch (error) { list.innerHTML = `<p>${error.message}</p>`; }
  });
})();
