const apiOrigin = ['localhost', '127.0.0.1'].includes(location.hostname) ? '' : 'https://garden-api.kguo.dev';
const list = document.querySelector('#bookmarks');
const search = document.querySelector('#search');
const filters = document.querySelector('#collection');
const clearSearch = document.querySelector('#clear-search');
let collection = '';
const empty = document.querySelector('#empty');
const owner = document.querySelector('#owner');
const login = document.querySelector('#login');
const add = document.querySelector('#add');
const logout = document.querySelector('#logout');
const status = document.querySelector('#status');

function filter() {
  const query = search.value.trim().toLowerCase();
  clearSearch.hidden = !search.value;
  let visible = 0;
  for (const row of list.children) {
    row.hidden = !(row.textContent.toLowerCase().includes(query) &&
      (!collection || row.lastElementChild.textContent === collection));
    if (!row.hidden) visible++;
  }
  empty.hidden = visible > 0;
}

function row(item) {
  const entry = document.createElement('li');
  const label = document.createElement('small');
  if (item.url && /^https?:\/\//.test(item.url)) {
    const link = document.createElement('a');
    link.href = item.url;
    link.textContent = item.title;
    entry.append(link);
  } else {
    entry.append(item.title);
  }
  label.textContent = item.collection;
  entry.append(label);
  return entry;
}

async function api(action, data) {
  const response = await fetch(apiOrigin + '/garden/api/' + action, {
    credentials: 'include',
    ...(data === undefined ? {} : {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data),
    }),
  });
  if (!response.headers.get('Content-Type')?.includes('application/json')) {
    throw new Error('The garden service is unavailable.');
  }
  const result = await response.json();
  if (!response.ok) {
    if (response.status === 401) signedIn(false);
    throw new Error(result.error || 'The request failed.');
  }
  return result;
}

function signedIn(active) {
  owner.hidden = false;
  login.hidden = active;
  add.hidden = !active;
  logout.hidden = !active;
}

function handle(button, action) {
  return async (event) => {
    event.preventDefault();
    if (button.disabled) return;
    button.disabled = true;
    status.textContent = '';
    try {
      await action();
    } catch (error) {
      status.textContent = error.message;
    } finally {
      button.disabled = false;
    }
  };
}

search.addEventListener('input', filter);
function selectCollection(button) {
  filters.querySelector('[aria-pressed="true"]').setAttribute('aria-pressed', 'false');
  button.setAttribute('aria-pressed', 'true');
  collection = button.dataset.collection;
  filter();
}
filters.addEventListener('click', (event) => {
  const button = event.target.closest('[data-collection]');
  if (button) selectCollection(button);
});
clearSearch.addEventListener('click', () => {
  search.value = '';
  filter();
  search.focus();
});
search.addEventListener('search', filter);
document.querySelector('#filters').hidden = false;

const loadBookmarks = api('bookmarks').then((items) => {
  list.replaceChildren(...items.map(row));
  filter();
}).catch(() => { /* The imported lists are also in the static page. */ });

api('session').then((session) => signedIn(session.signedIn)).catch(() => {});

login.addEventListener('submit', handle(login.querySelector('button'), async () => {
  const data = Object.fromEntries(new FormData(login));
  login.reset();
  await api('login', data);
  signedIn(true);
  add.elements.url.focus();
}));

add.addEventListener('submit', handle(add.querySelector('button'), async () => {
  const item = await api('bookmarks', Object.fromEntries(new FormData(add)));
  await loadBookmarks;
  if (![...list.querySelectorAll('a')].some((link) => link.href === item.url)) list.prepend(row(item));
  search.value = '';
  selectCollection(filters.querySelector('[data-collection=""]'));
  add.reset();
  status.textContent = 'Bookmark saved.';
  add.elements.url.focus();
}));

logout.addEventListener('click', handle(logout, async () => {
  await api('logout', {});
  add.reset();
  signedIn(false);
  login.elements.password.focus();
}));

const garden = document.querySelector('#garden-pixels').getContext('2d');

function pixel(color, x, y, width = 1, height = 1) {
  garden.fillStyle = color;
  garden.fillRect(x, y, width, height);
}

function tile(top, left, right, x, y, depth) {
  for (let offset = 0; offset < 9; offset++) {
    const face = Math.floor(offset / 2);
    pixel(left, x + offset, y + 4 + face, 1, depth);
    pixel(right, x + 17 - offset, y + 4 + face, 1, depth);
  }
  for (let offset = 0; offset < 8; offset++) {
    const inset = Math.abs(4 - offset) * 2;
    pixel(top, x + inset, y + offset, 18 - 2 * inset);
  }
}

function flower(x, y, color, height) {
  pixel('#527655', x, y - height, 1, height + 1);
  pixel('#659365', x - 2, y - 3, 2, 1);
  pixel('#416847', x + 1, y - 5, 2, 1);
  pixel(color, x - 2, y - height, 5, 1);
  pixel(color, x, y - height - 2, 1, 5);
  pixel('#f7d487', x, y - height);
}

if (garden) {
  const meadow = ['#8dad76', '#7ea170', '#9ab683'];
  for (let row = 0; row < 6; row++) {
    for (let col = 0; col < 7; col++) {
      const x = 63 + (col - row) * 8;
      const y = 26 + (col + row) * 4;
      const pond = row > 2 && row < 5 && col < 3 && !(row === 4 && col === 0);
      const path = col === 3 || (row === 1 && col > 3);
      const top = pond ? '#73a8aa' : path ? '#c6b48a' : meadow[(row * 2 + col) % meadow.length];
      tile(top, '#576a4e', '#718263', x, y, 5);
      if (pond && (row + col) % 2 === 0) pixel('#aed0c4', x + 7, y + 3, 5);
      if (path && (row + col) % 2 === 0) pixel('#aa9978', x + 8, y + 5, 2);
      if (!pond && !path && (row * 3 + col) % 4 !== 0) {
        const flowers = ['#e9a79e', '#f3d28f', '#e8ded0', '#b8a7d0'];
        flower(x + 10, y + 4, flowers[(row + col * 3) % flowers.length], 5 + (col % 3) * 2);
      }
    }
  }
}
