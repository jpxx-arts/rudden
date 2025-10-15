# Log de Refatorações
## Refatoração #1: God Class
- **Data**: 2025-10-13
- **Code Smell**: God Class em src/main.rs
- **Técnica Aplicada**: Extract Module
- **Arquivos Afetados**: src/main.rs
- **Justificativa**: Arquivo main.rs declarava as classes Cli, Task e os comandos de ToDoList
- **Resultado**:
 - cli.rs
 - task.rs
 - command.rs
- **Impacto**: Melhor testabilidade e legibilidade
- **Testes**: Todos os testes passando

## Refatoração #2: Duplicate Code
- **Data**: 2025-10-13
- **Code Smell**: Repetição do tipo dentro da própria classe
- **Técnica Aplicada**: Introduce Abstraction
- **Justificativa**: Utilizar o tipo da classe dentro da declaração da mesma torna a manutenção mais difícil
- **Mudanças**:
 - ToDoList → Self
 - Importance → Self
 - Status → Self

 ## Refatoração #3: Feature Envy
- **Data**: 2025-10-13
- **Code Smell**: commands.rs acessa diversos atributos de task.rs
- **Técnica Aplicada**: Move Method
- **Justificativa**: Aplica corrertamente os princípios de encapsulamento
-
