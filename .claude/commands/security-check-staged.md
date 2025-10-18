# Security Theatre Check (Staged Files Only)

Execute the security theatre detection on Git staged files only.

## Usage
```
/security-check-staged
```

## What it does
Scans ONLY the Rust files currently staged for commit. This is faster and more focused than a full codebase scan.

## When to use
- Before committing code
- Quick validation of your changes
- Part of your Git workflow
- When you want fast feedback

## Note
This command requires:
- You are in a Git repository
- You have staged files (`git add` has been run)
- At least one staged file is a Rust file (*.rs)
