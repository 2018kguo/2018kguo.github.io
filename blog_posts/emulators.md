# Feature emulators

I've been thinking about how to test a feature when it needs Slack, an email inbox, a sandbox provider, a queue, and someone clicking through the result.

That loop is slow enough that it rarely gets run after every change. With an agent writing code, it's also a handoff: the agent makes a change, I check Slack, then I describe what went wrong. It can only keep working once I've reported back.

I'd rather give it a local version of that loop. Send an event, run the product code, inspect the message and files it produced, and try again.

[emulate.dev](https://emulate.dev/) applies a similar idea to external APIs. You run a local service in place of something like Stripe or GitHub, with state that your normal client code can read and change. I'm interested in doing this for whole features, including the parts inside the app.

## Test the path a user takes

Take a Slack message with a file attached. The app has to receive it, store the attachment, pass it into a chat workflow, produce an artifact, and upload that artifact back to the thread.

A test of the upload helper won't catch an attachment that went missing before the workflow started. A test of the workflow alone won't catch the result being sent to the wrong thread. The failures I keep running into involve the connections between those pieces.

For that feature, I want a test that starts with a Slack-shaped event and ends with the outbound message. It should use the same entry point and workflow as production. The provider can be fake, but the app should still do the work.

Then the assertions can describe what went wrong:

- Did the attachment get stored and passed to the chat?
- Was the generated file uploaded to the right thread?
- Did a retry keep the right run and message IDs?
- Did the final message include the artifact?

I still use unit tests for parsers and small transformations. Those don't need a Slack workspace, real or emulated.

## Where to put the fake

The fake should sit where the app meets the provider. If production uses cursor pagination, the fake needs to paginate. If a sandbox streams events, the fake needs to send events in that form. A shortcut that calls an internal helper instead of the production entry point can skip the bug entirely.

It also needs state. Upload a file and it should be there when the app asks for it later. Send a message and the test should be able to inspect its body, attachments, and destination.

For a local session, I'd want:

- database fixtures for the user, thread, and documents
- provider state for messages and files
- local file storage
- the real workflow code
- captures of outbound messages and stream events

When a test fails, those captures should be available together. I'd much rather open the conversation, rows, and files from the failed run than start from an assertion that `postMessage` was called once.

## Make it cheap to repeat

The point of making this local is that it should be easy to run after every change. No API keys to find, stale messages in a shared test workspace, or sandbox account that's been rate limited.

An agent can seed a session, run the turn, and read the captures itself. If the document never reached the workflow, it has evidence to work from immediately. I don't need to open Slack and explain the symptom first.

The setup needs to be repeatable enough for CI, too. Otherwise I'll end up debugging differences between a local session and whatever state the shared test account happened to be in.

## Terminals have a useful version of this

A terminal's screen is a grid of characters. Feed bytes into a terminal implementation and you can inspect the resulting cells. [Ghostty's terminal library](https://mitchellh.com/writing/libghostty-is-coming) makes that boundary particularly interesting to me.

[Ink](https://github.com/vadimdemedes/ink) gives React terminal apps a related advantage: the rendered output is text that a test can inspect. There are still things to test in a real terminal, but the basic output is accessible without a screenshot.

A Slack or email feature doesn't produce a character grid. It does produce a message body, attachments, and stored state. Capturing those gives me something comparably useful to assert on.

## Where this breaks down

We've had fakes that were missing provider methods or returned a more convenient shape than the real service. A test can pass for months that way. When we find a mismatch, the fake needs to change; skipping the failing assertion only hides it.

Some bugs need the real provider. An emulator can't tell me whether Modal is having a networking problem, whether an OAuth grant works, or whether a vendor will rate-limit this workload. Those need separate live checks.

The tests I want here are for bugs like "I sent a file in Slack and the agent couldn't find it" or "the artifact was generated but never appeared in the thread." A local run should exercise enough of the product to reproduce those, and leave enough evidence to explain them.

## Sources

- [emulate.dev](https://emulate.dev/)
- [Libghostty Is Coming](https://mitchellh.com/writing/libghostty-is-coming)
- [Ghostty](https://github.com/ghostty-org/ghostty)
- [Ink](https://github.com/vadimdemedes/ink)
