# ⚛️ ProtonVM

![logo](https://github.com/ohayouarmaan/quark-vm-2/blob/main/logo.webp?raw=true)
**ProtonVM** is a general-purpose, secure, low-level virtual machine designed for maximal control, minimal abstraction, and memory safety. Built in **Rust**, it executes programs written in **QASM**, its custom assembly language. No registers, no garbage collection—just deliberate and explicit memory manipulation with powerful syscall support.

---

## 🔥 Key Features

- 💾 **Heap-Based Allocation** – All structured data lives in the managed heap.
- 🧊 **Raw Memory Buffer** – A sandboxed space for native syscalls and unsafe operations.
- 🔐 **Deep Security Model** – Hard separation of heap and raw buffer; corruption is virtually impossible.
- 📦 **QASM Assembly** – Lean and expressive instruction set for fine-grained control.
- 🧵 **Reference/Dereference Semantics** – Supports indirection across heap and raw buffer.
- 🛠️ **Native Syscalls** – Direct system-level interaction via QASM.
- 🚀 **Supports Native DLLs** – Can run Native DLLs.

---

## 🧠 Memory Architecture

### 📚 Constant Pool

Immutable values (numbers, strings, etc.) are stored here and accessed via QASM instructions.

### 🧠 Heap

The main arena for dynamic allocation. Every data structure, value, or reference exists here unless explicitly allocated elsewhere.

### 💣 Raw Buffer

Used for native syscalls and low-level memory operations. Cannot be directly read—must be copied into the heap for access. Offers deep isolation to prevent data leaks and unintended mutations.

---

## 📖 QASM Instruction Set

ProtonVM instructions are defined in the following Rust enum:

```rust
pub enum InstructionType {
    INST_NOOP = 0,
    INST_PUSH,
    INST_POP,
    INST_ADD,
    INST_AND,
    INST_OR,
    INST_XOR,
    INST_NOT,
    INST_SHL,
    INST_SHR,
    INST_MUL,
    INST_DIV,
    INST_SUB,
    INST_JMPZ,
    INST_JMPEQ,
    INST_JMPNEQ,
    INST_JMPNZ,
    INST_PUSH_STR,
    INST_ALLOC,
    INST_ALLOC_RAW,
    INST_SYSCALL,
    INST_DUP,
    INST_INSWAP,
    INST_PRINT,
    INST_LOAD,
    INST_STORE,
    INST_DEREF,
    INST_DEREF_FOREIGN,
    INST_REF,
    INST_DEBUG,
    INST_CALL,
    INST_RET,
    INST_PUT,
    INST_STD_SYSCALL,
    INST_DLL_LOAD,
    INST_DLL_CALL,
}
```

### 🔍 Common Instructions

| Instruction        | Description |
|--------------------|-------------|
| `PUSH <val>`       | Pushes a constant value onto the stack. |
| `POP`              | Pops the top value from the stack. |
| `ADD, SUB, MUL, DIV` | Basic arithmetic on top two stack values. |
| `AND, OR, XOR, NOT` | Bitwise logic operations. |
| `SHL, SHR`         | Bit shifts. |
| `JMPZ`             | Jump if top of stack is zero. |
| `JMPNZ`            | Jump if top of stack is non-zero. |
| `JMPNEQ, JMPEQ`    | Conditional jumps. |
| `ALLOC <n>`        | Allocates `n` words in the heap. |
| `ALLOC_RAW <n>`    | Allocates `n` bytes in raw memory. |
| `STORE <i>`        | Stores top of stack in constant pool at index `i`. |
| `LOAD <i>`         | Loads from heap at index/address `i` to stack. |
| `REF`              | Pushes a reference (pointer) to the value on top of the stack. |
| `DEREF`            | Dereferences the pointer on top of the stack. |
| `SYSCALL <n>`      | Pops `n` arguments and then the syscall ID from the stack. Executes native syscall. |
| `PUSH_STR "<s>"`   | Allocates a string from the constant pool into the heap and pushes its pointer to the stack. |
| `DUP`              | Duplicates the top value on the stack. |
| `INSWAP`           | Swaps top two elements on the stack. |
| `PRINT`            | Prints the top value (usually for debug). |
| `DEBUG`            | Emits current VM state snapshot (stack, heap, etc.). |
| `NOOP`             | Does nothing. Great for alignment or labels. |
| `DLL_LOAD`         | Loads a given DLL by Popping the TOS for the DLL Path |
| `DLL_CALL <n>`     | Calls any given method from the DLL by popping the TOS to get method name and n mentions the number of arguments it should pop |

---

## 🧾 QASM Example

```qasm
main:
    LOAD 0        ; Load value from heap at index 0
    ALLOC 16      ; Allocates 16 words in the heap
    ALLOC_RAW 16  ; Allocates 16 bytes in the raw memory
    STORE 0       ; Stores the top of stack in constant pool at index 0
    SYSCALL 3     ; Pops 3 args and syscall ID from the stack, performs syscall
```

---

## 🚀 Getting Started

### 🧰 Build (Requires Rust)

```bash
git clone https://github.com/yourusername/protonvm
cd protonvm
cargo build --bin assembler
cargo build --bin machine
```

### 🧪 Run a Program

```bash
cargo run assembler -- path/to/program.qasm path/to/output.out
cargo run machine -- path/to/bytecode.out
```

---

## 📌 Use Cases

- Writing your own language backend
- Building secure sandboxed runtimes
- Teaching VM fundamentals
- Systems experimentation and low-level OS design
- Native syscall-driven applications

---

## 🔮 Roadmap

- 🧼 Garbage collection (optional opt-in)
- 🧬 Structs and compound types in heap
- 📜 QASM includes/macros
- 🧪 Debugger and trace output
- 🧊 Safe interop with host system

---

## 🤝 Contributing

Got ideas? Want to improve QASM, add new instructions, or build tooling around ProtonVM? Pull requests are very welcome. If you like VMs that don’t hold your hand but still play nice, this one’s for you.

---

## 📜 License

MIT – Use it, fork it, ship it. Just don’t sue us.

---

## 🧘 Philosophy

ProtonVM is built around **intentionality**. No black-box features, no magic optimizations. You know where every byte goes, and why.

> _“Control isn’t dangerous—confusion is.”_

