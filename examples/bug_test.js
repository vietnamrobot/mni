// Bug test case: SWC produces invalid JavaScript
function updateToggleIcon(theme) {
  const toggleButton = document.getElementById('theme-toggle');
  if (!toggleButton) return;

  const icon = toggleButton.querySelector('#theme-icon') || toggleButton;
  icon.textContent = theme === 'dark' ? '☀️' : '🌙';
}
