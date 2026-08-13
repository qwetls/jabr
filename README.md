# Jabr

A programming language inspired by the Islamic Golden Age — easy to understand, yet powerful.

> *"The word 'algorithm' comes from al-Khwarizmi, the 9th-century scholar whose work laid the foundation for modern computation. Jabr continues that legacy."*

## Status

**v0.1.0** — tree-walking interpreter (arithmetic, variables, functions, control flow).

## Syntax (v1)

```jabr
// Variables
let x = 10;
let name = "Jabr";

// Arithmetic
print 1 + 2 * 3;        // 7
print (1 + 2) * 3;      // 9

// Strings
print "foo" + "bar";   // foobar

// Booleans & comparisons
print true and false;   // false
print 1 < 2;            // true
print 42 == 42;         // true

// Functions
fn add(a, b) {
    return a + b;
}
print add(3, 4);        // 7

// Control flow
let i = 0;
while i < 3 {
    print i;
    i = i + 1;
}

if x > 5 {
    print "big";
} else {
    print "small";
}
```

## Architecture

```
source.jabr
    │
    ▼
  Lexer ──► tokens
    │
    ▼
  Parser ──► AST
    │
    ▼
  Interpreter ──► output
```

**Roadmap:**
- [x] v1 — Tree-walking interpreter
- [ ] v2 — Bytecode VM
- [ ] v3 — Native codegen via LLVM
- [ ] v4 — Self-hosting

## Development

Local development only requires editing files and pushing. CI handles all building and testing:

```bash
git add -A && git commit -m "..." && git push
```

GitHub Actions runs `cargo fmt --check`, `cargo clippy`, `cargo build`, and `cargo test` on every push.

## License

MIT
