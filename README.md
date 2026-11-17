# kguo.dev

The site builds to static HTML, CSS, and JavaScript. GitHub Pages publishes
`dist/` when changes land on `dev`. Notes come from the Markdown in
`assets/blog_posts/`. The Garden imports both of the original lists from
that folder and supports additional bookmarks through a Cloudflare Worker
with a SQLite-backed Durable Object.

## Run locally

```sh
npm ci
printf 'GARDEN_PASSWORD=%s\n' "$(openssl rand -hex 32)" > .dev.vars
npm run dev
```

Open the local address printed by Wrangler. The password in `.dev.vars`
is for local testing. The API stores local test data in `.wrangler/`.

## Publish bookmark editing

The Worker uses `garden-api.kguo.dev` as a Cloudflare Custom Domain.
Deploy it in the Cloudflare account that owns the zone, on Workers Free.
GitHub Pages continues to serve the site on `kguo.dev`.

1. Authenticate with `npx wrangler login` and deploy with
   `npm run deploy:worker`.
2. Generate a fresh 64-character hex password with `openssl rand -hex 32`
   and save it in your password manager. Run
   `npx wrangler secret put GARDEN_PASSWORD` and paste it at the prompt.
3. Open `https://kguo.dev/garden/` and expand **Owner tools** to sign in
   and save a bookmark. Anyone can browse the public list. The only
   bookmark write is an authenticated add, and duplicate URLs are rejected.

The public page includes the two imported lists in its HTML. Owner tools
appear once the backend has a configured password. Sessions expire
after 24 hours and changing the password invalidates them.
