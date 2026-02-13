#!/usr/bin/env bash
# ==============================================================================
# Integration Tests for NULID SQLx Examples
# ==============================================================================
#
# This script runs integration tests against real PostgreSQL, MySQL, and MariaDB
# databases using Docker containers.
#
# Usage:
#   ./scripts/integration.sh [OPTIONS]
#
# Options:
#   --postgres    Run PostgreSQL integration test only
#   --mysql       Run MySQL integration test only
#   --mariadb     Run MariaDB integration test only
#   --all         Run all integration tests (default)
#   --cleanup     Only cleanup containers, don't run tests
#   --help        Show this help message
#
# Requirements:
#   - Docker
#   - Rust toolchain
#
# ==============================================================================

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Container names
POSTGRES_CONTAINER="nulid-test-postgres"
MYSQL_CONTAINER="nulid-test-mysql"
MARIADB_CONTAINER="nulid-test-mariadb"

# Database credentials
DB_USER="nulid"
DB_PASSWORD="nulid_test_password"
DB_NAME="nulid_test"

# Ports (using non-standard ports to avoid conflicts)
POSTGRES_PORT=5433
MYSQL_PORT=3307
MARIADB_PORT=3308

# Timeouts
STARTUP_TIMEOUT=60
HEALTH_CHECK_INTERVAL=2

# ==============================================================================
# Helper Functions
# ==============================================================================

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_docker() {
    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed or not in PATH"
        exit 1
    fi

    if ! docker info &> /dev/null; then
        log_error "Docker daemon is not running"
        exit 1
    fi
}

wait_for_postgres() {
    local container=$1
    local port=$2
    local timeout=$STARTUP_TIMEOUT
    local elapsed=0

    log_info "Waiting for PostgreSQL to be ready..."

    while [ $elapsed -lt $timeout ]; do
        if docker exec "$container" pg_isready -U "$DB_USER" -d "$DB_NAME" &> /dev/null; then
            log_success "PostgreSQL is ready!"
            return 0
        fi
        sleep $HEALTH_CHECK_INTERVAL
        elapsed=$((elapsed + HEALTH_CHECK_INTERVAL))
        echo -n "."
    done

    echo ""
    log_error "PostgreSQL failed to start within ${timeout}s"
    return 1
}

wait_for_mysql() {
    local container=$1
    local timeout=$STARTUP_TIMEOUT
    local elapsed=0

    log_info "Waiting for MySQL/MariaDB to be ready..."

    while [ $elapsed -lt $timeout ]; do
        if docker exec "$container" mysqladmin ping -h localhost -u"$DB_USER" -p"$DB_PASSWORD" --silent &> /dev/null; then
            # Additional wait to ensure MySQL is fully ready for connections
            sleep 3
            log_success "MySQL/MariaDB is ready!"
            return 0
        fi
        sleep $HEALTH_CHECK_INTERVAL
        elapsed=$((elapsed + HEALTH_CHECK_INTERVAL))
        echo -n "."
    done

    echo ""
    log_error "MySQL/MariaDB failed to start within ${timeout}s"
    return 1
}

cleanup_container() {
    local container=$1
    if docker ps -a --format '{{.Names}}' | grep -q "^${container}$"; then
        log_info "Stopping and removing container: $container"
        docker stop "$container" &> /dev/null || true
        docker rm "$container" &> /dev/null || true
    fi
}

cleanup_all() {
    log_info "Cleaning up all test containers..."
    cleanup_container "$POSTGRES_CONTAINER"
    cleanup_container "$MYSQL_CONTAINER"
    cleanup_container "$MARIADB_CONTAINER"
    log_success "Cleanup complete"
}

# ==============================================================================
# Database Setup Functions
# ==============================================================================

start_postgres() {
    log_info "Starting PostgreSQL container..."
    
    cleanup_container "$POSTGRES_CONTAINER"
    
    docker run -d \
        --name "$POSTGRES_CONTAINER" \
        -e POSTGRES_USER="$DB_USER" \
        -e POSTGRES_PASSWORD="$DB_PASSWORD" \
        -e POSTGRES_DB="$DB_NAME" \
        -p "${POSTGRES_PORT}:5432" \
        postgres:16-alpine \
        > /dev/null

    wait_for_postgres "$POSTGRES_CONTAINER" "$POSTGRES_PORT"
}

start_mysql() {
    log_info "Starting MySQL container..."
    
    cleanup_container "$MYSQL_CONTAINER"
    
    docker run -d \
        --name "$MYSQL_CONTAINER" \
        -e MYSQL_USER="$DB_USER" \
        -e MYSQL_PASSWORD="$DB_PASSWORD" \
        -e MYSQL_DATABASE="$DB_NAME" \
        -e MYSQL_ROOT_PASSWORD="$DB_PASSWORD" \
        -p "${MYSQL_PORT}:3306" \
        mysql:8.0 \
        --default-authentication-plugin=mysql_native_password \
        > /dev/null

    wait_for_mysql "$MYSQL_CONTAINER"
}

