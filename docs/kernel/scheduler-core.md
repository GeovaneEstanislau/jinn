# Scheduler Core — Jinn OS

## 1. Visão Geral

O `Scheduler Core` é o mecanismo de escalonamento do Jinn. Seu objetivo é separar o engine (mecanismo) das políticas, permitindo múltiplos perfis operacionais, adaptação dinâmica e integração com a Predictive Engine.

## 2. Objetivos

- Separar mecanismo/política.
- Suportar perfis (desktop, server, industrial, real-time).
- Integração com Predictive Engine para decisões pró-ativas.
- Baixa latência, alta vazão e determinismo configurável.
- Escalonamento eficiente em SMP e preparação para NUMA.

## 3. Responsabilidades

- Manter filas de execução e seleção do próximo thread.
- Coordenar migração de threads entre CPUs.
- Expor hooks para políticas e para a Predictive Engine.
- Garantir QoS e limites de latência.

## 4. Arquitetura Interna

- Scheduler Engine: código kernel confiável que executa a seleção de threads.
- Policy Layer: módulos carregáveis ou configuráveis que decidem prioridades e metas.
- Per-CPU runqueues e estruturas globais para balanceamento.
- Interface com Predictive Engine para prefetch e realocação pró-ativa.

Arquitetura em camadas:

  [Scheduler Engine]
        |
  [Policy Layer plugins]
        |
  [Per-CPU Runqueues] <-> [Global Balancer] <-> [Predictive Engine]

## 5. Estruturas de Dados

- `struct RunQueue` (per-CPU)
  - priority_bitmap: u64/u32 por faixa de prioridade
  - arrays/vectors de listas por prioridade
  - load_estimate: f32
- `struct GlobalBalancer`
  - cpus: list<CPU>
  - migration_candidates: lock-free queue
- `struct PolicyHandle`
  - id, config, hook pointers

Representação simplificada (Rust-like):

```rust
struct RunQueue {
    priority_bitmap: AtomicU64,
    buckets: [LinkedList<Thread>; N_PRIORITIES],
    load: AtomicF32,
}

struct GlobalBalancer { /* ... */ }
```

## 6. Fluxo de Funcionamento

1. Interrupção do timer/tick ou evento: CPU consulta sua `RunQueue`.
2. `Scheduler Engine` chama Policy Layer (callback) para escolher a classe de prioridade.
3. Thread selecionado é despachado para CPU.
4. Global Balancer executa periodicamente balanceamento/migração baseado em heurísticas e predições.

## 7. Interfaces Públicas

- `sched_register_policy(handle)` — registra política.
- `sched_yield()` — cede CPU do thread atual.
- `sched_set_affinity(tid, cpu_mask)` — define afinidade.
- Hooks: `on_context_switch`, `on_enqueue`, `on_dequeue`.

## 8. Integração com outros componentes

- Predictive Engine: fornece previsões de carga, janelas de prioridade e hints de migração.
- Process Supervisor: notifica criação/terminação de threads.
- Memory Core: informa sobre hotspots de memória para localização de threads (NUMA).

## 9. Segurança

- Policies não devem executar em contextos inseguros; hooks com validação.
- Limites de tempo e recursos para políticas carregadas dinamicamente.

## 10. Escalabilidade

- Per-CPU runqueues lock-free minimizam contenção.
- Global balancer trabalha em janelas e amostras para evitar overhead.
- Preparação para NUMA: statistik per-node e extensão do balancer.

## 11. Futuras Evoluções

- Suporte a políticas ML-based dentro da Predictive Engine.
- Estatísticas distribuídas por nó NUMA.

## 12. Comparação com sistemas modernos

- Linux CFS: similar em estimativas de load, mas Jinn separa mecanismo/política explicitamente e integra predição.
- Zircon: modelo de capacidades e IPC semelhante; Jinn foca adaptabilidade de políticas.

## 13. Pseudocódigo

Scheduler loop (simplificado):

```pseudo
on_tick(cpu):
  rq = cpu.runqueue
  if rq.has_ready():
    prio = policy.choose_priority(rq, cpu)
    thread = rq.pop(prio)
    context_switch_to(thread)
  else:
    idle()

periodic_balance():
  for cpu in cpus:
    if cpu.load > threshold:
       candidate = select_migration_candidate(cpu)
       migrate(candidate, target_cpu)

```

Adaptive Policy switching:

```pseudo
if predictive.hint == "latency_critical":
  activate_policy(low_latency_policy)
elif predictive.hint == "high_throughput":
  activate_policy(high_throughput_policy)

```

## 14. Diagramas ASCII

Topologia básica:

  CPU0 RunQ --\
  CPU1 RunQ ---- GlobalBalancer <-> PredictiveEngine
  CPU2 RunQ --/

Fluxo de escolha:

  [Interrupt] -> [Per-CPU Scheduler Engine] -> [Policy Layer] -> [Thread]

## Adaptive Policy Scheduler — Detalhes

- Engine: responsável por enfileirar, desempilhar e realizar context switch e migrações.
- Policy Layer: provê heurísticas (priority boosting, aging, bandwidth reservations, deadline enforcement).
- Perfis operacionais: coleções de parâmetros (latency target, throughput target, fairness) e módulos de decisão.

### CPU Affinity

- Afinidade por default é por processo; políticas podem ajustar afinidade para reduzir custo de cache.
- `sched_set_affinity` define máscara; migrator respeita preferências e limitações de política.

### Thread Migration

- Migração é realizada quando ganho estimado > custo (copy-queues, warm caches via Predictive Engine).
- Usar handshake com processos para re-localização de memória (NUMA hints).

