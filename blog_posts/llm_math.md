# A layman's view of LLMs

The detail that helped me understand LLMs was how they spend compute. A model generates text a token at a time. Each step runs through the model's layers; a harder question doesn't give that step extra layers to work with.

The total budget can vary a lot. A system can generate more reasoning tokens, try several answers, use tools, or hand work to another model. Those choices matter because a single pass has limited room to compute an answer.

## Compute per token

Suppose I ask for a familiar fact, then ask for the result of applying a hash function 500 times. Both prompts ask the model to finish a string. The fact might be easy to recall from training. The hash requires a sequence of exact operations that the model can't reliably squeeze into one pass.

Writing intermediate steps gives it more passes. Each generated token becomes input to the next, so a scratchpad lets a computation extend across tokens. That can help with a difficult problem, though it doesn't guarantee the steps are correct. For hashing, I'd want it to run code.

This is how I think about reasoning budgets and tool use: ways to give the system more work to do before it commits to an answer. The cost shows up in latency and tokens.

## Attention and long inputs

In dense self-attention, tokens compare information with other tokens in the sequence. The number of possible pairs grows quadratically: double the sequence length and there are roughly four times as many pairs. That applies to the attention work, not necessarily the cost of the whole model.

[FlashAttention](https://arxiv.org/abs/2205.14135) computes exact attention while reducing transfers between levels of GPU memory. The arithmetic matters, but so does how the data moves through the hardware.

Long context also has a quality problem. Fitting a document into the window doesn't establish that the model will use every part well. [Lost in the Middle](https://arxiv.org/abs/2307.03172) found that the models it tested often did worse when relevant information sat in the middle of a long input. I wouldn't treat that result as a verdict on every newer model, but it's a useful reason to test retrieval rather than assume it works.

For a large collection of documents, I usually want to compare a few approaches: retrieve relevant passages, process sections separately, or send the full context. Splitting the work can miss connections across sections; sending everything can be expensive and distracting. The task decides which tradeoff is acceptable.

## Training data

Parameters, data, and training compute have to be considered together. The [Chinchilla paper](https://arxiv.org/abs/2203.15556) showed that, under its training budget, a smaller model trained on more tokens outperformed larger models that had seen less data.

That still leaves the question of which data to use. Duplicated pages, good code, books, and incorrect answers teach different patterns. If I'm calling a model a pattern matcher, that makes the examples it learned from more relevant to me.

There's an adversarial version of this problem too. [Poisoning Web-Scale Training Datasets is Practical](https://arxiv.org/abs/2302.10149) examines how attackers can alter content collected for training. [Sleeper Agents](https://arxiv.org/abs/2401.05566) studies deliberately trained backdoor behavior that can survive subsequent safety training. These are different setups, but both make me wary of treating a dataset's size as sufficient information about it.

## Serving the model

Running a trained model has two main phases:

- **Prefill:** process the prompt.
- **Decode:** generate new tokens, each depending on the ones before it.

Prefill can process prompt positions in parallel. Decode has to proceed sequentially, which gives the two phases different performance characteristics.

The KV cache stores keys and values computed for earlier tokens so the model can reuse them during generation. A longer conversation needs a larger cache. Moving that data can become a bottleneck even when there's enough compute available.

[vLLM's PagedAttention](https://arxiv.org/abs/2309.06180) manages that cache in blocks, reducing wasted memory and making it easier to share storage. It's one example of why serving software matters: the same model can be much more practical to run with better memory management.

## Pattern recognition

A lot of work I use LLMs for involves recognizing something familiar in text: a code smell, a likely cause of an error, a clause in a contract, or an awkward paragraph. A model trained on enough examples can be useful at those tasks without doing them the way I do.

The trouble is that a plausible continuation isn't always a correct answer. Exact arithmetic needs checking. Missing facts need a source. A complicated change needs tests. I get better results when the system can do those checks and use what it finds on the next step.

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
