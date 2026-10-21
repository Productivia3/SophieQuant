# Crea un acceso directo en el Escritorio para SophieQuant.
#
#   powershell -ExecutionPolicy Bypass -File scripts\crear-acceso-directo.ps1
#
# Por defecto apunta al binario release (standalone, sin servidor de desarrollo).
# Con -Modo Dev crea en su lugar un acceso que levanta `npm run tauri dev`.
#
# Opciones:
#   -Modo Dev            acceso que arranca el modo desarrollo
#   -Nombre "Mi acceso"  nombre del acceso directo
#   -Destino <ruta>      carpeta donde se crea (por defecto, el Escritorio)

param(
    [switch]$ModoDev,
    [string]$Nombre = "SophieQuant",
    [string]$Destino = [Environment]::GetFolderPath('Desktop')
)

$ErrorActionPreference = 'Stop'
$raiz = Split-Path -Parent $PSScriptRoot
$exe = Join-Path $raiz "src-tauri\target\release\sophiequant.exe"

if (-not (Test-Path $exe) -and -not $ModoDev) {
    Write-Host "No encuentro el binario release en $exe" -ForegroundColor Yellow
    Write-Host "Compilalo con:  cargo build --release --manifest-path src-tauri\Cargo.toml" -ForegroundColor Yellow
    Write-Host "O usa -ModoDev para crear el acceso de desarrollo." -ForegroundColor Yellow
    exit 1
}

$shell = New-Object -ComObject WScript.Shell
$acceso = $shell.CreateShortcut((Join-Path $Destino "$Nombre.lnk"))

if ($ModoDev) {
    # modo desarrollo: lanza el servidor de Vite y la app con recarga en caliente
    $cmd = (Get-Command cmd.exe).Source
    $acceso.TargetPath = $cmd
    $acceso.Arguments = "/c npm run tauri dev"
    $acceso.WorkingDirectory = $raiz
    $acceso.IconLocation = if (Test-Path $exe) { "$exe,0" } else { "shell32.dll,13" }
    $acceso.Description = "SophieQuant (modo desarrollo)"
} else {
    $acceso.TargetPath = $exe
    $acceso.WorkingDirectory = Split-Path -Parent $exe
    $acceso.IconLocation = "$exe,0"
    $acceso.Description = "SophieQuant - plataforma de trading algoritmico (local)"
}

$acceso.Save()
Write-Host "Acceso directo creado: $(Join-Path $Destino "$Nombre.lnk")" -ForegroundColor Green
Write-Host "  destino : $($acceso.TargetPath) $($acceso.Arguments)"
