const theme = document.querySelector('#theme');
const systemDark = matchMedia('(prefers-color-scheme: dark)');
let preferredTheme;
try { preferredTheme = localStorage.getItem('theme'); } catch {}
function updateTheme() {
  const current = preferredTheme || (systemDark.matches ? 'dark' : 'light');
  document.documentElement.dataset.theme = current;
  theme.textContent = current === 'dark' ? 'Light' : 'Dark';
  theme.setAttribute('aria-label', `Switch to ${theme.textContent.toLowerCase()} mode`);
}
theme.addEventListener('click', () => {
  preferredTheme = document.documentElement.dataset.theme === 'dark' ? 'light' : 'dark';
  try { localStorage.setItem('theme', preferredTheme); } catch {}
  updateTheme();
});
systemDark.addEventListener('change', updateTheme);
updateTheme();
theme.hidden = false;

const canvas = document.querySelector('#life');
if (canvas) {
  const ctx = canvas.getContext('2d');
  const enabled = matchMedia('(min-width: 43.01rem) and (prefers-reduced-motion: no-preference)');
  const size = 26;
  let cells = [];
  let cols, rows, timer;

  function reset() {
    canvas.width = innerWidth;
    canvas.height = innerHeight;
    cols = Math.ceil(canvas.width / size);
    rows = Math.ceil(canvas.height / size);
    cells = Array.from({ length: cols * rows }, () => Math.random() < .19);
  }

  function frame() {
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    ctx.fillStyle = document.documentElement.dataset.theme === 'dark' ? '#354658' : '#cdd9e8';
    cells = cells.map((alive, index) => {
      const x = index % cols, y = Math.floor(index / cols);
      ctx.fillRect(x * size, y * size, 2, 2);
      if (alive) ctx.fillRect(x * size + 4, y * size + 4, size - 8, size - 8);
      let neighbors = 0;
      for (let dy = -1; dy <= 1; dy++) {
        for (let dx = -1; dx <= 1; dx++) {
          if ((dx || dy) && cells[((y + dy + rows) % rows) * cols + (x + dx + cols) % cols]) neighbors++;
        }
      }
      return neighbors === 3 || (alive && neighbors === 2);
    });
  }

  function sync() {
    clearInterval(timer);
    if (!ctx || !enabled.matches || document.hidden) return;
    reset();
    frame();
    timer = setInterval(frame, 600);
  }
  enabled.addEventListener('change', sync);
  document.addEventListener('visibilitychange', sync);
  addEventListener('resize', sync);
  sync();
}
