# Crea un acceso directo en el Escritorio para SophieQuant.
#
#   powershell -ExecutionPolicy Bypass -File scripts\crear-acceso-directo.ps1
#
# Por defecto copia el binario release a una carpeta estable del perfil
# (%LOCALAPPDATA%\Programs\SophieQuant) y apunta el acceso a esa copia: asi el
# acceso sobrevive a un `cargo clean` o a mover el repositorio.
#
# Con -ModoDev crea en su lugar un acceso que levanta `npm run tauri dev`
# (servidor de Vite + recarga en caliente).
#
# Opciones:
#   -ModoDev            acceso que arranca el modo desarrollo
#   -Nombre "Mi acceso"  nombre del acceso directo
#   -Destino <ruta>      carpeta donde se crea el acceso (por defecto, Escritorio)
#   -Instalador <msi>    instala el MSI en vez de copiar el ejecutable

param(
    [switch]$ModoDev,
    [string]$Nombre = "SophieQuant",
    [string]$Destino = [Environment]::GetFolderPath('Desktop'),
    [string]$Instalador = ""
)

$ErrorActionPreference = 'Stop'
$raiz = Split-Path -Parent $PSScriptRoot
$exeRelease = Join-Path $raiz "src-tauri\target\release\sophiequant.exe"
$establo = Join-Path $env:LOCALAPPDATA "Programs\SophieQuant"

$shell = New-Object -ComObject WScript.Shell
$acceso = $shell.CreateShortcut((Join-Path $Destino "$Nombre.lnk"))

if ($ModoDev) {
    # modo desarrollo: levanta el servidor de Vite y la app con recarga en caliente
    $cmd = (Get-Command cmd.exe).Source
    $acceso.TargetPath = $cmd
    $acceso.Arguments = "/c npm run tauri dev"
    $acceso.WorkingDirectory = $raiz
    $acceso.IconLocation = if (Test-Path $exeRelease) { "$exeRelease,0" } else { "shell32.dll,13" }
    $acceso.Description = "SophieQuant (modo desarrollo)"
    Write-Host "Acceso directo (desarrollo): $(Join-Path $Destino "$Nombre.lnk")" -ForegroundColor Green
    Write-Host "  destino : $($acceso.TargetPath) $($acceso.Arguments)"
    exit 0
}

# ── modo instalador (MSI): deja la app instalada y el acceso en la instalacion ──
if ($Instalador) {
    if (-not (Test-Path $Instalador)) { Write-Error "no encuentro el instalador $Instalador"; exit 1 }
    Write-Host "Instalando $Instalador ..." -ForegroundColor Cyan
    $p = Start-Process msiexec.exe -ArgumentList "/i `"$Instalador`" /quiet" -Wait -PassThru
    if ($p.ExitCode -ne 0) { Write-Error "la instalacion fallo (codigo $($p.ExitCode))"; exit 1 }
    $instalado = Join-Path $env:ProgramFiles "SophieQuant\SophieQuant.exe"
    if (Test-Path $instalado) {
        $acceso.TargetPath = $instalado
        $acceso.WorkingDirectory = Split-Path -Parent $instalado
        $acceso.IconLocation = "$instalado,0"
        $acceso.Description = "SophieQuant - plataforma de trading algoritmico (instalada)"
        $acceso.Save()
        Write-Host "Instalada en $instalado" -ForegroundColor Green
    } else {
        Write-Host "Instalada. Busca SophieQuant en el menu Inicio." -ForegroundColor Green
    }
    exit 0
}

# ── modo portable: copia estable + acceso ──
if (-not (Test-Path $exeRelease)) {
    Write-Host "No encuentro el binario release en $exeRelease" -ForegroundColor Yellow
    Write-Host "Compilalo con:  npm run tauri build" -ForegroundColor Yellow
    exit 1
}

New-Item -ItemType Directory -Force -Path $establo | Out-Null
$exe = Join-Path $establo "SophieQuant.exe"
Copy-Item $exeRelease $exe -Force

$acceso.TargetPath = $exe
$acceso.WorkingDirectory = $establo
$acceso.IconLocation = "$exe,0"
$acceso.Description = "SophieQuant - plataforma de trading algoritmico (local)"
$acceso.Save()

Write-Host "Acceso directo creado: $(Join-Path $Destino "$Nombre.lnk")" -ForegroundColor Green
Write-Host "  ejecutable : $exe"
Write-Host "  (copia estable; el binario del repositorio es $exeRelease)"