### Priority Management

- Prioridades organizadas em classes + subprioridades.
- Bitmap por runqueue para seleção rápida da prioridade mais alta.

### QoS

- Reservoirs de CPU: políticas podem reservar fatia mínima de CPU para serviços críticos.
- Enforcement por contagem de tempo e throttling sob políticas.

### Latency Control

- Políticas de preemption agressiva para perfis de baixa latência.
- Deadline-aware scheduling para modos industriais (modo determinístico).

### Integration with Predictive Engine

- Predictive Engine fornece:
  - Hints de carga futura por serviço.
  - Riscos de hotspots de cache/memória.
  - Recomendações de política e afinidade.

Flow integration:

1. Predictive Engine analisa telemetria.
2. Emite hint (e.g., "short burst incoming on service X").
3. Scheduler ativa política de baixa latência temporária para threads de X.

## Considerações para implementação em Rust

- Usar `unsafe` apenas no mínimo: abstrair runqueues com tipos seguros.
- `Atomic*` e `spin`/`parking_lot` para sincronização de baixo nível.
- Interfaces FFI para políticas (drivers de política) devem usar boundary-safe APIs e validação de entrada.
- Favor `no_std` para código do kernel e dividir em crates: `jinn-sched-engine`, `jinn-sched-policy`, `jinn-sched-common`.

---

Arquivo: [Scheduler Core](./scheduler-core.md)

Arquitetura

O Núcleo do Agendador consiste em três camadas principais:

Mecanismo de Agendamento: Esta camada lida com o agendamento de processos, alocando e desalocando recursos conforme necessário.

Política de Agendamento: Esta camada define a política de agendamento específica a ser usada (por exemplo, Primeiro a Chegar, Primeiro a Ser Servido, Agendamento por Prioridade, etc.). O Núcleo do Agendador recebe as decisões de agendamento desta camada e as executa.

Interface do Motor Preditivo: Esta camada integra-se ao Motor Preditivo, recebendo resultados de modelagem preditiva e ajustando as decisões de agendamento de acordo.

Fluxo de Operação

O Motor Preditivo fornece um conjunto de modelos preditivos, que são usados ​​para determinar a decisão de agendamento ideal para cada processo.

A camada de Política de Agendamento define a política de agendamento específica a ser usada, com base em fatores como prioridade do processo, uso da CPU, consumo de memória, etc.

A camada de Mecanismo de Agendamento recebe as decisões de agendamento da camada de Política de Agendamento e as executa, alocando ou desalocando recursos do sistema conforme necessário.

O Núcleo do Agendador monitora o desempenho do processo e ajusta seu estado interno de acordo.

Principais Estruturas de Dados

Fila de Processos: Uma estrutura de dados que mantém o controle dos processos em execução, com cada entrada contendo informações relevantes, como ID do processo, prioridade e uso de recursos.

Decisão de Agendamento: Uma estrutura que representa a decisão de agendamento tomada pela camada de Política de Agendamento, incluindo o processo específico a ser agendado em seguida.

Interfaces Públicas

O Núcleo do Agendador fornece as seguintes interfaces públicas:

schedule_process(): Agenda um processo com base na política de agendamento atual.

get_current_scheduled_process(): Retorna o processo atualmente agendado.

update_predictive_model(): Atualiza o modelo preditivo usado pelo Mecanismo Preditivo.

Integração com outros componentes do Jinn

O Núcleo do Agendador integra-se com os seguintes componentes:

Mecanismo Preditivo: Recebe resultados de modelagem preditiva e ajusta as decisões de agendamento de acordo.

Supervisor de Processos: Fornece informações sobre os processos em execução, permitindo decisões de agendamento mais informadas.

Núcleo de Memória: Gerencia a alocação e desalocação de memória para todo o sistema.

Considerações de Segurança

O Núcleo do Agendador deve garantir que as informações confidenciais sejam protegidas e que os processos sejam devidamente isolados uns dos outros. Ele consegue isso por meio de:

Aplicação de níveis de privilégio e permissões de processo.

Monitoramento do uso de recursos do processo para evitar o consumo excessivo de recursos do sistema.

Colaboração com o Mecanismo Preditivo para otimizar as decisões de agendamento com base em considerações de segurança.

Implementações Futuras

Possíveis implementações futuras para o Núcleo do Agendador incluem:

Suporte para processos hierárquicos (ou seja, relações pai-filho).

Integração com um coletor de lixo para gerenciar o uso de memória de forma eficiente.

Modelagem preditiva aprimorada para decisões de agendamento mais precisas.

Exemplo de Fluxo de Execução

Suponha que temos três processos de nível de usuário, A, B e C, em execução simultânea. O fluxo de operação do Núcleo de Agendamento pode ser semelhante a este:

O Mecanismo Preditivo fornece um conjunto de modelos preditivos, indicando que A tem a maior prioridade.

A camada de Política de Agendamento define uma política de agendamento com base na prioridade do processo, alocando recursos de CPU para A.

A camada de Mecanismo de Agendamento agenda A como o próximo processo a ser executado, alocando memória e outros recursos conforme necessário.

À medida que os processos B e C ficam disponíveis, o Núcleo de Agendamento ajusta seu estado interno e os reagenda com base em sua prioridade.

Ao gerenciar cuidadosamente os recursos do sistema e integrar-se ao Mecanismo Preditivo, o Núcleo de Agendamento garante que o Jinn OS seja executado de forma eficiente, previsível e segura em uma ampla gama de cenários.