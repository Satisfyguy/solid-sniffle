# Security Theatre Check

Execute the security theatre detection skill with default settings.

## Usage
```
/security-check
```

## What it does
Scans all Rust files in the project for security theatre patterns including:
- Unwrapped results without error handling
- Placeholder comments (TODO/FIXME)
- Magic numbers without constants
- Hardcoded credentials
- Debug print statements
- Unsafe blocks without justification

## Parameters
Uses default settings:
- Scans entire codebase
- Minimum severity: INFO
- Excludes files in `.security-theatre-ignore`

## Related Commands
- `/security-check-verbose` - Detailed output with all issues
- `/security-check-staged` - Check only Git staged files
- `/security-check-critical` - Show only CRITICAL and HIGH issues
