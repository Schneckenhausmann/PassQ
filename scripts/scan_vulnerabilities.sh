#!/bin/bash
# scan_vulnerabilities.sh
# Automated vulnerability scanning script for PassQ Docker images

set -e

echo "=== PassQ Docker Vulnerability Scanner ==="
echo "Starting vulnerability scan at $(date)"
echo

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to check if Trivy is installed
check_trivy() {
    if ! command -v trivy &> /dev/null; then
        echo -e "${RED}Error: Trivy is not installed.${NC}"
        echo "Please install Trivy first:"
        echo "  macOS: brew install trivy"
        echo "  Linux: See https://aquasecurity.github.io/trivy/latest/getting-started/installation/"
        exit 1
    fi
}

# Function to scan a Docker image
scan_image() {
    local image=$1
    local name=$2
    
    echo -e "${YELLOW}Scanning $name ($image)...${NC}"
    echo "----------------------------------------"
    
    # Run Trivy scan
    if trivy image --severity HIGH,CRITICAL --format table "$image"; then
        echo -e "${GREEN}✓ Scan completed for $name${NC}"
    else
        echo -e "${RED}✗ Scan failed for $name${NC}"
        return 1
    fi
    
    echo
}

# Function to scan images with JSON output for CI/CD
scan_image_json() {
    local image=$1
    local name=$2
    local output_file="vulnerability-report-$(echo $name | tr ' ' '-' | tr '[:upper:]' '[:lower:]').json"
    
    echo "Generating JSON report for $name..."
    trivy image --severity HIGH,CRITICAL --format json --output "$output_file" "$image"
    echo "Report saved to: $output_file"
}

# Main scanning function
main() {
    check_trivy
    
    # Create reports directory if it doesn't exist
    mkdir -p reports
    cd reports
    
    echo "Scanning Docker images used in PassQ..."
    echo
    
    # Scan PostgreSQL image
    scan_image "postgres:15.8-alpine3.22" "PostgreSQL Database"
    
    # Scan Node.js image
    scan_image "node:18.20.4-alpine3.22" "Node.js Build Environment"
    
    # Scan Nginx image
    scan_image "nginx:1.27.2-alpine" "Nginx Web Server"
    
    # Scan Rust image
    scan_image "rust:1.82.0-slim-bookworm" "Rust Build Environment"
    
    # Scan Debian base image
    scan_image "debian:bookworm-20241202-slim" "Debian Base Image"
    
    echo -e "${GREEN}=== Vulnerability scan completed ===${NC}"
    echo "Scan completed at $(date)"
    
    # Generate JSON reports if --json flag is provided
    if [[ "$1" == "--json" ]]; then
        echo
        echo "Generating JSON reports..."
        scan_image_json "postgres:15.8-alpine3.22" "PostgreSQL Database"
        scan_image_json "node:18.20.4-alpine3.22" "Node.js Build Environment"
        scan_image_json "nginx:1.27.2-alpine" "Nginx Web Server"
        scan_image_json "rust:1.82.0-slim-bookworm" "Rust Build Environment"
        scan_image_json "debian:bookworm-20241202-slim" "Debian Base Image"
        echo -e "${GREEN}JSON reports generated in reports/ directory${NC}"
    fi
    
    cd ..
}

# Show usage if help is requested
if [[ "$1" == "--help" || "$1" == "-h" ]]; then
    echo "Usage: $0 [--json] [--help]"
    echo
    echo "Options:"
    echo "  --json    Generate JSON reports for CI/CD integration"
    echo "  --help    Show this help message"
    echo
    echo "This script scans all Docker images used in PassQ for security vulnerabilities."
    echo "Reports are saved in the 'reports/' directory."
    exit 0
fi

# Run main function
main "$@"