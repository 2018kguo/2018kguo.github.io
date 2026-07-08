# A layman's view of LLMs

The single most useful idea for understanding LLMs is this: every answer runs on a fixed budget.

A forward pass through the model does a set amount of work. That amount is decided by the model size, how long your input is, and how many tokens it generates. It is *not* decided by how hard your question is. The model does not notice that a question is difficult and quietly spend 100x more effort on it.

Ask this:

> what is the capital of France?

then ask this:

> prove this obscure theorem from first principles

The second one is much harder for me. To the model, both are just "produce the next token," running the same machinery at the same cost per token. The only way to throw more compute at the hard one is to *give* it more: let it write more tokens, take more attempts, call tools, use a bigger model, hand pieces to subagents. Left alone, a hard prompt does not buy itself more thinking time.

Almost everything below is a consequence of that one fact.

## Fixed compute per token

There's a sharp example in [LLMs and computation complexity](https://www.lesswrong.com/posts/XNBZPbxyYhmoqD87F/llms-and-computation-complexity):

- "The richest country in North America is the United States of ____"
- "The SHA1 of `abc123`, iterated 500 times, is ____"

Both are just "finish the string." But the first is a fact the model has seen a thousand times, and the second is 500 rounds of exact hashing. There is no "oh, this is SHA1, let me allocate more compute" switch. One forward pass has a fixed number of steps. If the answer needs one more step than the model can do in a single pass, it doesn't get close — it gets it wrong, confidently.

This is why chain-of-thought, scratchpads, tool calls, sampling several attempts, and subagents work at all. They don't make the model smarter per token. They let it spend more tokens, and each token it writes becomes input for the next step. Writing out its reasoning literally buys the model more sequential steps to compute with. You pay for it in latency, context, and tokens — but you're buying real computation.

## Attention gets expensive fast

Transformers work by letting every token look at every other token. That "looking at" is attention.

The catch is in the bookkeeping. If everyone in a room has to shake hands with everyone else, doubling the number of people roughly quadruples the handshakes. Same with tokens: a prompt twice as long means about four times as many token-to-token comparisons during prefill. That's the "quadratic" cost people complain about. Long context is genuinely useful, and genuinely expensive.

