import { createHash } from 'node:crypto';
import { cp, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import MarkdownIt from 'markdown-it';

await rm(new URL('../dist/', import.meta.url), { recursive: true, force: true });
const md = new MarkdownIt();
const gardenScriptVersion = createHash('sha256').update(await readFile('site/garden.js')).digest('hex').slice(0, 12);
const posts = [
  ['ramp_career', 'Notes after Ramp', '2026'],
  ['llm_math', "A layman's view of LLMs", '2026'],
  ['emulators', 'Feature emulators', '2026'],
  ['postgres', 'Postgres', '2024'],
  ['interpreters', 'Interpreters', '2024'],
];
const lists = [
  ['reading_list', 'Reading List'],
  ['pub_links', 'Misc Links'],
];
const contacts = [
  ['Email', 'mailto:kev.guo123@gmail.com'],
  ['LinkedIn', 'https://www.linkedin.com/in/2018kguo/'],
  ['GitHub', 'https://github.com/2018kguo'],
  ['X', 'https://x.com/2018kguo'],
  ['Medium', 'https://2018kguo.medium.com/'],
  ['Resume', 'https://drive.google.com/file/d/1aL_SNToa_CmD92qWKYF2ouFmAGUMPb2Y/view'],
];

const escape = md.utils.escapeHtml;
const readPost = (slug) => readFile(`assets/blog_posts/${slug}.md`, 'utf8');
const postLink = (slug) => `/notes/${slug}/`;
const write = async (path, content) => {
  const url = new URL(`../dist/${path}`, import.meta.url);
  await mkdir(new URL('.', url), { recursive: true });
  await writeFile(url, content);
};

function page(title, body, area) {
  const nav = [['/', 'About'], ['/notes/', 'Notes'], ['/garden/', 'Garden']];
  return `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>${escape(title === 'Kevin Guo' ? title : `${title} · Kevin Guo`)}</title>
<meta name="referrer" content="strict-origin-when-cross-origin">
${area === 'Garden' ? `<meta http-equiv="Content-Security-Policy" content="default-src 'self'; connect-src 'self' https://garden-api.kguo.dev; object-src 'none'; base-uri 'none'; form-action 'none'">` : ''}
<link rel="apple-touch-icon" href="/icon_ios_touch_192.png">
<link rel="icon" href="/favicon.ico">
<link rel="stylesheet" href="/site.css">
<script src="/site.js" defer></script>
</head>
<body>
<header class="masthead">
  ${area === 'About' ? '<h1 class="name"><a href="/">Kevin Guo</a></h1>' : '<a class="name" href="/">Kevin Guo</a>'}
  <nav aria-label="Site">${nav.map(([href, label]) => `<a href="${href}"${label === area ? ' aria-current="page"' : ''}>${label}</a>`).join('')}</nav>
  <button id="theme" type="button" hidden>Theme</button>
</header>
${body}
</body>
</html>`;
}

function notesList(active) {
  return `<ul class="notes-list">${posts.map(([slug, title, year]) =>
    `<li><a href="${postLink(slug)}"${active === slug ? ' aria-current="page"' : ''}>${escape(title)}</a> <small>${year}</small></li>`).join('')}
  <li><a href="/garden/">Reading List &amp; Misc Links</a></li></ul>`;
}

function notesPage(title, content, active) {
  return page(title, `<main class="notes-layout">
<aside class="notes-nav"><h2>Notes</h2>${notesList(active)}</aside>
<section class="reading-pane">${content}</section>
</main>`, 'Notes');
}

await write('index.html', page('Kevin Guo', `<main class="home">
<section class="about" aria-label="About Kevin">
<p>I'm a software engineer based in NYC. I was previously at Ramp and am now working on something new.</p>
<p>I strive to build performant and composable software. I've recently been interested in event-driven systems, Rust, and programming languages.</p></section>
<section class="links"><h2>Links</h2><ul>${contacts.map(([label, href]) =>
  `<li><a href="${href}">${label}</a></li>`).join('')}</ul></section>
<section class="home-notes"><h2>Notes</h2>${notesList()}</section>
<canvas id="life" aria-hidden="true"></canvas>
</main>`, 'About'));

await write('notes/index.html', page('Notes', `<main><h1>Notes</h1>${notesList()}</main>`, 'Notes'));
for (const [slug, title, year] of posts) {
  const source = await readPost(slug);
  const heading = `# ${title}\n\n`;
  const content = source.startsWith(heading) ? source.slice(heading.length) : source;
  await write(`notes/${slug}/index.html`, notesPage(title, `<article>
<a class="back" href="/notes/">← All notes</a>
<h1>${escape(title)}</h1><p class="year">${year}</p>${md.render(content)}
</article>`, slug));
}

const bookmarks = [];
for (const [slug, collection] of lists) {
  // The two source files are flat Markdown lists with one title and optional URL per item.
  for (const token of md.parse(await readPost(slug), {})) {
    if (token.type !== 'inline') continue;
    const title = token.children.filter((child) => child.type === 'text').map((child) => child.content).join('');
    const url = token.children.find((child) => child.type === 'link_open')?.attrGet('href') ?? '';
    if (!title || (url && !/^https?:\/\//.test(url))) throw new Error(`Invalid entry in ${slug}`);
    bookmarks.push({ title, url, collection });
  }
}
await write('garden/seed.json', JSON.stringify(bookmarks));
await write('garden/index.html', page('Garden', `<main class="garden-layout">
<section class="garden">
<h1>Garden</h1>
<p>Things I’m reading and links I keep coming back to.</p>
<details id="owner" class="owner">
<summary>Owner tools</summary>
<form id="login" method="post">
<label>Password <input type="password" name="password" autocomplete="current-password" maxlength="128" required></label>
<button disabled>Sign in</button>
<noscript>Enable JavaScript to sign in.</noscript>
</form>
<form id="add" hidden>
<label>URL <input type="url" name="url" maxlength="2048" placeholder="https://" required></label>
<label>Title <input name="title" maxlength="200" required></label>
<label>List <select name="collection"><option>Bookmarks</option><option>Reading List</option><option>Misc Links</option></select></label>
<button>Save bookmark</button>
</form>
<button id="logout" type="button" hidden>Sign out</button>
<p id="status" role="status"></p>
</details>
<div id="filters" hidden>
<div class="garden-search">
<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" aria-hidden="true"><circle cx="10.5" cy="10.5" r="6"/><path d="m15 15 4 4" stroke-linecap="round"/></svg>
<input id="search" type="search" aria-label="Search bookmarks" placeholder="Find a bookmark…" autocomplete="off">
<button id="clear-search" type="button" aria-label="Clear search" hidden>×</button>
</div>
<div id="collection" role="group" aria-label="Filter by list">
<button type="button" data-collection="" aria-pressed="true">All</button>
${['Reading List', 'Misc Links', 'Bookmarks'].map((name) => `<button type="button" data-collection="${name}" aria-pressed="false">${name}</button>`).join('')}
</div>
</div>
<ul id="bookmarks">${bookmarks.map(({ title, url, collection }) => `<li>${url ? `<a href="${escape(url)}">${escape(title)}</a>` : escape(title)}<small>${escape(collection)}</small></li>`).join('')}</ul>
<p id="empty" hidden>No matching bookmarks.</p>
</section>
<canvas id="garden-pixels" width="152" height="104" aria-hidden="true"></canvas>
<script src="/garden.js?v=${gardenScriptVersion}" defer></script>
</main>`, 'Garden'));

for (const file of ['CNAME', 'jumpseat', 'site/site.css', 'site/site.js', 'site/garden.js',
  'assets/favicon.ico', 'assets/icon_ios_touch_192.png']) {
  await cp(file, `dist/${file.split('/').at(-1)}`, { recursive: true });
}
// Preserve the public URLs produced by the previous build.
for (const slug of [...posts, ...lists, ['s3'], ['local_slack_agent_draft']].map(([name]) => name)) {
  await write(`blog_posts/${slug}.md`, await readPost(slug));
}
