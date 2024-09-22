## Notes from [Homegrown bytecode interpreters](https://medium.com/bumble-tech/home-grown-bytecode-interpreters-51e12d59b25c)

- For passing in arguments of arbitrary lengths, you can add a table on constants separate 
from the opcode array and add instructions that refer to the address in the table of constants
- Add a built in stack to evaluate complex expressions - Python and the JVM are stack based
- VMs can either be stack or register based - technically both are interopable, registers are 
harder to compile to though. [This paper](https://www.usenix.org/legacy/events/vee05/full_papers/p153-yunhe.pdf) 
compiled JVM stack code into register code. Generally speaking, register-based VMs are faster due to
less instructions needed to manage the stack.
- Can use VMs to implement regex patterns
- Can use static superinstructions to combine multiple sequences of instructions to optimize, but this is hard to determine
without knowing a specific program ahead of time. Can dynamically compile these superinstructions, which was used in primitive
JIT compilers
- Can optimize the switch branch for opcodes to squeeze out more time