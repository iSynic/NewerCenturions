#requires -Version 7.0
param(
    [string]$Package,
    [string]$Filter,
    [string]$TestTarget,
    [switch]$OriginalFixtures,
    [string]$ManifestPath,
    [string]$TargetDirectory,
    [switch]$Library,
    [ValidateRange(1, 64)][int]$BuildJobs = [Math]::Min(64, [Environment]::ProcessorCount),
    [switch]$AllowCompilerParallelism = $true,
    [switch]$Timings,
    [ValidateRange(16, 4096)][int]$MaximumMemoryMiB = 512,
    [ValidateRange(1, 600)][int]$MaximumTestSeconds = 60
)
& "$PSScriptRoot/Run-Cargo.ps1" -Action Test @PSBoundParameters
