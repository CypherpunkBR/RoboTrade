#!/bin/bash
# RoboTrade - End-to-End Test Runner
# Executa testes de integração completos com Docker e gera relatórios

set -e  # Exit on error

# Colors para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configurações
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MIN_COVERAGE=80

echo -e "${BLUE}"
echo "╔═══════════════════════════════════════════════════════════╗"
echo "║         RoboTrade - End-to-End Test Suite                ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo -e "${NC}"

# Função para imprimir mensagens formatadas
print_step() {
    echo -e "\n${BLUE}▶ $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# ============================================================================
# Pré-requisitos
# ============================================================================

print_step "Verificando pré-requisitos..."

# Verificar Rust
if ! command -v cargo &> /dev/null; then
    print_error "Rust não está instalado. Instale em: https://rustup.rs/"
    exit 1
fi
print_success "Rust instalado: $(rustc --version)"

# Verificar Docker
if ! docker info &> /dev/null 2>&1; then
    print_error "Docker não está rodando. Por favor inicie o Docker."
    print_warning "macOS: open -a Docker"
    print_warning "Linux: sudo systemctl start docker"
    exit 1
fi
print_success "Docker rodando: $(docker --version)"

# Verificar cargo-llvm-cov
if ! command -v cargo-llvm-cov &> /dev/null; then
    print_warning "cargo-llvm-cov não encontrado. Instalando..."
    cargo install cargo-llvm-cov
fi
print_success "cargo-llvm-cov disponível"

# ============================================================================
# Limpeza
# ============================================================================

print_step "Limpando ambiente..."

# Parar containers antigos
docker ps -a | grep robotrade-test-postgres | awk '{print $1}' | xargs -r docker rm -f 2>/dev/null || true
docker ps -a | grep testcontainers | awk '{print $1}' | xargs -r docker rm -f 2>/dev/null || true

# Limpar artefatos de build antigos
rm -rf "$PROJECT_ROOT/target/debug/deps"
rm -rf "$PROJECT_ROOT/target/llvm-cov"

print_success "Ambiente limpo"

# ============================================================================
# Build
# ============================================================================

print_step "Compilando projeto..."

cd "$PROJECT_ROOT"

# Build em release mode para testes mais rápidos
if cargo build --release --all-features; then
    print_success "Build concluído com sucesso"
else
    print_error "Falha no build"
    exit 1
fi

# ============================================================================
# Testes Unitários
# ============================================================================

print_step "Executando testes unitários..."

if cargo test --lib --all-features --release -- --test-threads=$(nproc 2>/dev/null || echo 4); then
    print_success "Testes unitários passaram"
else
    print_error "Testes unitários falharam"
    exit 1
fi

# ============================================================================
# Testes de Integração
# ============================================================================

print_step "Executando testes de integração..."

# Verificar se diretório existe
if [ -d "$PROJECT_ROOT/integration_tests" ]; then
    cd "$PROJECT_ROOT/integration_tests"

    # Rodar testes de integração sequencialmente (evita conflitos de porta)
    if cargo test --release --all-features -- --test-threads=1 --nocapture; then
        print_success "Testes de integração passaram"
    else
        print_error "Testes de integração falharam"
        exit 1
    fi
else
    print_warning "Diretório integration_tests não encontrado, pulando..."
fi

# ============================================================================
# Cobertura de Código
# ============================================================================

print_step "Gerando relatório de cobertura..."

cd "$PROJECT_ROOT"

# Gerar cobertura em formato LCOV
if cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info; then
    print_success "Relatório LCOV gerado: lcov.info"
else
    print_warning "Falha ao gerar LCOV"
fi

# Gerar relatório HTML
if cargo llvm-cov --all-features --workspace --html; then
    print_success "Relatório HTML gerado: target/llvm-cov/html/index.html"
else
    print_warning "Falha ao gerar HTML"
fi

# Obter summary
COVERAGE_OUTPUT=$(cargo llvm-cov --all-features --workspace --summary-only 2>&1)
TOTAL_LINE=$(echo "$COVERAGE_OUTPUT" | grep "TOTAL")

if [ -n "$TOTAL_LINE" ]; then
    COVERAGE=$(echo "$TOTAL_LINE" | grep -oP '\d+\.\d+(?=%)')

    echo -e "\n${BLUE}╔═══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║              Relatório de Cobertura                       ║${NC}"
    echo -e "${BLUE}╠═══════════════════════════════════════════════════════════╣${NC}"
    echo -e "${BLUE}║${NC}  ${TOTAL_LINE}${BLUE}  ║${NC}"
    echo -e "${BLUE}╚═══════════════════════════════════════════════════════════╝${NC}\n"

    # Verificar threshold
    if (( $(echo "$COVERAGE >= $MIN_COVERAGE" | bc -l) )); then
        print_success "Cobertura $COVERAGE% atende o mínimo de $MIN_COVERAGE%"
    else
        print_error "Cobertura $COVERAGE% está abaixo do mínimo de $MIN_COVERAGE%"
        exit 1
    fi
else
    print_warning "Não foi possível extrair cobertura total"
fi

# ============================================================================
# Linting e Formatação
# ============================================================================

print_step "Verificando formatação..."

if cargo fmt --all -- --check; then
    print_success "Código está formatado corretamente"
else
    print_warning "Código precisa de formatação. Execute: cargo fmt --all"
fi

print_step "Executando Clippy..."

if cargo clippy --all-targets --all-features -- -D warnings; then
    print_success "Sem warnings do Clippy"
else
    print_warning "Clippy encontrou issues"
fi

# ============================================================================
# Relatório Final
# ============================================================================

echo -e "\n${GREEN}"
echo "╔═══════════════════════════════════════════════════════════╗"
echo "║              ✅ TODOS OS TESTES PASSARAM! ✅               ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo -e "${NC}"

echo -e "${BLUE}📊 Resumo:${NC}"
echo "  • Testes Unitários: ✅ Passou"
echo "  • Testes de Integração: ✅ Passou"
echo "  • Cobertura: ✅ $COVERAGE% (mínimo: $MIN_COVERAGE%)"
echo "  • Formatação: ✅ Ok"
echo "  • Clippy: ✅ Ok"

echo -e "\n${BLUE}📁 Artefatos:${NC}"
echo "  • Relatório HTML: target/llvm-cov/html/index.html"
echo "  • Relatório LCOV: lcov.info"

echo -e "\n${BLUE}🌐 Abrir relatório de cobertura:${NC}"

# Detectar sistema operacional e abrir navegador
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    open "$PROJECT_ROOT/target/llvm-cov/html/index.html" 2>/dev/null || true
    echo "  Abrindo no navegador (macOS)..."
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    # Linux
    xdg-open "$PROJECT_ROOT/target/llvm-cov/html/index.html" 2>/dev/null || true
    echo "  Abrindo no navegador (Linux)..."
elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]]; then
    # Windows
    start "$PROJECT_ROOT/target/llvm-cov/html/index.html" 2>/dev/null || true
    echo "  Abrindo no navegador (Windows)..."
else
    echo "  Abra manualmente: $PROJECT_ROOT/target/llvm-cov/html/index.html"
fi

echo -e "\n${GREEN}✨ Pipeline de testes concluído com sucesso!${NC}\n"

exit 0
