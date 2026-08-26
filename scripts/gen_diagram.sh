#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

PLANTUML_VERSION="1.2025.0"
PLANTUML_JAR="$HOME/.cache/plantuml/plantuml-${PLANTUML_VERSION}.jar"
PLANTUML_URL="https://github.com/plantuml/plantuml/releases/download/v${PLANTUML_VERSION}/plantuml-${PLANTUML_VERSION}.jar"

if ! command -v java &>/dev/null; then
    echo "Error: java is required to generate diagrams"
    echo "Install it with: sudo apt install default-jre or sudo dnf install default-jre"
    exit 1
fi

if [ ! -f "$PLANTUML_JAR" ]; then
    mkdir -p "$(dirname "$PLANTUML_JAR")"
    echo "Downloading PlantUML ${PLANTUML_VERSION}..."
    curl -sL "$PLANTUML_URL" -o "$PLANTUML_JAR"
fi

cd "$PROJECT_DIR"
java -jar "$PLANTUML_JAR" -tsvg docs/messages.plantuml -o "$PROJECT_DIR/docs/"
echo "docs/messages.svg generated"
