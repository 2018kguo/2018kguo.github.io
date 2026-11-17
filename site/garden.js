const apiOrigin = ['localhost', '127.0.0.1'].includes(location.hostname) ? '' : 'https://garden-api.kguo.dev';
const list = document.querySelector('#bookmarks');
const search = document.querySelector('#search');
const collection = document.querySelector('#collection');
const empty = document.querySelector('#empty');
const owner = document.querySelector('#owner');
const login = document.querySelector('#login');
const add = document.querySelector('#add');
const logout = document.querySelector('#logout');
const status = document.querySelector('#status');

function filter() {
  const query = search.value.trim().toLowerCase();
  let visible = 0;
  for (const row of list.children) {
    row.hidden = !(row.textContent.toLowerCase().includes(query) &&
      (!collection.value || row.lastElementChild.textContent === collection.value));
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
collection.addEventListener('change', filter);
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
  collection.value = '';
  filter();
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
