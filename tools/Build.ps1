#requires -Version 7.0
param([switch]$Package, [switch]$OriginalFixtures, [switch]$ExtendedScenarios)
$ErrorActionPreference = 'Stop'
$taskRoot = Split-Path $PSScriptRoot -Parent
Push-Location $taskRoot
try {
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
        python -m unittest discover -s tests/python -p test_translation_plan.py
    }
    if ($LASTEXITCODE) { throw 'Parser tests failed' }
    cargo fmt --check
    if ($LASTEXITCODE) { throw 'Formatting failed' }
    if ($verifyOriginals) {
        & "$PSScriptRoot/Run-Tests.ps1" -OriginalFixtures -MaximumTestSeconds 300
    } else {
        & "$PSScriptRoot/Run-Tests.ps1" -MaximumTestSeconds 300
    }
    if ($LASTEXITCODE) { throw 'Tests failed' }
    & "$PSScriptRoot/Run-Tests.ps1" -ManifestPath vendor/miniquad/Cargo.toml -Library -TargetDirectory target/miniquad-check
    if ($LASTEXITCODE) { throw 'Windows keyboard compatibility tests failed' }
    if ($verifyOriginals) {
        & "$PSScriptRoot/Run-Cargo.ps1" -Action Clippy -AllTargets -OriginalFixtures
    } else {
        & "$PSScriptRoot/Run-Cargo.ps1" -Action Clippy -AllTargets
    }
    if ($LASTEXITCODE) { throw 'Clippy failed' }
    & "$PSScriptRoot/Run-Cargo.ps1" -Action Build -Release
    if ($LASTEXITCODE) { throw 'Release build failed' }
    if ($verifyOriginals) {
        if ($ExtendedScenarios) {
            foreach ($seed in @(12, 1)) {
                & '.\target\release\centurions-tools.exe' verify-opening $taskRoot $seed
                if ($LASTEXITCODE) { throw "Opening workflow failed for seed $seed" }
            }
            & '.\target\release\centurions-tools.exe' verify-fight $taskRoot
            if ($LASTEXITCODE) { throw 'Combat and resumed replay failed' }
            & '.\target\release\centurions-tools.exe' verify-day-two $taskRoot
            if ($LASTEXITCODE) { throw 'Day-two routes failed' }
            & '.\target\release\centurions-tools.exe' verify-sustained $taskRoot
            if ($LASTEXITCODE) { throw 'Authored supply, reload and subsequent battle workflow failed' }
        }
        & '.\target\release\centurions-tools.exe' verify-stage-one $taskRoot
        if ($LASTEXITCODE) { throw 'First mission transition and resumed replay failed' }
    }
    if ($Package) {
        python tools/package.py
        if ($LASTEXITCODE) { throw 'Packaging failed' }
    }
} finally { Pop-Location }
