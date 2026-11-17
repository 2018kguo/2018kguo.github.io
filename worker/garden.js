import { DurableObject } from 'cloudflare:workers';
import seed from '../dist/garden/seed.json';

const prefix = '/garden/api/';
const cookieName = '__Host-garden';
const sessionSeconds = 86400;
const collections = new Set(['Bookmarks', 'Reading List', 'Misc Links']);
const encoder = new TextEncoder();

function json(body, status = 200, headers = {}) {
  return Response.json(body, { status, headers: {
    'Cache-Control': 'no-store',
    'X-Content-Type-Options': 'nosniff',
    ...headers,
  } });
}

async function readJson(request) {
  if (request.headers.get('Content-Type') !== 'application/json' || !request.body) return null;
  const reader = request.body.getReader();
  const bytes = new Uint8Array(8192);
  let length = 0;
  try {
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      if (length + value.length > bytes.length) {
        await reader.cancel();
        return null;
      }
      bytes.set(value, length);
      length += value.length;
    }
    return JSON.parse(new TextDecoder().decode(bytes.subarray(0, length)));
  } catch {
    return null;
  } finally {
    reader.releaseLock();
  }
}

function bookmark(input) {
  if (!input || typeof input.title !== 'string' || typeof input.url !== 'string' ||
      !collections.has(input.collection)) return null;
  const title = input.title.trim();
  if (!title || title.length > 200 || input.url.length > 2048) return null;
  try {
    const url = new URL(input.url);
    if (!['http:', 'https:'].includes(url.protocol) || url.username || url.password ||
        url.href.length > 2048) return null;
    return { title, url: url.href, collection: input.collection };
  } catch {
    return null;
  }
}

async function digest(value) {
  return await crypto.subtle.digest('SHA-256', encoder.encode(value));
}

async function sessionHash(token) {
  return new Uint8Array(await digest(token)).toHex();
}

function sessionToken(request) {
  const value = request.headers.get('Cookie')?.split(';').map((part) => part.trim())
    .find((part) => part.startsWith(cookieName + '='))?.slice(cookieName.length + 1);
  return /^[a-f0-9]{64}$/.test(value ?? '') ? value : null;
}

function cookie(token, age) {
  return `${cookieName}=${token}; Max-Age=${age}; Path=/; Secure; HttpOnly; SameSite=Strict`;
}

export default {
  async fetch(request, env) {
    const url = new URL(request.url);
    if (!url.pathname.startsWith(prefix)) return json({ error: 'Not found.' }, 404);
    const origin = request.headers.get('Origin');
    const local = ['localhost', '127.0.0.1'].includes(url.hostname);
    const allowed = origin === (local ? url.origin : 'https://kguo.dev');
    if (request.method === 'OPTIONS') {
      if (!allowed) return json({ error: 'Request must come from the garden.' }, 403);
      return new Response(null, { status: 204, headers: {
        'Access-Control-Allow-Origin': origin,
        'Access-Control-Allow-Credentials': 'true',
        'Access-Control-Allow-Methods': 'GET, POST',
        'Access-Control-Allow-Headers': 'Content-Type',
        'Vary': 'Origin',
      } });
    }
    const response = await handleRequest(request, env, url.pathname.slice(prefix.length), allowed);
    response.headers.set('Vary', 'Origin');
    if (allowed) {
      response.headers.set('Access-Control-Allow-Origin', origin);
      response.headers.set('Access-Control-Allow-Credentials', 'true');
    }
    return response;
  },
};

