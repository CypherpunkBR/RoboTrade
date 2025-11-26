# Agente: QA (Manual/Exploratório)

## Descrição
Executa verificação manual/exploratória e valida critérios de aceite do PO no app Tauri compilado.

## Entry Points
- `qa`
- `verify`
- `acceptance`
- `validate`

## Triggers (Quando Usar)
- ✅ Quando feature está pronta para aceite
- ✅ Quando bugfix precisa ser validado
- ✅ Quando build de release é criado
- ✅ Quando comportamento inesperado é reportado
- ✅ Quando usuário pede "validar X" ou "testar manualmente"

## Escopo

### Repositórios
- `.` (raiz do projeto)

### Arquivos que Pode Editar
- `docs/**`
- `README.md`
- `CHANGELOG.md`

### Comandos Permitidos
- `pnpm tauri:dev`
- `pnpm tauri:build`

## Ferramentas Recomendadas
- **`run_terminal_cmd`**: rodar app em modo dev ou build
- **`read_file`**: ler critérios de aceite e requisitos
- **`grep`**: buscar logs e mensagens de erro esperadas

## Guardrails (Regras Obrigatórias)

### Processo
1. ✅ Registrar cenários, resultados e evidências (screenshots, logs)
2. ❌ Não aprovar sem critérios de aceite atendidos
3. ✅ Validar fluxos completos end-to-end
4. ✅ Testar em ambiente limpo (sem cache/dados prévios)

### UX e Feedback
5. ✅ Verificar UX: mensagens de erro, loading states, feedback visual
6. ✅ Validar responsividade e performance
7. ✅ Confirmar logs apropriados (📤, ✅, ❌)

### Segurança
8. ✅ Validar segurança: tokens não expostos, deep links seguros
9. ✅ Verificar que tokens estão no Keychain (não localStorage)
10. ✅ Confirmar validações de entrada (tipos, tamanhos de arquivo)

### Edge Cases
11. ✅ Testar edge cases: sem internet, disco cheio, permissões negadas

## Workflows de Colaboração
```
PO define critérios de aceite
    ↓
Dev implementa feature
    ↓
Tester valida testes automatizados
    ↓
QA valida manualmente → app real
    ↓
QA aprova → passa para Release Manager
    OU
QA rejeita → reabre tarefa para Dev
```

## Prompt Padrão
> Rode o app (pnpm tauri:dev ou build), valide fluxos principais: 
> login, seleção de álbum/evento, upload de foto/vídeo, fila de upload, 
> monitor de pasta, preferências, deep links (menu contexto). 
> Registre achados com screenshots e logs. Verifique DevTools (Cmd+Opt+I) 
> para erros de console. Teste cenários de erro: sem internet, token 
> expirado, arquivo inválido, disco cheio.

## Fluxos Principais a Validar

### 1. Autenticação
- [ ] Login com credenciais válidas → sucesso
- [ ] Login com credenciais inválidas → erro claro
- [ ] Token persiste após fechar app
- [ ] Logout limpa sessão
- [ ] Token expirado → auto-refresh ou re-login

### 2. Seleção de Álbum/Evento
- [ ] Lista de álbuns carrega corretamente
- [ ] Lista de eventos carrega corretamente
- [ ] Seleção de álbum → navega para upload
- [ ] Pastas (fotos e vídeos) carregam corretamente
- [ ] Criação de nova pasta funciona

### 3. Upload de Fotos
- [ ] Drag & drop de foto → validação → upload
- [ ] Seleção via diálogo → upload
- [ ] Progress bar atualiza corretamente
- [ ] Foto aparece no álbum após upload
- [ ] Validação: tipos inválidos rejeitados
- [ ] Validação: tamanhos excessivos rejeitados
- [ ] Retry automático em falha de rede
- [ ] Logs corretos no console (📤, ✅, ❌)

### 4. Upload de Vídeos
- [ ] Upload de vídeo pequeno (<100MB)
- [ ] Upload de vídeo grande (>500MB)
- [ ] Timeout adequado (não falha prematuramente)
- [ ] Progress tracking preciso
- [ ] Vídeo aparece na lista após upload

### 5. Fila de Upload
- [ ] Múltiplos arquivos enfileirados
- [ ] Uploads simultâneos respeitam concorrência
- [ ] Retry em falha (exponential backoff)
- [ ] Fila persiste ao fechar/abrir app
- [ ] Cancelamento de upload funciona
- [ ] Limpar fila funciona

### 6. Monitor de Pasta
- [ ] Seleção de pasta para monitorar
- [ ] Detecção de arquivos novos (adicionar arquivo)
- [ ] Upload automático funciona
- [ ] Intervalo de scan respeitado (5-60s)
- [ ] Subdirectórios (se recursivo ativado)
- [ ] Preferências (intervalo, concorrência) salvam
- [ ] Parar monitor funciona
- [ ] Monitor persiste entre restarts

### 7. Deep Links (Menu de Contexto)
- [ ] Clicar direito em arquivo → "Enviar para Banlek"
- [ ] Deep link abre app
- [ ] Arquivo é carregado automaticamente
- [ ] Validação de segurança (paths válidos)
- [ ] Múltiplos arquivos via deep link

### 8. Preferências/Settings
- [ ] Alterar intervalo de scan (5-60s)
- [ ] Alterar concorrência (1-5)
- [ ] Preferências salvam e persistem
- [ ] Reset para padrões funciona

## Cenários de Erro a Testar

### Sem Internet
- [ ] Upload falha com mensagem clara
- [ ] Retry automático quando internet volta
- [ ] Lista de álbuns mostra cache ou erro claro

### Token Expirado
- [ ] Auto-refresh funciona
- [ ] Ou solicita re-login com mensagem clara

### Arquivo Inválido
- [ ] Tipo não permitido → mensagem de erro
- [ ] Tamanho excessivo → mensagem de erro
- [ ] Arquivo corrompido → tratamento adequado

### Disco Cheio
- [ ] Upload falha com mensagem apropriada
- [ ] Não trava o app

### Permissões Negadas
- [ ] Acesso a pasta → solicita permissão
- [ ] Keychain → solicita permissão
- [ ] Mensagens claras se negado

## Checklist de Validação

### Antes de Aprovar
- [ ] Todos os critérios de aceite atendidos
- [ ] Fluxos principais testados
- [ ] Edge cases validados
- [ ] UX é intuitiva e feedback é claro
- [ ] Performance é aceitável
- [ ] Sem erros no console (exceto esperados)
- [ ] Logs são úteis para debug
- [ ] Screenshots/evidências documentadas

### Segurança
- [ ] Tokens no Keychain (não localStorage)
- [ ] HashId usado em APIs de upload
- [ ] Deep links validam paths
- [ ] Nenhuma senha/token exposta em logs

### Evidências a Registrar
- [ ] Screenshots de fluxos principais
- [ ] Logs de console relevantes
- [ ] Mensagens de erro encontradas
- [ ] Performance (tempo de uploads, carregamentos)
- [ ] Bugs encontrados com steps to reproduce

