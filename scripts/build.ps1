
# build.ps1 - Build the kernel and copy ELF to boot folder
$ErrorActionPreference = 'Stop'

# Move to the project root (one level up from the scripts folder)
Set-Location -Path (Join-Path $PSScriptRoot "..")

# Build release ELF for the custom target
cargo build --release --target x86_64-unknown-none

# Ensure the boot directory exists (relative to the project root)
$bootDir = Join-Path -Path (Get-Location) "boot"
New-Item -ItemType Directory -Force -Path $bootDir | Out-Null

# Copy the produced ELF (named after the crate, without extension) to boot/kernel.elf
$elfPath = Join-Path -Path (Join-Path "target" "x86_64-unknown-none\release") "jinn"
Copy-Item -Path $elfPath -Destination (Join-Path $bootDir "kernel.elf") -Force

Write-Host "Build completed. ELF copied to $bootDir\kernel.elf"