async function handleRequest(request, env, action, allowed) {
  const store = env.GARDEN.getByName('personal-garden');
  if (request.method === 'GET' && action === 'bookmarks') return json(await store.list());

  const password = env.GARDEN_PASSWORD;
  if (typeof password !== 'string' || !/^[a-f0-9]{64}$/.test(password)) {
    return json({ error: 'Owner sign-in is unavailable.' }, 503);
  }
  // A rotated password invalidates existing sessions.
  const passwordKey = await sessionHash(password);
  const token = sessionToken(request);
  const hash = token && await sessionHash(token);
  if (request.method === 'GET' && action === 'session') {
    return json({ signedIn: Boolean(hash && await store.hasSession(hash, passwordKey)) });
  }
  if (request.method !== 'POST') return json({ error: 'Not found.' }, 404);
  if (!allowed) {
    return json({ error: 'Request must come from the garden.' }, 403);
  }

  if (action === 'login') {
    const ip = request.headers.get('CF-Connecting-IP') || 'local';
    if (!(await env.LOGIN_LIMIT.limit({ key: ip })).success) {
      return json({ error: 'Too many attempts. Try again in a minute.' }, 429);
    }
    const input = await readJson(request);
    const submitted = input?.password;
    if (typeof submitted !== 'string' || submitted.length !== 64 ||
        !crypto.subtle.timingSafeEqual(await digest(submitted), await digest(password))) {
      return json({ error: 'Incorrect password.' }, 401);
    }
    const nextToken = crypto.getRandomValues(new Uint8Array(32)).toHex();
    await store.signIn(await sessionHash(nextToken), passwordKey, hash);
    return json({ signedIn: true }, 200, { 'Set-Cookie': cookie(nextToken, sessionSeconds) });
  }

  if (action === 'logout') {
    if (hash) await store.signOut(hash);
    return json({ signedIn: false }, 200, { 'Set-Cookie': cookie('', 0) });
  }
  if (action !== 'bookmarks') return json({ error: 'Not found.' }, 404);
  if (!hash || !await store.hasSession(hash, passwordKey)) {
    return json({ error: 'Sign in to save bookmarks.' }, 401);
  }
  const item = bookmark(await readJson(request));
  if (!item) return json({ error: 'Enter a title, a valid HTTP or HTTPS URL, and a list.' }, 400);
  const result = await store.add(hash, passwordKey, item);
  if (result === 'expired') return json({ error: 'Sign in to save bookmarks.' }, 401);
  if (result === 'duplicate') return json({ error: 'This URL is already in the garden.' }, 409);
  return json(item, 201);
}

export class Garden extends DurableObject {
  constructor(ctx, env) {
    super(ctx, env);
    this.sql = ctx.storage.sql;
    ctx.storage.transactionSync(() => {
      this.sql.exec(`CREATE TABLE IF NOT EXISTS bookmarks (
        id INTEGER PRIMARY KEY AUTOINCREMENT, title TEXT NOT NULL, url TEXT, collection TEXT NOT NULL, added INTEGER NOT NULL
      )`);
      this.sql.exec('CREATE UNIQUE INDEX IF NOT EXISTS bookmark_url ON bookmarks(url)');
      this.sql.exec(`CREATE TABLE IF NOT EXISTS sessions (
        hash TEXT PRIMARY KEY, password_key TEXT NOT NULL, expires INTEGER NOT NULL
      )`);
      for (const [index, item] of seed.entries()) {
        this.sql.exec(
          'INSERT OR IGNORE INTO bookmarks (id, title, url, collection, added) VALUES (?, ?, ?, ?, 0)',
          -index - 1, item.title, item.url ? new URL(item.url).href : null, item.collection,
        );
      }
    });
  }

  list() {
    return this.sql.exec(
      'SELECT title, coalesce(url, \'\') AS url, collection FROM bookmarks ORDER BY added DESC, id DESC',
    ).toArray();
  }

  hasSession(hash, passwordKey) {
    return this.sql.exec('SELECT 1 FROM sessions WHERE hash = ? AND password_key = ? AND expires > ?',
      hash, passwordKey, Date.now()).toArray().length > 0;
  }

  signIn(hash, passwordKey, previous) {
    this.ctx.storage.transactionSync(() => {
      this.sql.exec('DELETE FROM sessions WHERE expires <= ? OR password_key <> ? OR hash = ?',
        Date.now(), passwordKey, previous);
      this.sql.exec('INSERT INTO sessions VALUES (?, ?, ?)',
        hash, passwordKey, Date.now() + sessionSeconds * 1000);
    });
  }

  signOut(hash) {
    this.sql.exec('DELETE FROM sessions WHERE hash = ?', hash);
  }

  add(hash, passwordKey, item) {
    if (!this.hasSession(hash, passwordKey)) return 'expired';
    const inserted = this.sql.exec(
      'INSERT INTO bookmarks (title, url, collection, added) VALUES (?, ?, ?, ?) ON CONFLICT(url) DO NOTHING RETURNING id',
      item.title, item.url, item.collection, Date.now(),
    );
    return inserted.toArray().length ? 'added' : 'duplicate';
  }
}
