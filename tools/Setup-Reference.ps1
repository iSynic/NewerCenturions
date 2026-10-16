param(
    [Parameter(Mandatory=$true)][string]$EmulatorRoot,
    [Parameter(Mandatory=$true)][string]$RomPath,
    [string]$MingwRoot = 'C:\msys64\mingw32',
    [switch]$Launch
)
$ErrorActionPreference = 'Stop'
$taskRoot = Split-Path $PSScriptRoot -Parent
$referenceRoot = Join-Path $taskRoot 'reference'
$image = Join-Path $referenceRoot 'centurions.hfv'
$original = Join-Path $EmulatorRoot 'runtime\system76.hfv'
$rom = (Resolve-Path -LiteralPath $RomPath).Path
New-Item -ItemType Directory -Force -Path $referenceRoot | Out-Null
if (-not (Test-Path -LiteralPath $image)) {
    if (Get-Process BasiliskII -ErrorAction SilentlyContinue) { throw 'Close the source emulator cleanly before cloning its volume.' }
    Copy-Item -LiteralPath $original -Destination $image
    $sourceHash = (Get-FileHash -LiteralPath $original -Algorithm SHA256).Hash
    $copyHash = (Get-FileHash -LiteralPath $image -Algorithm SHA256).Hash
    if ($sourceHash -ne $copyHash) { throw 'Reference copy hash mismatch' }
}
$prefs = Join-Path $referenceRoot 'BasiliskII_prefs'
Set-Content -LiteralPath $prefs -Encoding utf8 -Value @(
    "rom $rom", "disk $image", 'bootdriver 0', 'ramsize 134217728', 'frameskip 1',
    'modelid 14', 'cpu 4', 'fpu true', 'screen win/800/600', 'displaycolordepth 8',
    'nosound false', 'nocdrom true', 'nogui true', 'enableextfs false',
    "xpram $referenceRoot\xpram.dat", 'sdlrender software', 'sdl_vsync false',
    'title Centurions Reference'
)
$receipt = Join-Path $referenceRoot 'baseline.json'
if (-not (Test-Path -LiteralPath $receipt)) {
    @{volume_sha256=(Get-FileHash -LiteralPath $image -Algorithm SHA256).Hash;
      rom_sha256=(Get-FileHash -LiteralPath $rom -Algorithm SHA256).Hash;
      source_volume=$original;registration='User registered before disk copy'} |
        ConvertTo-Json | Set-Content -LiteralPath $receipt -Encoding utf8
}
if ($Launch) {
    $binaryRoot = Join-Path $referenceRoot 'bin'
    New-Item -ItemType Directory -Force -Path $binaryRoot | Out-Null
    Copy-Item -LiteralPath (Join-Path $EmulatorRoot 'src\macemu\BasiliskII\src\Windows\BasiliskII.exe') -Destination $binaryRoot
    Copy-Item -LiteralPath (Join-Path $EmulatorRoot 'src\macemu\BasiliskII\src\Windows\divinity-harness-client.exe') -Destination $binaryRoot
    foreach ($dll in @('libgcc_s_dw2-1.dll', 'libwinpthread-1.dll')) {
        Copy-Item -LiteralPath (Join-Path $MingwRoot ('bin\' + $dll)) -Destination $binaryRoot
    }
    $exe = Join-Path $binaryRoot 'BasiliskII.exe'
    $previousPath = $env:PATH
    try {
        $env:PATH = "$(Join-Path $MingwRoot 'bin');$previousPath"
        $process = Start-Process -FilePath $exe -ArgumentList @('--config', ('"' + $prefs + '"'), '--divinity-harness') -WorkingDirectory $referenceRoot -WindowStyle Hidden -RedirectStandardOutput (Join-Path $referenceRoot 'stdout.log') -RedirectStandardError (Join-Path $referenceRoot 'stderr.log') -PassThru
        $process.PriorityClass = 'BelowNormal'
    } finally { $env:PATH = $previousPath }
    $process.Id | Set-Content -LiteralPath (Join-Path $referenceRoot 'basilisk.pid')
    Write-Output "Reference emulator PID $($process.Id)"
}
