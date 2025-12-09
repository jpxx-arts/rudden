# Log de Depuração - Projeto Rudden

Este documento é um registro retrospectivo de bugs complexos que foram identificados e corrigidos durante o desenvolvimento do Rudden. Cada entrada detalha o problema, as técnicas de depuração utilizadas e a solução implementada.

---

## Bug 1: Remoção de Tarefa Incorreta por ID

*   **Data de Identificação:** 15/10/2023
*   **Severidade:** Alta
*   **Módulo:** `src/task.rs` (Método `remove_task`)

### Descrição do Problema

Foi reportado que ao tentar remover uma tarefa pelo seu ID, ocasionalmente a tarefa errada era removida da lista, ou nenhuma tarefa era removida, mesmo com um ID válido. Por exemplo, em uma lista com tarefas de IDs `[1, 3, 4]`, executar `rudden rm 3` às vezes removia a tarefa de ID `4`.

### Técnicas de Depuração Aplicadas

**Técnica: Teste Automatizado para Isolamento**

Para reproduzir o bug de forma consistente e isolar a causa, um teste unitário específico foi criado. O teste simulava um cenário onde a lista de tarefas não era contígua (ou seja, uma tarefa do meio havia sido removida anteriormente).

```rust
// Teste criado para isolar o bug
#[test]
fn bug_repro_remove_non_contiguous_id() {
    // Arrange
    let mut list = ToDoList::default();
    list.add_task("Task 1".into(), Importance::Normal); // id 1
    list.add_task("Task 2".into(), Importance::Normal); // id 2
    list.add_task("Task 3".into(), Importance::Normal); // id 3
    list.remove_task(2); // Agora os IDs são [1, 3]

    // Act: Tentar remover o ID 3
    list.remove_task(3);

    // Assert: A task 1 deve permanecer
    assert_eq!(list.tasks.len(), 1);
    assert_eq!(list.tasks[0].id, 1);
}
```

Ao executar este teste, ele falhou. A depuração revelou que a função de remoção estava usando o **índice** do vetor em vez do **ID** da tarefa.

### Solução (Antes e Depois)

A implementação original de `remove_task` estava usando `Vec::remove()`, que opera sobre o índice do vetor, mas recebia um ID como parâmetro. Isso causava um comportamento indefinido.

*   **Código ANTES (Com Erro):**

```rust
// src/task.rs

pub fn remove_task(&mut self, id: u32) -> bool {
    // BUG: O 'id' está sendo tratado como um índice!
    // Se o ID não corresponder ao índice, o item errado será removido.
    if id > 0 && (id as usize) <= self.tasks.len() {
        self.tasks.remove(id as usize - 1); // Errado!
        return true;
    }
    false
}
```

A solução foi reescrever a função para usar `Vec::retain()`, que filtra a lista de tarefas com base em um predicado, mantendo apenas as tarefas cujo ID *não corresponde* ao ID a ser removido.

*   **Código DEPOIS (Corrigido):**

```rust
// src/task.rs

pub fn remove_task(&mut self, id: u32) -> bool {
    let initial_len = self.tasks.len();
    // CORRETO: Filtra a lista mantendo todos os IDs, exceto o alvo.
    self.tasks.retain(|task| task.id != id);
    // Retorna true se o tamanho do vetor diminuiu (ou seja, algo foi removido)
    self.tasks.len() < initial_len
}
```

---

## Bug 2: Falha no Parse de Tarefas com Vírgulas no Nome

*   **Data de Identificação:** 02/11/2023
*   **Severidade:** Média
*   **Módulo:** `src/task.rs` (Implementação `FromStr` para `Task`)

### Descrição do Problema

O sistema falhava em ler o arquivo `.rudden` se uma tarefa contivesse vírgulas em sua descrição (ex: `rudden add "Refatorar, e depois otimizar, o módulo de parsing"`). A tarefa era salva corretamente, mas na próxima execução, o programa encontrava um erro de parse e ignorava a tarefa.

### Técnicas de Depuração Aplicadas

**Técnica: Uso de `dbg!` Estratégico**

Para entender como a linha do CSV estava sendo dividida, a macro `dbg!` foi inserida dentro da implementação `FromStr for Task`.

```rust
// src/task.rs (FromStr for Task)

// ANTES da correção, com dbg!
impl FromStr for Task {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(',').collect();
        dbg!(&parts); // <-- dbg! inserido aqui
        //...
    }
}
```

Ao rodar o programa com uma tarefa problemática, o `dbg!` revelou o seguinte output:

