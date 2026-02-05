# CopyPaws Desktop - Coding Rules

## General

1. **Language:** Vietnamese comments OK, but code/variables/functions in English
2. **No console.log in production:** Use proper logging
3. **Error handling:** Always handle errors explicitly, never silent catch

---

## Rust (Backend)

### Style
- Follow Rust standard naming: `snake_case` for functions/variables, `PascalCase` for types
- Use `#[derive]` macros where appropriate
- Prefer `Result<T, E>` over panicking

### Architecture
- Each module in separate file (`websocket.rs`, `clipboard.rs`, etc.)
- Use `Arc<Mutex<T>>` for shared state
- Async with Tokio

### Error Handling
```rust
// Good
fn do_something() -> Result<(), AppError> {
    let result = risky_operation()?;
    Ok(())
}

// Bad
fn do_something() {
    risky_operation().unwrap(); // Don't panic!
}
```

### Database
- All DB operations through `database/mod.rs`
- Use prepared statements
- Handle migrations properly

---

## TypeScript (Frontend)

### Style
- Use TypeScript strict mode
- Define interfaces for all data structures
- Prefer `const` over `let`

### Components
- Functional components with hooks
- Props interface at top of file
- Keep components small and focused

### State
- Use `useState` for local state
- Use `useEffect` for side effects
- Cleanup subscriptions in useEffect return

### Tauri Commands
```typescript
// Good - typed invoke
const status = await invoke<ServerStatus>("get_server_status");

// Bad - untyped
const status = await invoke("get_server_status");
```

---

## Platform-Specific: Windows

### System Tray
- Use Tauri's built-in tray support
- Icon must be `.ico` format for Windows

### Autostart
- Use `tauri-plugin-autostart`
- Registry entry in `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`

### Window Behavior
- `CloseRequested` → hide to tray, don't exit
- Single instance enforcement

---

## Git Workflow

1. **Commits:** Clear, descriptive messages
2. **Branches:** `feature/`, `fix/`, `docs/` prefixes
3. **No large files:** Don't commit build artifacts

---

## Documentation

1. Update `PROGRESS.md` when completing features
2. Update `TODO.md` when adding/removing tasks
3. Keep `Architecture/` in sync across all 3 folders
