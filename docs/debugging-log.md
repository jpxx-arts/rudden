# Log de Depuração (Debugging Log)

Este documento registra os bugs identificados, investigados e corrigidos durante o desenvolvimento do projeto Rudden.

---

## Bug 01: Falha no parsing de tarefas com vírgula

- **ID do Bug:** `RUD-BUG-001`
- **Data de Identificação:** 2025-12-09
- **Severidade:** Crítica
- **Status:** CORRIGIDO

### Descrição
A aplicação falhava ao carregar tarefas se o nome de uma tarefa contivesse uma vírgula. Isso corrompia o estado da aplicação, pois a lógica de parsing tratava a vírgula no nome como um delimitador de campo.

### Passos para Reprodução
1.  Adicionar uma tarefa com uma vírgula no nome: `rudden add -m "Refatorar o módulo, incluindo os testes"`
2.  Executar um comando que lê as tarefas, como `rudden show`.
3.  A aplicação interpretava a linha de forma incorreta, resultando em dados corrompidos ou pânico.

### Investigação
A causa raiz foi identificada na implementação de `FromStr` para a struct `Task`, que usava `s.splitn(4, ',')`. Um teste unitário (`test_todolist_save_and_load`) foi criado para replicar o bug, confirmando que a desserialização falhava com nomes de tarefa contendo vírgulas.

### Correção (Code Diff)
A solução foi refatorar a lógica de parsing para usar `rsplitn(3, ',')`, isolando primeiro os campos da direita (`importance` e `status`) que garantidamente não contêm vírgulas, e depois tratando o restante da string.

```diff
--- a/src/task.rs
+++ b/src/task.rs
@@ -82,26 +82,35 @@
 impl FromStr for Task {
     type Err = String;
 
-    fn from_str(s: &str) -> Result<Self, Self::Err> {
-        let parts: Vec<&str> = s.splitn(4, ',').collect();
-        if parts.len() != 4 {
-            return Err("Incorrect line format".to_string());
-        }
+   fn from_str(s: &str) -> Result<Self, Self::Err> {
+        // We split from the right, because status and importance don't contain commas.
+        let mut parts: Vec<&str> = s.rsplitn(3, ',').collect();
+        if parts.len() != 3 {
+            return Err("Incorrect line format: couldn't split into 3 parts from right".to_string());
+        }
+        // Because we used rsplit, the vector is reversed. Let's fix that.
+        parts.reverse();
+        // Now, parts[0] is "id,name", parts[1] is "status", parts[2] is "importance"
+        let id_and_name = parts[0];
+        let status_str = parts[1];
+        let importance_str = parts[2];
 
-        let id = parts[0].parse::<u32>().map_err(|e| e.to_string())?;
-        let name = parts[1].to_string();
-        let status = Status::from_str(parts[2]).map_err(|_| "Invalid Status".to_string())?;
+        let id_name_parts: Vec<&str> = id_and_name.splitn(2, ',').collect();
+        if id_name_parts.len() != 2 {
+            return Err("Incorrect line format: couldn't split id and name".to_string());
+        }
+
+        let id = id_name_parts[0].parse::<u32>().map_err(|e| e.to_string())?;
+        let name = id_name_parts[1].to_string();
+        let status = Status::from_str(status_str).map_err(|_| "Invalid Status".to_string())?;
         let importance =
-            Importance::from_str(parts[3]).map_err(|_| "Invalid Importance".to_string())?;
+            Importance::from_str(importance_str).map_err(|_| "Invalid Importance".to_string())?;
 
         Ok(Self {
             id,
             name,
             status,
-            importance,
-        })
-    }
+            importance
+        })
+    }
 }
```

### Verificação
O teste unitário `test_todolist_save_and_load` passou com sucesso após a correção.

---

## Bug 02: Condição de Corrida (Race Condition) na Modificação de Tarefas

- **ID do Bug:** `RUD-BUG-002`
- **Data de Identificação:** 2025-12-09
- **Severidade:** Alta
- **Status:** IDENTIFICADO

### Descrição
Se dois processos `rudden` forem executados simultaneamente para modificar o arquivo de tarefas, pode ocorrer uma condição de corrida. O último processo a escrever sobrescreverá as alterações do primeiro, resultando em perda de dados, pois o padrão "Read-Modify-Write" não é atômico.

### Passos para Reprodução
1.  Crie uma lista com várias tarefas.
2.  Em um terminal, execute `sleep 1 && rudden add -m "Nova tarefa A"`
3.  Imediatamente em outro terminal, execute `rudden rm --id 1`
4.  O estado final do arquivo será inconsistente.

### Investigação
A análise do fluxo de dados em `lib.rs` (`try_run`) confirma o padrão "Read-Modify-Write" sem um mecanismo de bloqueio (locking) para serializar o acesso ao arquivo `.rudden`.

### Correção (Proposta)
Implementar um mecanismo de file locking antes de qualquer operação de escrita, utilizando uma crate como `fs2` para adquirir um bloqueio exclusivo no arquivo `.rudden`.

---

## Bug 03: `check` não encontra commits antigos por ler o reflog

- **ID do Bug:** `RUD-BUG-003`
- **Data de Identificação:** 2025-12-09
- **Severidade:** Alta
- **Status:** CORRIGIDO

### Descrição
A funcionalidade `rudden check` é pouco confiável, pois só encontra commits que estão no reflog (`.git/logs/HEAD`), que é um registro de curto prazo das movimentações do `HEAD`. Commits mais antigos, embora presentes no histórico, não são encontrados.

### Passos para Reprodução
1.  Encontre uma mensagem de um commit antigo no repositório com `git log`.
2.  Adicione uma nova tarefa com essa mensagem de commit exata.
3.  Rode `rudden check`.
4.  O comando reporta "No tasks to update...", falhando em atualizar a tarefa.

### Investigação
O teste manual falhou consistentemente para commits mais antigos. A análise da função `commands::check_tasks` mostrou que a fonte de dados é o arquivo de reflog, que é inadequado para pesquisar todo o histórico de commits.

### Correção (Proposta)
Refatorar a função `check_tasks` para parar de ler o reflog e, em vez disso, invocar o comando `git log` como um subprocesso e capturar seu `stdout`.

**Exemplo:**
```rust
use std::process::Command;

fn get_git_log() -> io::Result<String> {
    let output = Command::new("git")
        .arg("log")
        .arg("--pretty=format:%s") // Pega apenas a linha de assunto
        .output()?;
    // ... (tratamento de erro)
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
```

Esta abordagem garante que a verificação seja feita contra o histórico real e completo do repositório.