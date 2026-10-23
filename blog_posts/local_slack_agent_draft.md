# Running agents from Slack on my laptop

I've been messing around w/ a setup where I can tag a Slack bot and have it run stuff on my laptop.

At a high level:

```text
slack message -> local slack app -> local terminal API -> agent process
```

The terminal API part is [pi-mux](https://github.com/2018kguo/pi-mux), which is basically a small HTTP server for PTYs. It can spawn a terminal, write to stdin, read output, wait for regexes, send ctrl-c, restart the process, etc.

The slack part is just a Bolt app running locally using Socket Mode. Socket Mode is nice here because I don't need to expose my laptop to the internet or run ngrok. The app opens a websocket to Slack, receives mentions/DMs, and talks to pi-mux on `127.0.0.1`.

The reason this is useful is that my laptop already has all the annoying local context:

- repos checked out
- `gh` auth
- local scripts
- dev servers
- CLI tools like `psql`, `railway`, `vercel`, etc.
- random logs and files that I don't want to upload somewhere

So instead of turning every small thing into a real service, I can just ask the bot from Slack.

Example things this is good for:

- check if a PR is still failing CI
- run a local script and tell me when it's done
- tail some logs and summarize the new errors
- poll an API every few minutes and ping me when something opens
- keep a long-running agent task in a thread instead of a terminal tab

## Similar idea to @claude

The mental model is pretty close to [Claude Code's GitHub Action](https://code.claude.com/docs/en/github-actions). You tag `@claude` in a PR/issue and it gets the relevant GitHub context.

I think the nice part is not specifically Claude or GitHub Actions. It's the "tag the agent where the task already is" flow.

For GitHub, that place is a PR or issue. The diff, CI, comments, and review context are already there.

For local laptop stuff, Slack works better. I can tag it from my phone, the result lands in the same thread, and I don't need to copy terminal output into a chat window.

## Minimal version

The basic version is small:

1. Create a Slack app
2. Enable Socket Mode
3. Add bot scopes like `app_mentions:read`, `chat:write`, and `im:history`
4. Run a local Bolt app
5. On mention/DM, start or reuse a terminal
6. Write the message into the terminal
7. Read output and post back to the same thread

Roughly:

```ts
app.event("app_mention", async ({ event, client }) => {
  const threadTs = event.thread_ts ?? event.ts;
  const key = `${event.channel}:${threadTs}`;

  const term = await getOrCreateTerminal(key, {
    command: "pi",
    args: ["--task", "help from this slack thread"],
  });

  await write(term.id, stripMention(event.text));
  const reply = await waitForReply(term.id, { timeoutSeconds: 120 });

  await client.chat.postMessage({
    channel: event.channel,
    thread_ts: threadTs,
    text: reply,
  });
});
```

You don't need pi-mux for this. `node-pty` plus a little state file would be enough.

I used pi-mux because I wanted:

- one terminal per Slack thread
- output stored w/ sequence numbers
- `wait until this regex appears`
- screenshots of terminal state
- restart/signal/key primitives
- a simple `/status?format=text` view I can read from my phone

The per-thread part matters a lot. A reply in the same Slack thread should continue the same terminal session. Otherwise you lose all the context after every message.

## Why use a PTY?

Calling an LLM API directly from the Slack app is cleaner if you're building a normal chatbot.

For this, I wanted the opposite. I wanted the agent to be able to use the same shell environment I use. Same repo checkout, same `gh` login, same local scripts, same weird one-off commands.

The PTY is a lazy integration layer. The Slack app doesn't need to know how GitHub, Railway, Postgres, or my build scripts work. It just writes text into a terminal and reads text back.

This is also why I think the setup is more useful than it looks. A lot of automation is not worth productizing. It only needs to run on my machine and report back somewhere convenient.

## Security

This is the part to be careful about.

A Slack bot connected to a local terminal is not "just a chatbot". It can do whatever your user account can do. If your shell can push code, delete files, deploy services, or read private notes, the bot can probably do those too.

My default assumptions:

- bind the terminal API to `127.0.0.1`
- use Socket Mode instead of exposing a public endpoint
- use a token for the local pi-mux API if anything else can reach it
- allowlist Slack channels/users
- don't run it as root
- redact token-looking strings before posting to Slack
- don't pass `.env`, private keys, SSH config, etc. into the agent
- cap output size before sending it back to Slack
- log what the bot did locally
- require confirmation for destructive or externally visible actions

The last one is important. Reading logs is different from pushing a commit. Summarizing CI is different from deploying prod. I want the bot to help with local work, not become a remote shell that any Slack message can drive.

I also think the bot should treat Slack as an untrusted input source. Even in a private workspace, someone can paste weird instructions, files, or terminal-looking text. The bridge should strip Slack markup, avoid blindly executing pasted commands, and make the agent ask before doing anything irreversible.

None of this makes it perfectly safe. It just keeps the blast radius closer to what the thing actually is: a local operator with my permissions.

## What I ended up liking

The useful part is not that the architecture is clever. It is mostly glue.

```text
Slack = interface
SQLite = thread/session mapping
PTY = execution
local files = transcripts/memory
```

But the workflow feels good. I can start something from my phone, check status later, and keep the whole task in a Slack thread.

It also makes small monitors easy. I had one that checked an availability endpoint every few minutes and posted when a slot opened. That is too small to deserve a real app, but perfect as a local bot job.

Same for PR checks, log scans, and random "run this and tell me when it finishes" tasks.

The broader point is that not every useful agent workflow needs to be a hosted product. Sometimes you just want a taggable process on your own laptop.
