# Jinn OS

Jinn é um sistema operacional experimental em Rust para `x86_64`, projetado como um kernel mínimo com foco em boot simples, scheduler, timer e gerenciamento de memória.

## Objetivo do projeto

- Kernel `#![no_std]` escrito em Rust
- Inicialização via Limine e saída em VGA texto
- Estrutura modular para memória, timer, scheduler, PIC/IDT e boot
- Documentação técnica para facilitar revisão e evolução

## Estado atual

- Protótipo de kernel com scheduler cooperativo e tarefas de exemplo
- Infraestrutura de boot e scripts de geração de ISO disponíveis
- Recursos como drivers, serviços e suporte completo de hardware ainda em desenvolvimento
- Repositório preparado para revisão, mas não é um sistema operacional completo para produção

## Estrutura do repositório

- `src/` - código do kernel principal
- `boot/`, `iso_root/`, `limine/` - infraestrutura de boot e configuração
- `kernel/` - crate auxiliar ou legado
- `docs/` - documentação técnica e visão do projeto
- `scripts/` - scripts de build, geração de ISO e execução
- `Cargo.toml`, `rust-toolchain.toml` - configuração de build Rust
- `linker.ld`, `target.json` - linker e configuração de target

## Como compilar

1. Instale o Rust e `nightly`:
   ```powershell
   rustup toolchain install nightly
   ```
2. Adicione o target bare-metal:
   ```powershell
   rustup target add x86_64-unknown-none
   ```
3. Compile o kernel:
   ```powershell
   cargo build --release --target x86_64-unknown-none
   ```

## Como executar

- Use `scripts/run.ps1` para gerar a ISO e executar no QEMU.
- O script pode baixar os binários do Limine automaticamente caso não estejam presentes.
- O arquivo `iso_root/boot/jinn_kernel/jinn` é gerado durante o processo de build/ISO e não deve ser incluído no repositório.

## ISO gerada

- `jinn.iso` está agora disponível na raiz do repositório.
- SHA256: `80A99E4E54C6146222FE21007FA629B707E1337A26863AF98BADCC66310B371D`

## Documentação

- `docs/README.md` — índice dos documentos técnicos
- `docs/PROJECT_OVERVIEW.md` — visão geral do projeto e seus objetivos
- `docs/architecture/` — visão arquitetural do Jinn OS
- `docs/kernel/` — design do kernel e do scheduler
- `docs/services/` — serviços planejados em espaço de usuário

## Contribuindo

- Veja `CONTRIBUTING.md` para diretrizes básicas
- Abra issues para bugs e propostas
- Faça commits pequenos e atômicos
- Execute `cargo build --release --target x86_64-unknown-none` antes de enviar mudanças

## Licença

- Veja `LICENSE` para os termos do projeto

## Versão em Inglês

- [README_EN.md](README_EN.md)

## Observações

- Arquivos de build e logs são filtrados pelo `.gitignore`
- A pasta `limine/` contém artefatos de bootloader de terceiros e não precisa ser incluída no repositório principal
