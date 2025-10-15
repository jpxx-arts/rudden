# Feature Envy
## Arquivo "commands.rs" acessa muitos recursos de "task.rs"
Durante a implementação da classe Task o tipo da classe é repetido diversas vezes ao invés de utilizar o comando Self, o que dificultaria a manutenção do código caso fosse necessário renomear a classe.