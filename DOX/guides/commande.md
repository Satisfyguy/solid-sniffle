# Monero Marketplace Server Commands

## 1. Start the Server (in a separate terminal)

Run this command to start the server in the foreground. You will see its output directly in the terminal.

```bash
DATABASE_URL=sqlite:marketplace.db DB_ENCRYPTION_KEY=1507741993bdf8914031465a9dc63dd7e1f32a7bc2cd2b49e647042450503724 cargo run -p server --bin server > server.log 2>&1 &
```

## 2. View Server Logs (in another separate terminal)

Run this command in a *different* terminal to continuously view the server's log output.

```bash
tail -f server.log
```

---

**Instructions for Testing Edit Functionality:**

1.  Ensure the server is running using the command above.
2.  Open your web browser and navigate to a listing's detail page (e.g., `http://localhost:8080/listings/{id}`).
3.  Click the 'EDIT LISTING' button.
4.  Modify the listing details in the form.
5.  Submit the form.
6.  Copy and paste the relevant log output from *both* terminals here for analysis.
