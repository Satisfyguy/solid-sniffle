#!/usr/bin/env bash
. "$HOME/.cargo/env"
# Script: pre-commit.sh
# Pre-commit verifications
# Usage: ./scripts/pre-commit.sh

set -e  # Exit on error (can be overridden for specific checks)

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${CYAN}PRE-COMMIT CHECKS${NC}"
echo -e "${CYAN}===================${NC}"

# Verify we're in the project root
if [[ ! -f ".cursorrules" ]]; then
    echo -e "${RED}ERROR: Run this script from the project root${NC}"
    exit 1
fi

errors=0
warnings=0

# 1. Verify project compiles
echo -e "\n${YELLOW}1. Verifying compilation...${NC}"
if cargo check 2>&1; then
    echo -e "   ${GREEN}Project compiles correctly${NC}"
else
    echo -e "   ${RED}Compilation errors detected${NC}"
    ((errors++))
fi

# 2. Code formatting
echo -e "\n${YELLOW}2. Verifying format...${NC}"
if cargo fmt --check 2>&1; then
    echo -e "   ${GREEN}Code is properly formatted${NC}"
else
    echo -e "   ${YELLOW}Code formatting issues, auto-correcting...${NC}"
    if cargo fmt; then
        echo -e "   ${GREEN}Code reformatted automatically${NC}"
    else
        echo -e "   ${RED}Error during formatting${NC}"
        ((errors++))
    fi
fi

# 3. Clippy (linter)
echo -e "\n${YELLOW}3. Verifying Clippy...${NC}"
if cargo clippy -- -D warnings 2>&1; then
    echo -e "   ${GREEN}No Clippy warnings${NC}"
else
    echo -e "   ${YELLOW}Clippy warnings detected${NC}"
    ((warnings++))
fi

# 4. Tests
echo -e "\n${YELLOW}4. Running tests...${NC}"
if cargo test 2>&1; then
    echo -e "   ${GREEN}All tests pass${NC}"
else
    echo -e "   ${RED}Tests failed${NC}"
    ((errors++))
fi

# 5. Verify specs exist
echo -e "\n${YELLOW}5. Verifying specs...${NC}"
function_count=$(find . -name "*.rs" -not -path "*/target/*" -exec grep -cE "pub\s+(async\s+)?fn\s+\w+" {} + | awk '{sum+=$1} END {print sum}')
spec_count=$(find docs/specs -name "*.md" 2>/dev/null | wc -l)

if [[ $spec_count -ge $function_count ]]; then
    echo -e "   ${GREEN}All functions have specs${NC}"
else
    echo -e "   ${YELLOW}$((function_count - spec_count)) function(s) without specs${NC}"
    ((warnings++))
fi

# 6. Verify unwraps
echo -e "\n${YELLOW}6. Verifying unwraps...${NC}"
unwrap_count=$(find . -name "*.rs" -not -path "*/target/*" -exec grep -c "\.unwrap(" {} + | awk '{sum+=$1} END {print sum}')

if [[ $unwrap_count -eq 0 ]]; then
    echo -e "   ${GREEN}No unwrap() found${NC}"
elif [[ $unwrap_count -le 5 ]]; then
    echo -e "   ${YELLOW}$unwrap_count unwrap() found (threshold: 5)${NC}"
    ((warnings++))
else
    echo -e "   ${RED}$unwrap_count unwrap() found (threshold: 5)${NC}"
    ((errors++))
fi

# 7. Verify TODOs
echo -e "\n${YELLOW}7. Verifying TODOs...${NC}"
todo_count=$(find . -name "*.rs" -not -path "*/target/*" -exec grep -ciE "TODO|FIXME" {} + | awk '{sum+=$1} END {print sum}')

if [[ $todo_count -eq 0 ]]; then
    echo -e "   ${GREEN}No TODOs found${NC}"
elif [[ $todo_count -le 10 ]]; then
    echo -e "   ${YELLOW}$todo_count TODO(s) found (threshold: 10)${NC}"
    ((warnings++))
else
    echo -e "   ${RED}$todo_count TODO(s) found (threshold: 10)${NC}"
    ((errors++))
fi

# 8. Check Security Theatre
echo -e "\n${YELLOW}8. Checking for security theatre...${NC}"
if [[ -f "./scripts/check-security-theatre.sh" ]]; then
    if ./scripts/check-security-theatre.sh; then
        echo -e "   ${GREEN}No security theatre detected${NC}"
    else
        echo -e "   ${RED}Security theatre detected!${NC}"
        ((errors++))
    fi
else
    echo -e "   ${YELLOW}Security theatre script not found (skipping)${NC}"
    ((warnings++))
fi

# 9. Check Monero/Tor Security
echo -e "\n${YELLOW}9. Checking Monero/Tor security...${NC}"
if [[ -f "./scripts/check-monero-tor.sh" ]]; then
    if ./scripts/check-monero-tor.sh; then
        echo -e "   ${GREEN}No Monero/Tor security issues detected${NC}"
    else
        echo -e "   ${RED}Monero/Tor security issues detected!${NC}"
        ((errors++))
    fi
else
    echo -e "   ${YELLOW}Monero/Tor security script not found (skipping)${NC}"
    ((warnings++))
fi

# 10. Update metrics
echo -e "\n${YELLOW}10. Updating metrics...${NC}"
if [[ -f "./scripts/update-metrics.sh" ]]; then
    if ./scripts/update-metrics.sh; then
        echo -e "   ${GREEN}Metrics updated${NC}"
    else
        echo -e "   ${YELLOW}Error updating metrics${NC}"
        ((warnings++))
    fi
else
    echo -e "   ${YELLOW}Metrics update script not found (skipping)${NC}"
    ((warnings++))
fi

# Final summary
echo -e "\n${CYAN}PRE-COMMIT SUMMARY${NC}"
echo -e "${CYAN}===================${NC}"

if [[ $errors -eq 0 && $warnings -eq 0 ]]; then
    echo -e "${GREEN}ALL CHECKS PASS - Ready to commit!${NC}"
    exit 0
elif [[ $errors -eq 0 ]]; then
    echo -e "${YELLOW}$warnings warning(s) detected - Commit possible but be careful${NC}"
    echo -e "${CYAN}Consider fixing warnings before committing${NC}"
    exit 0
else
    echo -e "${RED}$errors error(s) detected - COMMIT BLOCKED${NC}"
    echo -e "${YELLOW}Fix errors before committing${NC}"
    exit 1
fi
