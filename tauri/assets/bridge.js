'use strict';
window.desktopWindow = { action(action) {
  if (action === 'paste') return;
  fetch('/api/window', { method: 'POST', headers: { 'X-Workbench-Token': sessionStorage.getItem('workbench-token') || location.hash.slice(1), 'Content-Type': 'application/json' }, body: JSON.stringify({ action }) });
} };
document.getElementById('desktop-titlebar').addEventListener('mousedown', event => {
  if (event.button === 0 && !event.target.closest('button')) window.desktopWindow.action('drag');
});
