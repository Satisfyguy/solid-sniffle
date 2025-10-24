# Audit Guide

This document provides a guide to using the `swissy.sh` audit script.

## Running the script

To run the script, execute the following command from the project root:

```bash
./scripts/swissy.sh
```

## Interpreting the score

The script provides a score from 0 to 100. The score is calculated based on the severity of the issues found.

- **CRITICAL**: 25 points deduction
- **HIGH**: 15 points deduction
- **MEDIUM**: 10 points deduction
- **LOW**: 5 points deduction

## Corrective Actions

Based on the output of the script, you should take the following actions:

- **CRITICAL** issues must be fixed immediately.
- **HIGH** issues should be prioritized.
- **MEDIUM** issues should be addressed in a timely manner.
- **LOW** issues are suggestions for improvement.

## CI/CD Integration

The script can be integrated into your CI/CD pipeline to automate the audit process. The script will exit with a non-zero status code if any issues are found.
