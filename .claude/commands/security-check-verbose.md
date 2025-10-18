# Security Theatre Check (Verbose)

Execute the security theatre detection with detailed output showing all issues.

## Usage
```
/security-check-verbose
```

## What it does
Same as `/security-check` but with verbose output showing:
- File paths and line numbers for each issue
- Full context of problematic code
- Detailed severity levels
- Category-by-category breakdown

## When to use
Use this when you need to:
- Review all security issues in detail
- Understand the context of each problem
- Prepare for a code review
- Debug why pre-commit hooks are failing
