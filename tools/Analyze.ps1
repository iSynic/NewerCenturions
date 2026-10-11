param(
    [string]$GhidraRoot = 'F:\Ghidra\ghidra_12.1.2_PUBLIC',
    [switch]$Refresh
)
$ErrorActionPreference = 'Stop'
$taskRoot = Split-Path $PSScriptRoot -Parent
Push-Location $taskRoot
try {
    python tools/extract.py
    if ($LASTEXITCODE) { throw 'Extraction failed' }
    python tools/prepare_ghidra.py
    if ($LASTEXITCODE) { throw 'Seed preparation failed' }
    New-Item -ItemType Directory -Force -Path analysis/ghidra/project | Out-Null
    $headless = Join-Path $GhidraRoot 'support/analyzeHeadless.bat'
    if ($Refresh) {
        & $headless "$taskRoot\analysis\ghidra\project" CenturionsMac68k -process 434f4445_1.bin -noanalysis -scriptPath "$taskRoot\tools\ghidra" -postScript ExportCenturions.java "$taskRoot\analysis\ghidra\exports"
    } else {
        & $headless "$taskRoot\analysis\ghidra\project" CenturionsMac68k -import "$taskRoot\analysis\extracted\resources\106\434f4445_1.bin" -loader BinaryLoader -loader-baseAddr 0x100000 -processor 68000:BE:32:Centurions -scriptPath "$taskRoot\tools\ghidra" -preScript ImportCenturions.java $taskRoot -postScript ExportCenturions.java "$taskRoot\analysis\ghidra\exports" -max-cpu 4 -analysisTimeoutPerFile 180
    }
    if ($LASTEXITCODE) { throw 'Ghidra analysis failed' }
} finally { Pop-Location }
