# run.ps1 - Gera a ISO com Limine e inicia o QEMU
$ErrorActionPreference = 'Stop'

# Força TLS 1.2 (necessário para conexões com GitHub no PowerShell antigo)
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

# ------------------------------------------------------------------
# 1) Caminho do projeto
# ------------------------------------------------------------------
$projectRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path

# ------------------------------------------------------------------
# 2) Localizar ou baixar o binário limine-bios-x86_64.bin
# ------------------------------------------------------------------
$limineBin = $null

# Procura nos caminhos mais comuns
$candidates = @(
    (Join-Path $projectRoot "limine\limine-bios-x86_64.bin"),
    (Join-Path $projectRoot "limine\bin\limine-bios-x86_64.bin"),
    (Join-Path $projectRoot "limine-bios-x86_64.bin")
)
foreach ($c in $candidates) {
    if (Test-Path $c) {
        $limineBin = $c
        Write-Host "Limine BIOS binary encontrado: $limineBin"
        break
    }
}

# Se não encontrou, baixa diretamente da release mais recente
if (-not $limineBin) {
    Write-Host "Binário não encontrado. Consultando a última release do Limine no GitHub..."
    $apiUrl  = "https://api.github.com/repos/limine-bootloader/limine/releases/latest"
    $release = Invoke-RestMethod -Uri $apiUrl -Headers @{'User-Agent' = 'PowerShell'}
    $tag     = $release.tag_name
    Write-Host "Última release: $tag"

    # URL direta do binário pré-compilado na release
    $binUrl  = "https://github.com/limine-bootloader/limine/releases/download/$tag/limine-bios-x86_64.bin"
    $destBin = Join-Path $projectRoot "limine-bios-x86_64.bin"

    Write-Host "Baixando $binUrl ..."
    Invoke-WebRequest -Uri $binUrl -OutFile $destBin -Headers @{'User-Agent' = 'PowerShell'}

    if (-not (Test-Path $destBin)) {
        Write-Error "Falha ao baixar o binário Limine."
        exit 1
    }
    $limineBin = $destBin
    Write-Host "Download concluído: $limineBin"
}

# ------------------------------------------------------------------
# 3) Definir caminhos de trabalho
# ------------------------------------------------------------------
$isoPath = Join-Path $projectRoot "jinn.iso"

# Remove ISO anterior
if (Test-Path $isoPath) { Remove-Item $isoPath -Force }

# ------------------------------------------------------------------
# 4) Localizar mkisofs / genisoimage
# ------------------------------------------------------------------
$mkiso = $null
if (Get-Command mkisofs -ErrorAction SilentlyContinue) {
    $mkiso = "mkisofs"
} elseif (Get-Command genisoimage -ErrorAction SilentlyContinue) {
    $mkiso = "genisoimage"
} else {
    # Procura em locais comuns de instalação
    $knownPaths = @(
        "C:\Program Files\Git\usr\bin\mkisofs.exe",
        "C:\msys64\usr\bin\mkisofs.exe",
        "C:\msys64\mingw64\bin\mkisofs.exe",
        "C:\cygwin64\bin\mkisofs.exe"
    )
    foreach ($p in $knownPaths) {
        if (Test-Path $p) { $mkiso = $p; break }
    }
}

if (-not $mkiso) {
    # try Windows oscdimg as a fallback (part of Windows ADK)
    if (Get-Command oscdimg -ErrorAction SilentlyContinue) {
        $mkiso = "oscdimg"
        Write-Host "mkisofs/genisoimage não encontrado, usando oscdimg como fallback."
    } else {
        Write-Error @"
'mkisofs' ou 'genisoimage' não encontrado.
Instale um deles ou o Windows ADK (fornece 'oscdimg').
Opções:
  - Git Bash     : https://git-scm.com/download/win  (já inclui mkisofs)
  - MSYS2        : https://www.msys2.org  → pacman -S mingw-w64-x86_64-cdrtools
  - Windows ADK  : https://learn.microsoft.com/en-us/windows-hardware/get-started/adk-install
Depois reinicie este PowerShell.
"@
        exit 1
    }
}
Write-Host "Usando ferramenta de criação de ISO: $mkiso"

# ------------------------------------------------------------------
# 5) Montar a estrutura de boot para a ISO
# ------------------------------------------------------------------
# O mkisofs precisa de um diretório com os arquivos a gravar
# Copiamos o binário do Limine para lá e o kernel compilado
$bootStage = Join-Path $projectRoot "iso_stage"
if (Test-Path $bootStage) { Remove-Item $bootStage -Recurse -Force }
New-Item -ItemType Directory -Path $bootStage | Out-Null

# Binário Limine (para o El Torito)
Copy-Item -Path $limineBin -Destination $bootStage -Force

# Kernel compilado (o ELF/BIN gerado pelo build.ps1)
$kernelELF = Join-Path $projectRoot "kernel\jinn"
if (-not (Test-Path $kernelELF)) {
    # Tenta o caminho alternativo gerado pelo cargo build
    $kernelELF = Join-Path $projectRoot "target\x86_64-unknown-none\release\jinn"
}
if (Test-Path $kernelELF) {
    Copy-Item -Path $kernelELF -Destination $bootStage -Force
} else {
    Write-Warning "Kernel não encontrado em $kernelELF. A ISO pode não bootar o kernel."
}

# ------------------------------------------------------------------
# 6) Gerar a ISO com El Torito (BIOS boot via Limine)
# ------------------------------------------------------------------
$limineBinName = Split-Path $limineBin -Leaf
$mkisoArgs = @(
    "-o", $isoPath,
    "-b", $limineBinName,
    "-no-emul-boot", "-boot-load-size", "4", "-boot-info-table",
    "-quiet",
    $bootStage
)

Write-Host "Gerando ISO em $isoPath ..."
& $mkiso @mkisoArgs

if (-not (Test-Path $isoPath)) {
    Write-Error "Geração da ISO falhou."
    exit 1
}
Write-Host "ISO gerada com sucesso."

# ------------------------------------------------------------------
# 7) Iniciar o QEMU
# ------------------------------------------------------------------
Write-Host "Iniciando QEMU..."
qemu-system-x86_64 `
    -cdrom  $isoPath `
    -m      512M `
    -serial stdio `
    -display sdl

# ------------------------------------------------------------------
# 8) Limpeza
# ------------------------------------------------------------------
Remove-Item $bootStage -Recurse -Force -ErrorAction SilentlyContinue
