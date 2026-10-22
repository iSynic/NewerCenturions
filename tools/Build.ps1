#requires -Version 7.0
param(
    [switch]$Package, [switch]$OriginalFixtures, [switch]$ExtendedScenarios,
    [ValidateRange(1, 64)][int]$BuildJobs = [Math]::Min(64, [Environment]::ProcessorCount),
    [switch]$AllowCompilerParallelism = $true,
    [switch]$Timings
)
$ErrorActionPreference = 'Stop'
$cargoOptions = @{ BuildJobs = $BuildJobs; AllowCompilerParallelism = $AllowCompilerParallelism; Timings = $Timings }
$taskRoot = Split-Path $PSScriptRoot -Parent
Push-Location $taskRoot
try {
    python tools/source_audit.py
    if ($LASTEXITCODE) { throw 'Test code exceeds the project ceiling' }
    $hasOriginals = (Test-Path -LiteralPath 'NewCenturions/packsmith-mapping.json') -and (Test-Path -LiteralPath 'evidence/fixtures')
    if ($OriginalFixtures -and -not $hasOriginals) { throw 'Original fixture checks require local NewCenturions and evidence/fixtures directories' }
    $verifyOriginals = $OriginalFixtures -or $hasOriginals
    if ($verifyOriginals) {
        python tools/extract.py
        if ($LASTEXITCODE) { throw 'Content extraction failed' }
        python -m unittest discover -s tests/python
    } else {
        python -m unittest discover -s tests/python -p test_extract.py
        if ($LASTEXITCODE) { throw 'Parser tests failed' }
        python -m unittest discover -s tests/python -p test_public.py
        if ($LASTEXITCODE) { throw 'Public source checks failed' }
    }
    if ($LASTEXITCODE) { throw 'Parser tests failed' }
    cargo fmt --check
    if ($LASTEXITCODE) { throw 'Formatting failed' }
    if ($verifyOriginals) {
        & "$PSScriptRoot/Run-Tests.ps1" -OriginalFixtures -MaximumTestSeconds 600 @cargoOptions
    } else {
        & "$PSScriptRoot/Run-Tests.ps1" -MaximumTestSeconds 600 @cargoOptions
    }
    if ($LASTEXITCODE) { throw 'Tests failed' }
    if ($ExtendedScenarios -or ($Package -and $verifyOriginals)) {
        if (!$verifyOriginals) { throw 'Campaign validation requires original-fixtures and installed Thunder content' }
        & "$PSScriptRoot/Run-Tests.ps1" -Package centurions-core -TestTarget native_reference -OriginalFixtures -Filter complete_campaign_replays -Ignored -MaximumTestSeconds 600 @cargoOptions
        if ($LASTEXITCODE) { throw 'Thunder campaign completion gate failed; local milestone packaging stopped' }
    }
    & "$PSScriptRoot/Run-Tests.ps1" -ManifestPath vendor/miniquad/Cargo.toml -Library -TargetDirectory target/miniquad-check @cargoOptions
    if ($LASTEXITCODE) { throw 'Windows keyboard compatibility tests failed' }
    if ($verifyOriginals) {
        & "$PSScriptRoot/Run-Cargo.ps1" -Action Clippy -AllTargets -OriginalFixtures @cargoOptions
    } else {
        & "$PSScriptRoot/Run-Cargo.ps1" -Action Clippy -AllTargets @cargoOptions
    }
    if ($LASTEXITCODE) { throw 'Clippy failed' }
    & "$PSScriptRoot/Run-Cargo.ps1" -Action Build -Release @cargoOptions
    if ($LASTEXITCODE) { throw 'Release build failed' }
    if ($Package) {
        python tools/package.py
        if ($LASTEXITCODE) { throw 'Packaging failed' }
    }
} finally { Pop-Location }
