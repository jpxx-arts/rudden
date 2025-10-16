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
