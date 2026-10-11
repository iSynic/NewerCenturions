param([string]$GhidraRoot='F:\Ghidra\ghidra_12.1.2_PUBLIC', [switch]$Refresh)
$ErrorActionPreference='Stop'
$taskRoot=Split-Path $PSScriptRoot -Parent
Push-Location $taskRoot
try {
    python tools/decode_globals.py
    if ($LASTEXITCODE) { throw 'Startup recovery failed' }
    $headless=Join-Path $GhidraRoot 'support/analyzeHeadless.bat'
    if ($Refresh) {
        & $headless "$taskRoot\analysis\ghidra\project" CenturionsMapped -process code1-mapped.bin -noanalysis -scriptPath "$taskRoot\tools\ghidra" -postScript ExportCenturions.java "$taskRoot\analysis\ghidra\mapped-exports"
    } else {
        & $headless "$taskRoot\analysis\ghidra\project" CenturionsMapped -import "$taskRoot\analysis\ghidra\startup\code1-mapped.bin" -loader BinaryLoader -loader-baseAddr 0x100000 -processor 68000:BE:32:Centurions -scriptPath "$taskRoot\tools\ghidra" -preScript ImportMappedCenturions.java $taskRoot -postScript ExportCenturions.java "$taskRoot\analysis\ghidra\mapped-exports" -max-cpu 4 -analysisTimeoutPerFile 180
    }
    if ($LASTEXITCODE) { throw 'Mapped analysis failed' }
} finally { Pop-Location }
