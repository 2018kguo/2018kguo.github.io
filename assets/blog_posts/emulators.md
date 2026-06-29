# Feature emulators

I've been thinking more about emulators as a way to make product iteration faster.

Not CPU emulators. More like local versions of the world your app normally talks to.

[emulate.dev](https://emulate.dev/) is a good example of this idea for external APIs. Instead of mocking Stripe or GitHub with a few fake functions, you run a local service that behaves enough like the real thing that your normal SDK code can talk to it. It is stateful, works offline, and can run in CI.

I think the same idea applies inside a product.

The main point is iteration speed.

If a feature takes a real Slack workspace, a real email inbox, a real sandbox provider, a real queue, and a bunch of manual clicking to test, nobody is going to check it on every change. The loop is too slow. You make a change, guess, deploy, click around, wait for some background job, then paste logs back into whatever is helping you.

That is especially bad for agent-heavy development. The agent can only fix what it can observe. If the verification step is "human opens Slack, sees if it looked right, then explains the failure", the agent is stuck waiting on the human. If the verification step is "run this local scenario and inspect the captured output", the agent can tighten the loop itself.

So the real value is self-verification:

- make a change
- run a realistic local scenario
- inspect the surface output and persisted state
- fix the next thing
- repeat

The emulator is the thing that makes that loop cheap enough to run constantly.

The version of this I like is building emulators around features and surfaces, not just provider clients. The goal is not "mock Slack" or "mock email". The goal is closer to:

- run a Slack turn without Slack
- run an email turn without Resend
- run a web chat turn without clicking through the UI
- run a sandbox/artifact turn without Modal
- run Library/grid flows without depending on a pile of external state

The important part is that the product code still runs.

If a Slack message normally enters through the Slack path, the emulator should enter through something shaped like that path. It should create a thread, attach files, run the same chat/workflow code, capture the outbound Slack message, and let the test inspect what a user would have seen.

That is very different from stubbing the last function in the call stack.

## Mocks are still useful

This is not an anti-mock take.

Mocks are good for narrow units. If a parser has 5 edge cases, just test the parser. If a function maps one object shape to another object shape, don't boot the world.

But mocks get weird when the feature boundary is larger than the function boundary.

For example, if the actual bug is "Slack file arrives, becomes a canonical attachment, gets passed into the chat workflow, produces an artifact, and uploads that artifact back to Slack", then mocking the final Slack upload helper only proves a tiny part of the chain.

The bug is usually in the glue:

- which id got persisted
- whether a file was normalized before the workflow started
- whether a background operation kept its parent run id
- whether a streamed update was visible on the surface
- whether retry/replay history still had the right shape
- whether email and Slack diverged even though they share backend code

Those are not comfortable unit tests. They are feature tests.

## What makes a good emulator

A useful emulator preserves the boundary that production code actually sees.

For an external API, that might mean HTTP shape, auth shape, pagination, files, webhooks, and state. This is the emulate.dev style.

For an internal product feature, the boundary might be:

- inbound surface event
- seeded database state
- fake but realistic provider state
- local storage for files
- workflow execution
- captured outbound messages

The emulator does not need to be perfect. It needs to fail in the same category as production.

If Slack has cursor pagination, your fake Slack client should have cursor pagination. If the sandbox provider streams events, the emulator should stream events. If the real feature writes rows and later replays them, the emulator should make that state inspectable.

The trap is building an emulator that is too convenient. If the test calls a helper that production never calls, it can pass while the product is still broken.

## Determinism matters because speed matters

The main benefit is not just repeatability in CI. It is making the verification loop short enough that you actually use it.

No API keys. No sandbox account that got rate limited. No stale third-party test data. No "works locally but failed in CI because the Slack workspace looked different".

A good harness can use local session state for provider data and captures. That makes tests easier to reason about:

- seed the state
- run the turn
- inspect the persisted app state
- inspect captured outbound messages

It also makes debugging much better. A failed test can dump the chat, provider captures, workflow rows, files, and events from one local session. That is a lot more useful than "expected postMessage to be called once".

This is what lets an agent self-correct. It does not need a human to describe what happened in the UI. The harness can expose the useful facts directly:

- what inbound event was sent
- what rows were written
- what files were created
- what message would have gone out
- what stream events the user would have seen

## TUIs can emulate themselves

The cleanest version of this idea is a TUI, because a TUI is already text in and text out. The full surface a user sees is a grid of characters you can capture, and the input is a stream of bytes you can replay. There is no screenshot diffing, no DOM, no headless browser. The thing the user perceives is the same thing the test can assert on.

Ghostty leans hard into this. Its terminal core can be driven with escape sequences and then asserted against the resulting cell grid — "given these bytes, the screen should look exactly like this" — and it has an extensive conformance suite doing exactly that. The core is being extracted into a standalone `libghostty-vt` library that parses sequences and maintains terminal state on its own, which is about as close to "the terminal can emulate itself" as it gets.

Claude Code gets a weaker version of the same thing almost for free, because it's a React/[Ink](https://github.com/vadimdemedes/ink) app. Ink renders the component tree to a text buffer instead of the DOM, so the frame the user sees is a string you can render and assert on directly. The surface is inspectable for the same reason a terminal's is: it's text the whole way down.

In both cases there's almost no gap between the real surface and the test surface, which is exactly the property that makes a test trustworthy.

A feature emulator is me trying to recover that property for things that aren't TUIs. A Slack turn or an email turn doesn't hand you a tidy character grid, so you build the harness that captures the equivalent — the outbound message, the rows written, the files produced. The goal is the same one the terminal gets for free: make the surface the user experiences directly inspectable, so the product can assert tightly on itself.

## The emulator should not hide reality

There is also a failure mode here.

If the emulator becomes a parallel product, it will lie to you.

We hit versions of this where the harness was missing a provider method, or a fake client did not behave like the real Slack client, or a sandbox emulator returned data in a shape that was easier than the real provider. The fix is not to weaken the test. The fix is usually to make the emulator boundary more faithful.

The test should say:

> production expects this provider surface to exist

not:

> skip this assertion because the fake does not implement it

It is also important to know when the emulator is the wrong tool. If the bug is specifically about Modal behavior, or a real provider's auth/quota/network behavior, then run against the real thing. The emulator is for product logic and integration shape, not for proving a vendor is behaving correctly.

## Why I like this pattern

The best tests are usually at the level where a user would describe the bug. Not "function X returned undefined" but "I sent a file in Slack and the agent said there was no document", or "the generated artifact existed but never showed up in the thread", or "the grid looked done but some cells were still running in the background." Those are the bugs people actually report, and they all live in the glue between components, not inside any one of them.

That is the level where emulators help. They let you build a small local world where the feature can happen end-to-end while staying deterministic enough for CI.

None of this replaces unit tests for pure logic. The split I've landed on is narrow: unit tests for the parser-shaped stuff, emulators for the product boundaries that need fast iteration. The only rules that have actually mattered are keep the fake providers stateful, capture what the user would have seen, hold onto enough state to debug a failure, and never let yourself believe emulator coverage is the same as live-provider coverage.

I think this is the same reason local API emulators are compelling. The interesting part is not "fake Stripe". The interesting part is preserving the contract where your app meets the outside world.

For product features, the outside world might be Slack, email, a sandbox, a file store, a workflow engine, or just another surface in your own app.

Once that boundary is emulated well, adding a regression test stops feeling like building a fragile mock tower. It feels more like dropping a scenario into a small local version of the product.

## Sources

- [emulate.dev](https://emulate.dev/)
- [Libghostty Is Coming](https://mitchellh.com/writing/libghostty-is-coming)
- [Ghostty](https://github.com/ghostty-org/ghostty)
- [Ink](https://github.com/vadimdemedes/ink)