[FlashAttention](https://arxiv.org/abs/2205.14135) is worth knowing about here. It computes the exact same attention, but reorganizes how the numbers move through GPU memory. It's a reminder that the math is only half the cost — shuffling data around is the other half, and often the bigger half.

## Long documents need routing

A tempting mistake:

> the model has a 1M token context window, so I'll just dump everything into the prompt.

Sometimes that's fine. Often it's wasteful, and sometimes it actively makes the answer worse.

Think about the ratio of useful text to total text. If a model does well on a 10-page document, a 100-page document isn't going to do as well on the same thin sliver of attention. Stuffing the whole thing in does two bad things at once: prefill cost balloons, and the handful of sentences you actually care about are now drowning in ten times more noise. This is the ["lost in the middle"](https://arxiv.org/abs/2307.03172) problem — a model can have a huge context window and still be bad at using something buried in the middle of it.

So the good systems break the problem up instead:

- retrieval over chunks
- map/reduce over sections
- routing easy questions to a quick retrieval pass and hard ones to full long context
- subagents that each read a different file or section and report back

It all falls out of the compute shape. One giant, noisy attention problem is worse than several small, focused ones, so you split it and recombine the results.

## Data quality

The classic scaling-law story is "more parameters, more data, more compute." True, but incomplete, because it treats all data as interchangeable. It isn't.

A trillion tokens of duplicated SEO garbage is not a trillion tokens of good books, code, papers, and real conversations. The model learns whatever patterns it's shown most often — so repeated junk teaches junk, and clean examples teach clean behavior. Data quality quietly decides what facts the model has probably seen, what writing style it imitates, and which mistakes it learned to treat as normal. "Just crawl more internet" hits diminishing returns fast.

The [Chinchilla paper](https://arxiv.org/abs/2203.15556) made one part of this concrete: for a fixed training budget, a smaller model trained on *more* data beats a bigger model that was starved of it. But it still leaves the harder question wide open — more data, sure, but *which* data?

And here's the thing people get backwards: if you believe LLMs are "just pattern matchers," that should make you care *more* about the data, not less. The patterns are the whole product.

## Data poisoning

Data quality has an adversarial cousin. If ordinary bad data shapes the model by accident, someone can shape it on purpose.

The uncomfortable part is how little control that takes. [Poisoning Web-Scale Training Datasets is Practical](https://arxiv.org/abs/2302.10149) shows that the web these crawls draw from isn't fixed — pages change, crowdsourced entries change, crawlers snapshot at different moments — and an attacker can plant content knowing it'll get scraped. [Sleeper Agents](https://arxiv.org/abs/2401.05566) demonstrates models that act normal until a specific trigger shows up, then flip behavior. And more recent work suggests you may not need to poison a large *fraction* of the data to slip in a backdoor — a small, fixed number of poisoned examples can be enough.

This doesn't mean every model is compromised. It means the origin of the training data is a real property of the model, not a footnote. Where the data came from matters as much as how much of it there was.

## Serving a model is its own hard problem

Training gets the headlines, but actually running a model for users — inference — is a separate engineering problem with its own bottlenecks, and it's the part that decides whether a model is fast and cheap enough to be worth using.

When you send a prompt, serving happens in two phases:

- **prefill**: read and digest your prompt
- **decode**: generate the answer one token at a time

Prefill can chew through your whole prompt in parallel. Decode can't — token 50 depends on token 49, so it's stuck going one at a time. That's why the first token can feel slow and then the rest streams out steadily.

The main trick that keeps decode fast is the **KV cache**: instead of re-reading the whole conversation for every new token, the model stores what it already computed about earlier tokens and reuses it. The cost is memory — the cache grows with the conversation, and past a certain length the bottleneck stops being math and becomes memory bandwidth, i.e. just *moving that cached data around*.

This is why serving people care about time-to-first-token, tokens-per-second, batch size, KV cache size, quantization, speculative decoding, and paged attention. [vLLM's PagedAttention](https://arxiv.org/abs/2309.06180) is a clean example: it manages the KV cache like a computer manages virtual memory pages. It doesn't make the model any smarter — it lets the same GPUs serve more requests with less waste. Same model, more throughput.

## Pattern recognition

"It's just pattern matching" gets said like it's a dismissal. It shouldn't be, because a lot of skilled human work is pattern matching with good taste on top:

- noticing a code smell
- recognizing what kind of contract clause you're looking at
- seeing that an error smells like missing auth
- matching a new bug to an old incident
- sensing that a paragraph reads off before you can say why

An LLM isn't doing these the way a person does. But once you accept that it's an enormous pattern machine trained on text, it's not surprising that it's useful anywhere the input and output are mostly text-shaped patterns. That's a big chunk of knowledge work.

The limitations come from the exact same place. If a task needs exact arithmetic, a fact that simply isn't in the context, or more computation than one forward pass can do, the model will still produce fluent, confident text — it just won't be right. Pattern matching is why these things work at all; fixed compute per token is why they fail the way they do.

## Putting it together

That's most of my working mental model:

- **Patterns** explain why LLMs are capable in the first place.
- **Fixed compute per token** explains most of how they fail, and why chain-of-thought and tools help.
- **Data quality** is where a model's personality and blind spots come from, and **data provenance** is why poisoning is worth taking seriously.
- Once you see the compute shape, it's obvious why serious long-context work is a *system* — routing, chunking, subagents — and not one giant prompt.

## Sources

- [LLMs and computation complexity](https://www.lesswrong.com/posts/XNBZPbxyYhmoqD87F/llms-and-computation-complexity)
- [Attention Is All You Need](https://papers.neurips.cc/paper/7181-attention-is-all-you-need.pdf)
- [FlashAttention](https://arxiv.org/abs/2205.14135)
- [Efficiently Scaling Transformer Inference](https://arxiv.org/pdf/2211.05102)
- [vLLM / PagedAttention](https://arxiv.org/abs/2309.06180)
- [Lost in the Middle](https://arxiv.org/abs/2307.03172)
- [Retrieval Augmented Generation or Long-Context LLMs?](https://arxiv.org/abs/2407.16833)
- [Scaling Laws for Neural Language Models](https://arxiv.org/abs/2001.08361)
- [Training Compute-Optimal Large Language Models](https://arxiv.org/abs/2203.15556)
- [Language Models are Few-Shot Learners](https://arxiv.org/abs/2005.14165)
- [Poisoning Web-Scale Training Datasets is Practical](https://arxiv.org/abs/2302.10149)
- [Sleeper Agents](https://arxiv.org/abs/2401.05566)
