# Security Core — Jinn OS

## 1. Visão Geral

O `Security Core` define o modelo de segurança do Jinn: baseado em capacidades, zero-trust e isolamento por serviços. Fornece autenticação, autorização e auditoria para IPC e operações sensíveis.

## 2. Objetivos

- Implementar Zero Trust e Least Privilege.
- Gerenciar Capability Tokens.
- Isolar serviços e drivers.
- Oferecer logs de auditoria e políticas de autorização finas.

## 3. Responsabilidades

- Emissão e verificação de tokens de capability.
- Validação de chamadas IPC e checagem de permissões.
- Gerenciamento de sandboxes e namespaces.

## 4. Arquitetura Proposta

- `Auth Manager`: responsável por identities e chaves criptográficas.
- `Capability Store`: armazenamento imutável/assinado de capabilities.
- `Policy Engine`: regras expressas e verificáveis para autorização.

Fluxo:

  [Service/Driver] -> [IPC request] -> [Security Core] -> allow/deny

## 5. Estruturas de Dados

- `CapabilityToken { id, owner, target, perms, expiry, signature }`
- `ACL { resource, subject, perms }`
- `AuditEntry { ts, subject, action, resource, result }`

## 6. Fluxo de Funcionamento

1. Serviço pede capability para recurso X.
2. Auth Manager autentica identidade (cryptographic attestation ou signature).
3. Policy Engine avalia regra e emite `CapabilityToken` assinado.

Na requisição IPC:

1. Receiver valida signature e perms.
2. Se permissões suficientes, request é processado; caso contrário, negado e log gerado.

## 7. Interfaces Públicas

- `sec_request_capability(subject, resource, perms)`
- `sec_validate_token(token)`
- `sec_register_service(identity, attestation)`

## 8. Integração com outros componentes

- IPC Core: todas as mensagens passam por checagem de autorização.
- Process Supervisor: verifica identidade de serviços iniciados.
- Kernel: aplica limites a syscalls que exigem capabilities.

## 9. Segurança

- Tokens assinado por chave do sistema; rotação de chaves cuidada por root service.
- Minimizando privilégio: por padrão, serviços não recebem capabilities adicionais.

## 10. Escalabilidade

- Capability Store projetado com cache local por serviço para leituras rápidas.
- Policy Engine com cache de decisões para evitar recomputação frequente.

## 11. Futuras Evoluções

- Suporte a attestation baseada em hardware (TPM/SEVs) para drivers confiáveis.
- Políticas regidas por expressão de domínio (DSL) verificável.

## 12. Comparação com sistemas modernos

- seL4: modelo de capabilities forte; Jinn adota tokens assinados e auth centralizada.
- Qubes: isolamento por domínios; Jinn busca facilidade de gerenciamento de capabilities via políticas dinâmicas.

## 13. Pseudocódigo

Issuance flow:

```pseudo
request_cap(subject, resource, perms):
  if not auth_manager.authenticate(subject):
    return deny
  decision = policy_engine.evaluate(subject, resource, perms)
  if decision.allow:
    token = capability.issue(subject, resource, perms)
    return token
  else:
    log_audit(...)
    return deny

validate_on_ipc(token, action):
  if verify_signature(token) and token.perms includes action:
    return allow
  else:
    return deny

```

## 14. Diagramas ASCII

Capability issuance:

  [Service] -> [Auth Manager] -> [Policy Engine] -> [CapabilityToken]

IPC validation:

  [Sender] -> [IPC Core] -> [Security Core validate] -> [Receiver]

## 15. Considerações para implementação em Rust

- Tokens representados por structs imutáveis e assinados; validação com `ring`/`rust-crypto` no user-space ou módulo crypto seguro.
- Minimizar `unsafe` no Security Core; criptografia e parsing de tokens isolados em crates auditáveis.
- Comunicação entre Security Core e Policy Engine via canais assinados; usar tipos fortemente tipados para evitar parsing bugs.

---

Arquivo: [Security Core](./security-core.md)
