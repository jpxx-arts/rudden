# Análise de Desempenho (Performance Analysis)

Este documento analisa possíveis gargalos de desempenho na aplicação Rudden e propõe otimizações para garantir que a ferramenta permaneça rápida e escalável.

---

## Gargalo 01: Busca de Tarefas por ID

### Análise
Atualmente, a estrutura `ToDoList` armazena as tarefas em um `Vec<Task>`. Funções como `update_task` e `remove_task` precisam encontrar uma tarefa específica pelo seu `id`. Para fazer isso, elas iteram sobre o vetor do início ao fim até encontrar a tarefa correspondente.

- **Estrutura de Dados Atual:** `tasks: Vec<Task>`
- **Operação:** Encontrar tarefa pelo ID.
- **Complexidade Algorítmica (Big O):** **O(n)** no pior caso (linear), onde 'n' é o número total de tarefas. Em média, a complexidade também é O(n).

Isso significa que, se a lista de tarefas crescer para milhares de itens, o tempo para encontrar uma tarefa específica aumentará proporcionalmente. Para uma ferramenta de CLI, qualquer atraso perceptível (>100ms) pode degradar a experiência do usuário.

### Proposta de Otimização
Substituir o `Vec<Task>` por uma estrutura de dados mais eficiente para buscas por chave, como um `HashMap`. O `id` da tarefa seria a chave, e o objeto `Task` o valor.

- **Nova Estrutura de Dados:** `tasks: HashMap<u32, Task>`
- **Operação:** Encontrar tarefa pelo ID.
- **Nova Complexidade Algorítmica (Big O):** **O(1)** em tempo médio (constante). O pior caso é O(n), mas é extremamente raro e ocorre apenas com uma quantidade massiva de colisões de hash.

### Impacto da Otimização
A busca, atualização e remoção de tarefas se tornariam operações de tempo constante, independentemente do número de tarefas na lista. Isso garante que a aplicação permaneça instantânea, mesmo com um histórico de milhares de tarefas. A desvantagem é um uso ligeiramente maior de memória devido ao overhead do `HashMap` e a perda da ordem de inserção inerente (que poderia ser mantida usando `IndexMap` se a ordem for um requisito).

#### Exemplo de Refatoração (`update_task`):

**Antes (O(n)):**
```rust
pub fn update_task(&mut self, id: u32, ...) -> bool {
    if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
        // ... Lógica de atualização
        return true;
    }
    false
}
```

**Depois (O(1)):**
```rust
// Supondo tasks: HashMap<u32, Task>
pub fn update_task(&mut self, id: u32, ...) -> bool {
    if let Some(task) = self.tasks.get_mut(&id) {
        // ... Lógica de atualização
        return true;
    }
    false
}
```

---

## Gargalo 02: Operações de Leitura/Escrita do Arquivo de Tarefas

### Análise
O design atual adota uma estratégia de "Read-Modify-Write" para a persistência.
1. `ToDoList::load()`: Lê o arquivo `.rudden` inteiro e o desserializa para a memória no início de cada execução. (Complexidade **O(n)**, onde n é o número de tarefas).
2. `ToDoList::save()`: Serializa e escreve a lista de tarefas *inteira* de volta para o arquivo a cada modificação (adição, atualização, remoção). (Complexidade **O(n)**).

Para listas de tarefas muito grandes (dezenas de milhares de itens), essas operações de I/O podem se tornar um gargalo significativo, tornando os comandos lentos. Reescrever o arquivo inteiro a cada pequena mudança também é ineficiente e aumenta o desgaste em mídias de armazenamento como SSDs.

### Proposta de Otimização
Implementar um sistema de persistência baseado em **log de eventos (append-only log)**. Em vez de reescrever o arquivo inteiro, cada modificação seria anexada como um novo evento no final do arquivo.

- **Formato do Evento:** Um JSON ou formato binário representando a operação (ex: `{ "type": "ADD", "task": { ... } }`, `{ "type": "UPDATE", "id": 5, "status": "Finished" }`).
- **Operação de Salvar:** Anexar um novo evento ao final do arquivo. A complexidade se torna **O(1)** (tempo constante), pois depende apenas do tamanho do novo dado, não do número total de tarefas.
- **Operação de Carregar:** Ler o arquivo de log do início ao fim e "reproduzir" os eventos para reconstruir o estado atual em memória. A complexidade continua sendo **O(m)**, onde 'm' é o número de eventos no log.

Para evitar que o arquivo de log cresça indefinidamente, uma **estratégia de compactação** seria necessária. Periodicamente (ex: uma vez por dia, ou quando o log atinge um certo tamanho), a aplicação poderia ler todos os eventos, calcular o estado final e reescrever o arquivo de log com um único evento de "snapshot" contendo todas as tarefas atuais, descartando o log antigo.

### Impacto da Otimização
A maioria dos comandos que modificam o estado se tornaria significativamente mais rápida, pois a escrita no disco seria uma operação de anexação (append), que é muito mais veloz do que reescrever o arquivo inteiro. A inicialização da aplicação (`load`) manteria a mesma complexidade, mas a experiência geral do usuário para comandos rápidos como `add` e `update` seria aprimorada. Isso também tornaria a aplicação mais robusta contra corrupção de dados; se a aplicação falhar durante a escrita, apenas o último evento pode ser corrompido, em vez do arquivo inteiro.