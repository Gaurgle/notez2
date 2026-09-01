### Task 2.5: Reword the demo stub

`demo` stays unimplemented by decision, but its message promises a milestone that no longer exists.

**Files:**
- Modify: `crates/notez-cli/src/main.rs:92-94`

- [ ] **Step 1: Change the message**

```rust
        Commands::Demo { view: _ } => Err(anyhow::anyhow!(
            "demo is not implemented; it was a screenshot helper in the legacy CLI"
        )),
```

- [ ] **Step 2: Verify the workspace builds and all tests pass**

```bash
cd ~/Repos/notez && cargo build --workspace && cargo test --workspace
```

- [ ] **Step 3: Commit**

```bash
git -C ~/Repos/notez add crates/notez-cli/src/main.rs
git -C ~/Repos/notez commit -m "docs: drop stale milestone promise from demo stub"
```

---

## Phase 3: Install

