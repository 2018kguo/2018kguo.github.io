# A layman's view of LLMs

The most useful way to demystify LLMs is to remember that every answer is running on a budget.

For one forward pass through a model, the amount of work is mostly determined by the model, the input length, and the output length. The model does not secretly decide to spend 100x more FLOPs because the question is harder.

If I ask:

> what is the capital of France?

and then ask:

> prove this obscure theorem from first principles

the second question is harder for me, but the model is still running the same kind of machinery per token. It can spend more compute only if I give it more tokens, let it make more attempts, call tools, use a bigger model, route to subagents, etc.

Harder prompts do not automatically get more thinking time.

## Fixed compute per token

There is a good thought experiment in [LLMs and computation complexity](https://www.lesswrong.com/posts/XNBZPbxyYhmoqD87F/llms-and-computation-complexity):

- "The richest country in North America is the United States of ____"
- "The SHA1 of `abc123`, iterated 500 times, is ____"

Both ask for a next token / next string. One can be answered from a learned pattern. The other requires a bunch of exact computation.

The model does not get a special "this is SHA1, allocate more compute" mode. A normal forward pass has a fixed amount of computation. If the answer requires one more step than the model effectively performs, it misses.

Chain-of-thought, scratchpads, tool calls, sampling multiple attempts, and subagents matter because they buy more computation at inference time.

The model can write intermediate tokens. Those tokens become part of the next step. That gives it more sequential compute than a single token prediction. But now you are paying with latency, context, and tokens.

## Attention gets expensive fast

Transformers work by letting tokens look at other tokens.

That is attention.

If the prompt gets longer, the number of token relationships grows quickly. A 2x longer input can mean roughly 4x as many token-token comparisons in the attention part of the prefill. This is the quadratic thing people talk about.

The room analogy is fine: if everyone in a room has to check everyone else, adding people gets expensive fast.

Long context is useful, but expensive.

FlashAttention is one of the important systems improvements here. It keeps exact attention but changes how the work is tiled and moved through GPU memory. The point is that the math is only part of the cost. Data movement matters a lot too.

## Long documents need routing

There is another common mistake:

> the model has a 1M token context window, so just put everything in the prompt

Sometimes that works. Often it is wasteful or worse.

If a model does well on a 10 page document with a certain amount of useful context, then a 100 page document usually needs more than the same tiny slice of attention. To preserve quality over a larger corpus, you need to preserve the ratio of useful signal to total context.

If you simply stuff the whole corpus into the window, two things happen:

- attention/prefill cost grows very fast
- the useful detail is now competing with much more irrelevant text

This is related to the ["lost in the middle"](https://arxiv.org/abs/2307.03172) result: models can have long context windows and still be bad at using information buried in the middle.

So a lot of good systems split the problem up.

That can look like:

- retrieval over chunks
- dynamic chunking
- map/reduce over sections
- routing easy questions to RAG and hard ones to long context
- workflows that call smaller agents on pieces of the corpus
- Claude-style subagents that inspect different files or sections, then report back

This follows from the compute shape.

If full attention over the whole corpus is too expensive or too noisy, split the corpus into smaller pieces, run focused passes, then combine the results. You are trading one huge attention problem for several smaller, more targeted ones.

## Data quality

The older scaling-law story is: more parameters, more data, more compute. Useful, but incomplete. Data is not fungible.

One trillion tokens of duplicated SEO trash is not the same as one trillion tokens of high-quality books, code, papers, docs, and conversations. Repeated low-quality patterns teach the model low-quality patterns. Clean examples teach cleaner behavior. Domain data teaches domain behavior.

"Just crawl more internet" gets less satisfying over time.

The Chinchilla paper made the compute/data balance more concrete: for a fixed training budget, a smaller model trained on more data can beat a much larger undertrained model. But even that still leaves a harder question: what data?

Data quality shows up in boring but important ways. It decides what facts the model has probably seen, what styles it imitates, which mistakes it learned as normal, and which domains feel familiar to it. A model trained on a lot of a thing will be fluent in that thing, including the bad habits.

When people say models are pattern recognizers, that should make the data feel more important, not less.

## Data poisoning

If training data shapes behavior, poisoned training data can shape behavior too.

[Poisoning Web-Scale Training Datasets is Practical](https://arxiv.org/abs/2302.10149) is a good paper here. The basic point is that web-scale datasets are not immutable truth. Web pages change. Crowdsourced sources change. Crawlers snapshot things at different times. Attackers can exploit that.

The same idea shows up in LLM-specific poisoning work. [Sleeper Agents](https://arxiv.org/abs/2401.05566) shows proof-of-concept models that behave normally until a trigger appears, and then switch behavior. More recent poisoning work argues that attacks may not need to control a huge percentage of the training set to create a backdoor.

This does not mean all models are poisoned. It means data provenance matters.

If the model is a learned compression of patterns, then the quality, origin, duplication, and adversarial content of those patterns are core model properties. They are not footnotes.

## Inference has its own laws

Training gets most of the attention, but inference has its own constraints.

When you ask a model a question, serving has two phases:

- prefill: read the prompt
- decode: generate tokens one at a time

Prefill can use a lot of parallelism. Decode is more sequential because token 50 depends on token 49.

KV caching helps because the model stores attention information from previous tokens instead of recomputing everything. But the cache gets bigger as the context gets longer. Eventually the bottleneck can become memory bandwidth: moving cached data around, not doing raw math.

Inference systems care about:

- time to first token
- tokens per second
- batch size
- KV cache size
- memory bandwidth
- quantization
- speculative decoding
- paged attention

vLLM's PagedAttention is a nice example. It improves serving by managing KV cache memory more like virtual memory pages. That does not make the model smarter. It lets the same hardware serve more work with less waste.

## Pattern recognition

Calling it "just pattern matching" does not explain much.

A lot of human work is pattern recognition with good taste layered on top:

- noticing a code smell
- recognizing a clause type
- seeing that an error looks like missing auth
- matching a bug to a previous incident
- knowing that a paragraph sounds off
- spotting the shape of an argument before formalizing it

LLMs are not doing this the way humans do. But it should not be surprising that a huge learned pattern machine can be useful in domains where the input and output are mostly text-shaped patterns.

The limitation falls out of the same framing. If a task needs exact hidden state, exact arithmetic, a fact that isn't in the context, or simply more computation than one forward pass provides, the model can sound completely confident and still be wrong.

That's most of my working mental model. Patterns explain why these things are capable at all; the fixed compute per token explains most of how they fail. Data quality is where the personality comes from, and data provenance is why poisoning is worth worrying about. And once you see the compute shape, it's obvious why serious long-context setups are systems — routing, chunking, subagents — rather than one giant prompt.

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
