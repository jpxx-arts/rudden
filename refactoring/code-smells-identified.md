# Code Smells Identificados
## 1. God Object
- **Arquivo**: src/main.rs
- **Descrição**: Arquivo main.rs assume responsabilidades demais 
- **Severidade**: Alta
- **Ferramenta**: clippy
- **Status**: Corrigido ao final do projeto

## 2. Duplicate Code
- **Arquivos**: src/task.rs
- **Linhas**: 18-123
- **Descrição**: Repetição dos atributos da classe ao invés de utilizar "Self."
- **Severidade**: Baixa
- **Ferramenta**: clippy
- **Status**: Corrigido ao final do projeto

## 3. Feature Envy
- **Arquivos**: src/task.rs e src/commands.rs
- **Linhas**: 106-140 e 1-104
- **Descrição**: Diversos métodos de commands.rs acessam recursos que deveriam ser exclusivos de task.rs
- **Severidade**: Média
- **Ferramenta**: clippy
- **Status**: Corrigido ao final do projeto

