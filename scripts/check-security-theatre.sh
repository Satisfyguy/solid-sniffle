#!/usr/bin/env bash
# scripts/check-security-theatre.sh
# Automatic detection of security theatre in Rust code

# Default parameters
PATH_TO_SCAN="."
IGNORE_FILE=".security-theatre-ignore"
VERBOSE=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --verbose|-v)
            VERBOSE=true
            shift
            ;;
        --path)
            PATH_TO_SCAN="$2"
            shift 2
            ;;
        --ignore)
            IGNORE_FILE="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
GRAY='\033[0;37m'
WHITE='\033[1;37m'
NC='\033[0m' # No Color

echo -e "${CYAN}Security Theatre Detection${NC}"
echo -e "${CYAN}=============================${NC}"
echo ""

# Load exceptions
exceptions=()
if [[ -f "$IGNORE_FILE" ]]; then
    while IFS= read -r line; do
        # Skip comments and empty lines
        if [[ ! "$line" =~ ^\s*# && -n "${line// }" ]]; then
            exceptions+=("$line")
        fi
    done < "$IGNORE_FILE"

    if [[ "$VERBOSE" == true ]]; then
        echo -e "${YELLOW}Loaded ${#exceptions[@]} exceptions from $IGNORE_FILE${NC}"
    fi
fi

# Function to check if a line is excepted
is_exception() {
    local file_path="$1"
    local line_content="$2"

    for exception in "${exceptions[@]}"; do
        if [[ "$exception" =~ ^([^:]+):(.+)$ ]]; then
            local pattern="${BASH_REMATCH[1]}"
            local line_pattern="${BASH_REMATCH[2]}"

            # Check if file matches pattern
            if [[ "$file_path" == $pattern ]]; then
                # Check if line matches pattern
                if [[ "$line_content" =~ $line_pattern ]]; then
                    return 0  # True (is exception)
                fi
            fi
        fi
    done
    return 1  # False (not exception)
}

# Counters
total_issues=0
declare -a issues_found

# Detection patterns
patterns=(
    "assert!.*true"
    "assert!.*false"
    "//.*Placeholder"
    "//.*TODO"
    "//.*FIXME"
    "//.*XXX"
    "//.*HACK"
    "should.*work"
    "probably.*works"
    "assume"
    "HYPOTHÈSES"
    "À.*VALIDER"
    "ERREUR.*POSSIBLE"
    "À.*IMPLÉMENTER"
    "unimplemented!"
    "todo!"
    "panic!"
    "password.*="
    "secret.*="
    "key.*="
    "token.*="
    "api_key.*="
    "private_key.*="
    "\.unwrap\s*\(\s*\)"
    "println!"
    "print!"
    "dbg!"
)

# Find Rust files
mapfile -t rust_files < <(find "$PATH_TO_SCAN" -name "*.rs" -not -path "*/target/*" -not -path "*/.git/*")

echo -e "${YELLOW}Scanning ${#rust_files[@]} Rust files...${NC}"
echo ""

# Scan files
for file in "${rust_files[@]}"; do
    relative_path="${file#./}"

    line_number=0
    while IFS= read -r line; do
        ((line_number++))

        # Check exceptions
        if is_exception "$relative_path" "$line"; then
            continue
        fi

        # Test each pattern
        for pattern in "${patterns[@]}"; do
            if [[ "$line" =~ $pattern ]]; then
                issues_found+=("$relative_path:$line_number|${line// /}")
                ((total_issues++))

                if [[ "$VERBOSE" == true ]]; then
                    echo -e "${RED}❌ Security theatre detected${NC}"
                    echo -e "   ${GRAY}${relative_path}:${line_number}${NC}"
                    echo -e "   ${GRAY}${line}${NC}"
                    echo ""
                fi
            fi
        done
    done < "$file"
done

# Display report
echo -e "${CYAN}Security Theatre Report${NC}"
echo -e "${CYAN}=========================${NC}"
echo ""

if [[ $total_issues -eq 0 ]]; then
    echo -e "${GREEN}✅ No security theatre detected!${NC}"
    echo ""
    exit 0
fi

echo -e "${RED}❌ Security theatre detected: $total_issues issues${NC}"
echo ""

# Top 10 most critical issues
echo -e "${RED}Top Issues:${NC}"
count=0
for issue in "${issues_found[@]}"; do
    if [[ $count -ge 10 ]]; then break; fi

    IFS='|' read -r location content <<< "$issue"
    echo -e "  ${RED}$location${NC}"
    echo -e "    ${GRAY}$content${NC}"
    ((count++))
done
echo ""

# Recommendations
echo -e "${YELLOW}Recommendations:${NC}"
echo -e "  ${WHITE}1. Replace .unwrap() with proper error handling${NC}"
echo -e "  ${WHITE}2. Remove placeholder comments and implement real code${NC}"
echo -e "  ${WHITE}3. Replace assumptions with validated logic${NC}"
echo -e "  ${WHITE}4. Use constants instead of magic numbers${NC}"
echo -e "  ${WHITE}5. Remove hardcoded credentials${NC}"
echo ""

echo -e "${RED}❌ COMMIT BLOCKED - Fix security theatre issues first${NC}"
exit 1