```
[src/task.rs:101] &parts = [
    "1",
    "Refatorar",
    " e depois otimizar",
    " o módulo de parsing",
    "pending",
    "normal"
]
```

Ficou claro que `s.split(',')` estava dividindo o nome da tarefa em múltiplos elementos, quebrando a lógica de parse que esperava um número fixo de campos.

### Solução (Antes e Depois)

A abordagem ingênua de `split(',')` era o problema. A solução foi usar `rsplitn`, que divide a string a partir da **direita**, garantindo que os campos de `status` e `importance` (que nunca contêm vírgulas) sejam separados primeiro. O resto da string, à esquerda, formaria o `id` e o `name` juntos, que poderiam então ser divididos com segurança.

*   **Código ANTES (Com Erro):**

```rust
// src/task.rs

impl FromStr for Task {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // BUG: Nomes com vírgulas quebram o split.
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() < 4 { /* ... */ }
        // ... Lógica que assume um número fixo de partes
    }
}
```

*   **Código DEPOIS (Corrigido):**

```rust
// src/task.rs

impl FromStr for Task {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // CORRETO: Divide a partir da direita, isolando os campos sem vírgula.
        let mut parts: Vec<&str> = s.rsplitn(3, ',').collect();
        if parts.len() != 3 { /* ... erro */ }
        parts.reverse(); // A ordem fica [status, importance, "id,name"] -> ["id,name", status, importance]

        let id_and_name = parts[0];
        let id_name_parts: Vec<&str> = id_and_name.splitn(2, ',').collect();
        // ... resto da lógica de parse
    }
}
```

---

## Bug 3: Pânico ao Checar Status sem Repositório Git

*   **Data de Identificação:** 18/11/2023
*   **Severidade:** Alta
*   **Módulo:** `src/commands.rs` (Função `check_tasks`)

### Descrição do Problema

Executar o comando `rudden check` em um diretório que não era um repositório Git resultava em um pânico (panic) do programa, em vez de uma mensagem de erro amigável. A mensagem do pânico era: `thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: Io(Os { code: 2, kind: NotFound, message: "No such file or directory" })'`.

### Técnicas de Depuração Aplicadas

**Técnica: Análise de Stack Trace / Debugger (LLDB)**

O pânico indicava um `.unwrap()` em um `Err`. Para localizar exatamente onde, o programa foi executado dentro de um debugger (LLDB) com `RUST_BACKTRACE=1`.

```sh
$ RUST_BACKTRACE=1 cargo run -- check
# ... (output do pânico)
   4: rudden::commands::check_tasks
   5: rudden::try_run
   6: rudden::run
# ...
```

A stack trace apontou diretamente para a função `check_tasks`. Analisando o código com o debugger, foi possível inspecionar a variável que continha o `Err` antes do `unwrap`. A linha problemática era:

`let logs_content = fs::read_to_string(".git/logs/HEAD").unwrap();`

O debugger confirmou que `fs::read_to_string` retornava um `Err(io::ErrorKind::NotFound)` quando o diretório `.git` não existia, e o `.unwrap()` causava o pânico.

### Solução (Antes e Depois)

A solução foi substituir o `.unwrap()` por um `match` ou `if let` para tratar o `Result` de forma segura, exibindo uma mensagem de erro clara para o usuário se o arquivo de logs do Git não puder ser lido.

*   **Código ANTES (Com Erro):**

```rust
// src/commands.rs

pub fn check_tasks(list: &mut ToDoList) -> Result<String, String> {
    // BUG: unwrap() causa pânico se o arquivo não existe.
    let logs_content = std::fs::read_to_string(".git/logs/HEAD").unwrap();
    
    if list.update_statuses_from_logs(&logs_content) {
        // ...
    }
    // ...
}
```

*   **Código DEPOIS (Corrigido):**

```rust
// src/commands.rs

pub fn check_tasks(list: &mut ToDoList) -> Result<String, String> {
    // CORRETO: Trata o Result de forma segura.
    match std::fs::read_to_string(".git/logs/HEAD") {
        Ok(logs_content) => {
            if list.update_statuses_from_logs(&logs_content) {
                // ... salvar a lista e retornar Ok
            } else {
                Ok("Nenhuma task foi atualizada.".to_string())
            }
        }
        Err(_) => {
            // Retorna um Erro amigável em vez de pânico.
            Err("Não foi possível ler os logs do Git. Execute em um repositório Git.".to_string())
        }
    }
}
```