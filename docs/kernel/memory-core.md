# Memory Core — Jinn OS

## 1. Visão Geral

O `Memory Core` provê gerenciamento de memória física e virtual, pools especializados (DMA, Cache, Predictive) e serviços de compartilhamento de memória com isolamento por capacidades.

## 2. Objetivos

- Gerenciar memória física e virtual para microkernel e serviços isolados.
- Suportar alocadores eficientes para diferentes casos: pequenos objetos, páginas grandes, DMA.
- Integrar com Predictive Engine para pools proativos.
- Garantir isolamento e segurança para drivers em espaço de usuário.

## 3. Responsabilidades

- Alocação/Liberação física e virtual.
- Gerenciamento de páginas compartilhadas e capacidades.
- Suporte a lazy allocation, COW e huge pages.

## 4. Arquitetura Híbrida Proposta

- Boot/early: bitmap allocator para bootstrapping.
- Kernel runtime: buddy allocator para páginas e um slab allocator para objetos.
- Pools especializados: DMA Pool (contíguo), Cache Pool (para warming), Predictive Pool (reservas baseadas em predição).

Camadas:

  [Physical Manager (bitmap + buddy)]
            |
  [Virtual Manager (page tables, COW)]
            |
  [Slab / Cache Pools / Predictive Pools]

## 5. Estruturas de Dados

- `struct PhysRegion` { start, len, node }
- `struct BuddyAllocator` { free_lists: Vec<List<Page>> }
- `struct SlabCache` { size_class, partial, full }
- `struct Capability` { owner, permissions, page_refs }

## 6. Fluxo de Funcionamento

Exemplo: alocação de página para serviço isolado

1. Serviço requisita página via IPC.
2. Memory Core verifica capability e quotas.
3. Buddy/alocador decide bloco, marca e retorna endereço físico + mapeamento virtual.

Exemplo: COW em fork de serviço

1. Ao clonar espaço, páginas marcadas como COW.
2. Escrita dispara page fault; handler cria nova cópia via slab/buddy e atualiza page tables.

## 7. Interfaces Públicas

- `mm_alloc_pages(count, flags)`
- `mm_map(service, vaddr, paddr, perms)`
- `mm_create_shared_page(service_a, service_b, perms)`
- `mm_reserve_pool(pool_type, size)`

## 8. Integração com outros componentes

- Predictive Engine: solicita reservas temporárias (warm pages) e desenho de pools.
- Scheduler: fornece hints de afinidade para localização de memória.

## 9. Segurança

- Capabilities para páginas e regiões; só quem tem capability pode mapear ou transferir.
- DMA Pool exige validação de endereço físico e limites de IOMMU.

## 10. Escalabilidade

- Buddy por node NUMA, caches locais para reduzir cross-node traffic.
- Slabs com caches per-CPU para reduzir travamentos.

## 11. Futuras Evoluções

- Paginação por demanda baseada em predição.
- Pool sharing entre serviços com contratos de SLA.

## 12. Comparação com sistemas modernos

- Linux: usa buddy, slabs, huge pages — Jinn propõe adicionar Predictive Pool e stronger capability model similar ao seL4/Zircon.

## 13. Pseudocódigo

Alloc pages path:

```pseudo
mm_alloc_pages(n):
  node = select_node_by_policy()
  if node.buddy.has_free(n):
    p = node.buddy.alloc(n)
    return p
  else:
    try_compaction()
    return node.buddy.alloc(n)

on_page_fault(vaddr, service):
  if is_cow(vaddr):
    new_page = mm_alloc_pages(1)
    copy_contents(old_page, new_page)
    update_pagetable(service, vaddr, new_page)

```

## 14. Diagramas ASCII

Manager overview:

  [Boot Bitmap] -> [Buddy Allocator per node] -> [Slab Caches per size class]
                                      \-> [Pools: DMA, Cache, Predictive]

Shared page flow:

  Service A --(request)-> Memory Core --(cap check)-> allocate --(map)-> Service B

## 15. Considerações para implementação em Rust

- Estruturar em crates: `jinn-mm-phys`, `jinn-mm-virt`, `jinn-mm-pools`.
- Uso controlado de `unsafe` para manipulação de page tables e acesso físico.
- Tipos fortes para Capabilities e Permissions; evitar strings e IDs não verificados.
- Preferir `spin` locks para paths de baixa latência e `RwLock` para estruturas menos críticas.

---

Arquivo: [Memory Core](./memory-core.md)
