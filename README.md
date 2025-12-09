# Rudden

Este é um guia rápido para compilar e executar o projeto.

## Pré-requisitos

  * [Rust](https://www.rust-lang.org/tools/install) instalado.

## 1\. Como Compilar o Projeto

Para compilar o projeto, navegue até a pasta raiz e execute o seguinte comando no seu terminal. Isso irá criar um executável otimizado na pasta `target/release`.

```bash
cargo build --release
```

## 2\. Como Executar

Após compilar, você pode executar o programa diretamente com o Cargo.

Para ver todas as opções disponíveis e entender como usar o programa, execute o comando abaixo:

```bash
./target/release/rudden --help
```

> **Nota:** para executar em modo debug, utilize o comando cargo run -- desired_command

---

## 3\. Testes e Garantia de Qualidade

O projeto possui uma suíte de testes unitários completa para garantir a qualidade e a estabilidade do código.

### Executando os Testes

Para executar todos os testes unitários, utilize o comando padrão do Cargo:

```bash
cargo test
```

### Análise de Cobertura de Testes

Utilizamos `cargo-tarpaulin` para gerar relatórios de cobertura de testes. Para gerar o seu, siga os passos abaixo:

```bash
# 1. Instale a ferramenta (se ainda não tiver)
cargo install cargo-tarpaulin

# 2. Execute a análise na raiz do projeto
# O relatório será gerado em formato HTML e LCOV
cargo tarpaulin --ignore-tests --out Html --out Lcov
```

Após a execução, você pode abrir o arquivo `tarpaulin-report.html` em seu navegador para uma visualização interativa dos resultados.
