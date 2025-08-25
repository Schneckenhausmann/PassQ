#!/bin/bash
# generate_secrets.sh
# Script to generate secure secrets for PassQ production deployment

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

SECRETS_DIR="secrets"

echo "=== PassQ Secrets Generator ==="
echo "This script will generate secure secrets for production deployment."
echo

# Check if secrets directory exists
if [ ! -d "$SECRETS_DIR" ]; then
    echo -e "${RED}Error: secrets/ directory not found.${NC}"
    echo "Please run this script from the project root directory."
    exit 1
fi

# Function to generate a secret file
generate_secret() {
    local filename=$1
    local description=$2
    local command=$3
    local filepath="$SECRETS_DIR/$filename"
    
    if [ -f "$filepath" ]; then
        echo -e "${YELLOW}Warning: $filepath already exists.${NC}"
        read -p "Overwrite? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            echo "Skipping $filename"
            return
        fi
    fi
    
    echo "Generating $description..."
    eval "$command" > "$filepath"
    chmod 600 "$filepath"
    echo -e "${GREEN}✓ Generated $filename${NC}"
}

# Function to prompt for manual secret
prompt_secret() {
    local filename=$1
    local description=$2
    local filepath="$SECRETS_DIR/$filename"
    
    if [ -f "$filepath" ]; then
        echo -e "${YELLOW}Warning: $filepath already exists.${NC}"
        read -p "Overwrite? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            echo "Skipping $filename"
            return
        fi
    fi
    
    echo "Enter $description (leave empty to skip):"
    read -s secret_value
    
    if [ -n "$secret_value" ]; then
        echo -n "$secret_value" > "$filepath"
        chmod 600 "$filepath"
        echo -e "${GREEN}✓ Saved $filename${NC}"
    else
        echo "Skipped $filename"
    fi
}

echo "Generating core secrets..."
echo

# Generate core secrets
generate_secret "postgres_password.txt" "PostgreSQL password" "openssl rand -base64 32 | tr -d '\n'"
generate_secret "jwt_secret.txt" "JWT secret" "openssl rand -base64 64 | tr -d '\n'"
generate_secret "encryption_key.txt" "Encryption key" "openssl rand -base64 32 | tr -d '\n'"
generate_secret "audit_secret.txt" "Audit secret" "openssl rand -base64 32 | tr -d '\n'"

echo
echo "Optional OAuth secrets (press Enter to skip):"
echo

# Prompt for OAuth secrets
prompt_secret "microsoft_client_secret.txt" "Microsoft OAuth client secret"
prompt_secret "google_client_secret.txt" "Google OAuth client secret"

echo
echo "Optional SMTP secrets (press Enter to skip):"
echo

# Prompt for SMTP secrets
prompt_secret "smtp_password.txt" "SMTP password"

echo
echo -e "${GREEN}=== Secrets generation completed ===${NC}"
echo
echo "Generated secrets:"
ls -la "$SECRETS_DIR"/*.txt 2>/dev/null || echo "No .txt files found"

echo
echo -e "${YELLOW}Important reminders:${NC}"
echo "1. Never commit .txt files to version control"
echo "2. Backup these secrets securely"
echo "3. Set up secret rotation schedule"
echo "4. Update your docker-compose.yml to use Docker secrets"
echo "5. Test your application with the new secrets"
echo
echo "For production deployment instructions, see secrets/README.md"