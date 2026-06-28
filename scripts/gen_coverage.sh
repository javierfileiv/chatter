#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_DIR"

# Regex for integration-only files (excluded from coverage report)
IGNORE_REGEX="(client/|server/src/main\.rs|server/src/core/server\.rs)"

HTML=0
OUTPUT_DIR=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --html)
            HTML=1
            shift
            ;;
        --output-dir)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--html [--output-dir <path>]]"
            exit 1
            ;;
    esac
done

echo "Running coverage (excluding integration-only files)..."
COV_OUTPUT=$(cargo llvm-cov \
    --ignore-filename-regex "$IGNORE_REGEX" \
    --workspace \
    --all-targets \
    --all-features 2>&1)

echo "$COV_OUTPUT" | grep -A1 "^TOTAL" | tail -1

if [[ "$HTML" -eq 1 ]]; then
    REPORT_DIR="${OUTPUT_DIR:-coverage-html}"
    echo ""
    echo "Generating HTML report in $REPORT_DIR..."
    cargo llvm-cov \
        --ignore-filename-regex "$IGNORE_REGEX" \
        --workspace \
        --all-targets \
        --all-features \
        --html \
        --output-dir "$REPORT_DIR"
    
    # Extract line coverage percentage and generate badge
    PERCENT=$(echo "$COV_OUTPUT" | grep "^TOTAL" | awk '{for(i=1;i<=NF;i++) if($i ~ /^[0-9]+\.[0-9]+%$/) print $i}' | tail -1 | sed 's/%//')
    if [[ -n "$PERCENT" ]]; then
        # Round to nearest integer for badge
        INT_PERCENT=$(printf "%.0f" "$PERCENT")
        
        COLOR="red"
        if [ "$INT_PERCENT" -ge 80 ]; then
            COLOR="brightgreen"
        elif [ "$INT_PERCENT" -ge 60 ]; then
            COLOR="yellowgreen"
        elif [ "$INT_PERCENT" -ge 40 ]; then
            COLOR="yellow"
        fi
        
        echo ""
        echo "Coverage: ${INT_PERCENT}% ($COLOR)"
        
        # Generate badge via shields.io (inside the html/ dir so it's deployed with Pages)
        BADGE_URL="https://img.shields.io/badge/coverage-${INT_PERCENT}%25-${COLOR}.svg"
        curl -sL "$BADGE_URL" -o "$REPORT_DIR/html/badge.svg"
        echo "Badge saved to $REPORT_DIR/badge.svg"
    fi
    
    echo ""
    echo "HTML report written to $REPORT_DIR/index.html"
fi

echo ""
echo "Run 'cargo llvm-cov --open' to view in browser."