start_mariadb() {
    log_info "Starting MariaDB container..."
    
    cleanup_container "$MARIADB_CONTAINER"
    
    docker run -d \
        --name "$MARIADB_CONTAINER" \
        -e MYSQL_USER="$DB_USER" \
        -e MYSQL_PASSWORD="$DB_PASSWORD" \
        -e MYSQL_DATABASE="$DB_NAME" \
        -e MYSQL_ROOT_PASSWORD="$DB_PASSWORD" \
        -p "${MARIADB_PORT}:3306" \
        mariadb:11 \
        > /dev/null

    wait_for_mysql "$MARIADB_CONTAINER"
}

# ==============================================================================
# Test Functions
# ==============================================================================

run_postgres_test() {
    log_info "Running PostgreSQL integration test..."
    
    export DATABASE_URL="postgresql://${DB_USER}:${DB_PASSWORD}@localhost:${POSTGRES_PORT}/${DB_NAME}"
    
    if cargo run --example sqlx_postgres --features sqlx-postgres; then
        log_success "PostgreSQL integration test passed!"
        return 0
    else
        log_error "PostgreSQL integration test failed!"
        return 1
    fi
}

run_mysql_test() {
    log_info "Running MySQL integration test..."
    
    export DATABASE_URL="mysql://${DB_USER}:${DB_PASSWORD}@localhost:${MYSQL_PORT}/${DB_NAME}"
    
    if cargo run --example sqlx_mysql --features sqlx-mysql; then
        log_success "MySQL integration test passed!"
        return 0
    else
        log_error "MySQL integration test failed!"
        return 1
    fi
}

run_mariadb_test() {
    log_info "Running MariaDB integration test..."
    
    export DATABASE_URL="mysql://${DB_USER}:${DB_PASSWORD}@localhost:${MARIADB_PORT}/${DB_NAME}"
    
    if cargo run --example sqlx_mysql --features sqlx-mysql; then
        log_success "MariaDB integration test passed!"
        return 0
    else
        log_error "MariaDB integration test failed!"
        return 1
    fi
}

# ==============================================================================
# Main Integration Test Functions
# ==============================================================================

test_postgres() {
    echo ""
    echo "=============================================================================="
    echo "  PostgreSQL Integration Test"
    echo "=============================================================================="
    echo ""
    
    start_postgres
    local result=0
    run_postgres_test || result=1
    cleanup_container "$POSTGRES_CONTAINER"
    
    return $result
}

test_mysql() {
    echo ""
    echo "=============================================================================="
    echo "  MySQL Integration Test"
    echo "=============================================================================="
    echo ""
    
    start_mysql
    local result=0
    run_mysql_test || result=1
    cleanup_container "$MYSQL_CONTAINER"
    
    return $result
}

test_mariadb() {
    echo ""
    echo "=============================================================================="
    echo "  MariaDB Integration Test"
    echo "=============================================================================="
    echo ""
    
    start_mariadb
    local result=0
    run_mariadb_test || result=1
    cleanup_container "$MARIADB_CONTAINER"
    
    return $result
}

test_all() {
    local failed=0
    
    echo ""
    echo "=============================================================================="
    echo "  Running All Integration Tests"
    echo "=============================================================================="
    
    test_postgres || failed=$((failed + 1))
    test_mysql || failed=$((failed + 1))
    test_mariadb || failed=$((failed + 1))
    
    echo ""
    echo "=============================================================================="
    echo "  Integration Test Summary"
    echo "=============================================================================="
    echo ""
    
    if [ $failed -eq 0 ]; then
        log_success "All integration tests passed!"
        return 0
    else
        log_error "$failed integration test(s) failed"
        return 1
    fi
}

show_help() {
    echo "NULID Integration Tests"
    echo ""
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --postgres    Run PostgreSQL integration test only"
    echo "  --mysql       Run MySQL integration test only"
    echo "  --mariadb     Run MariaDB integration test only"
    echo "  --all         Run all integration tests (default)"
    echo "  --cleanup     Only cleanup containers, don't run tests"
    echo "  --help        Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                 # Run all tests"
    echo "  $0 --postgres      # Run PostgreSQL test only"
    echo "  $0 --cleanup       # Clean up any leftover containers"
    echo ""
    echo "Environment Variables:"
    echo "  POSTGRES_PORT      PostgreSQL port (default: 5433)"
    echo "  MYSQL_PORT         MySQL port (default: 3307)"
    echo "  MARIADB_PORT       MariaDB port (default: 3308)"
}

# ==============================================================================
# Main
# ==============================================================================

main() {
    check_docker
    
    # Override ports from environment if set
    POSTGRES_PORT=${POSTGRES_PORT:-5433}
    MYSQL_PORT=${MYSQL_PORT:-3307}
    MARIADB_PORT=${MARIADB_PORT:-3308}
    
    case "${1:-all}" in
        --postgres)
            test_postgres
            ;;
        --mysql)
            test_mysql
            ;;
        --mariadb)
            test_mariadb
            ;;
        --all|all)
            test_all
            ;;
        --cleanup)
            cleanup_all
            ;;
        --help|-h)
            show_help
            ;;
        *)
            log_error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
}

# Cleanup on exit (only if not running cleanup command)
trap 'cleanup_all' EXIT

main "$@"
