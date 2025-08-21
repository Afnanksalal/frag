# ⚡ Frag

Frag is an **experimental, compiler-based programming language** written in Rust, designed with the goal of becoming a **modern systems + robotics language**.

Think of it as:

* The **minimalism of C**,
* The **safety and semantics of Rust**,
* The **hardware awareness of Verilog**,
* The **approachability of Python** (eventually).

Frag is still in its **prototype stage** — but it’s already capable of parsing, interpreting, and executing basic arithmetic, variables, and print statements.

---

## ✨ Features (Current Status)

✅ Lexical analysis (tokenizer)
✅ Parser → AST (Abstract Syntax Tree)
✅ Expression evaluation (arithmetic, boolean logic)
✅ Variable declarations and assignment
✅ Print statements for output
✅ Binary and unary operators

---

## 📖 Language Example

Here’s a small `example.frag`:

```frag
let a = 10;
let b = 20;

print(a + b);

let flag = true && false;
print(flag);

40 + 2;
```

**Output:**

```
30
false
42
```

---

## 🛠️ Build & Run

Frag is written in **Rust**, so you’ll need Cargo.

```bash
# Clone the repo
git clone https://github.com/<your-username>/frag.git
cd frag

# Build
cargo build --release

# Run an example
cargo run examples/hello.frag
```

---

## 📂 Project Structure

```
frag/
 ├── src/
 │   ├── lexer.rs      # Tokenizer (converts source into tokens)
 │   ├── parser.rs     # Parser (turns tokens into AST)
 │   ├── ast.rs        # AST node definitions
 │   ├── codegen.rs    # Interpreter / evaluator
 │   └── main.rs       # Entry point
 ├── examples/
 │   └── example.frag    # Example code
 ├── Cargo.toml
 └── README.md
```

---

## 🎯 Vision

Frag’s long-term vision is **not** to be “just another toy language”.
The aim is to evolve into a **systems + robotics-focused programming language** that’s:

* **Safe by design** → Rust-like memory model, preventing crashes on bare metal.
* **Lightweight** → Compile fast, run on embedded devices, no runtime bloat.
* **Hardware-first** → GPIO, UART, I²C, SPI support for microcontrollers.
* **Robotics-ready** → Middleware for messaging, real-time control, and simulation.
* **Future-proof** → Potential compilation targets for FPGA/ASIC DSLs.

In short: Frag should feel like **C for robotics in 2025**, but with modern design principles.

---

## 🚀 Roadmap

Here’s a rough roadmap for Frag development:

### ✅ Stage 1 – MVP Interpreter (Current)

* Basic lexer, parser, AST
* Arithmetic & Boolean expressions
* Variables & assignments
* Print statements

### 🔜 Stage 2 – Core Language

* Functions, scopes, blocks
* Type system (int, float, bool, string)
* Control flow (`if`, `while`, `for`)
* Standard library (math, string, IO)
* REPL for quick experiments

### ⚙️ Stage 3 – Systems Integration

* Module & import system
* FFI for C/Rust interop
* File & OS bindings
* Embedded platform support (Arduino, Raspberry Pi, etc.)
* Hardware bindings (GPIO, UART, I²C, SPI)

### 🤖 Stage 4 – Robotics Focus

* Real-time task scheduler
* Messaging middleware (ROS-like pub/sub)
* Motor, sensor, and actuator APIs
* Simulation hooks (Gazebo/Unity integration)

### 🔮 Stage 5 – Advanced Compilation

* LLVM/MLIR backend for performance
* FPGA/Verilog-lite backend for hardware synthesis
* Optimizer for robotics workloads
* Safety checker for memory, concurrency, and hardware states

---

## 🤝 Contributing

Frag is in **prototype mode**, so contributions are super welcome.

Ways to contribute:

* File issues → feature requests, bug reports, language design debates
* Submit pull requests → fix typos, add tests, or extend features
* Review design → tear apart decisions, bully the syntax, or suggest better ones
* Documentation → make Frag understandable for the next dev

### Setup for Development

```bash
git clone https://github.com/<your-username>/frag.git
cd frag
cargo run examples/hello.frag
```

---

## 📜 License

MIT License – you can copy, modify, redistribute. Just don’t blame me if your robot decides to overthrow humanity.

---

## 🖼️ Logo

![Frag Logo](./assets/logo.png)

---

## 🧭 Future Directions

Frag wants to answer this question:
**“What if C was invented today, but specifically for robotics and embedded systems?”**

If that excites you — welcome aboard. 🚀

