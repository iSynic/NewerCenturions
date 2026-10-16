#requires -Version 7.0
param(
    [ValidateSet('Test', 'Check', 'Clippy', 'Build', 'Rustc', 'Run')][string]$Action = 'Test',
    [ValidateSet('centurions-replay-recorder')][string]$Binary,
    [string]$Package,
    [string]$Filter,
    [string]$TestTarget,
    [switch]$OriginalFixtures,
    [string]$ManifestPath,
    [string]$TargetDirectory,
    [switch]$Library,
    [switch]$Ignored,
    [switch]$AllTargets,
    [switch]$Release,
    [ValidateRange(1, 64)][int]$BuildJobs = [Math]::Min(64, [Environment]::ProcessorCount),
    [switch]$AllowCompilerParallelism = $true,
    [switch]$Timings,
    [switch]$JsonDiagnostics,
    [switch]$CompilerPhaseDiagnostics,
    [string[]]$CompilerArguments = @(),
    [ValidateRange(1, 3600)][int]$MaximumBuildSeconds = 600,
    [ValidateRange(512, 65536)][int]$MaximumCompilerMemoryMiB = 16384,
    [string]$MetricsPath,
    [ValidateRange(16, 4096)][int]$MaximumMemoryMiB = 512,
    [ValidateRange(1, 600)][int]$MaximumTestSeconds = 60
)
$ErrorActionPreference = 'Stop'
if ($Action -eq 'Run' -and ($Package -ne 'centurions-core' -or !$Binary)) { throw 'Run requires the headless core recorder.' }
if ($Binary -and $Action -ne 'Run') { throw 'The recorder binary requires Run.' }
if ($CompilerPhaseDiagnostics -and $Action -ne 'Rustc') { throw 'Compiler phase diagnostics require -Action Rustc.' }
if ($Filter -and $Action -ne 'Test') { throw 'A test filter requires -Action Test.' }
if ($TestTarget -and ($Action -ne 'Test' -or $Library)) { throw 'A named test target requires -Action Test without -Library.' }
$taskRoot = [IO.Path]::GetFullPath((Split-Path $PSScriptRoot -Parent))
$cargoPath = (Get-Command cargo -CommandType Application | Select-Object -First 1).Source
$startInfo = [Diagnostics.ProcessStartInfo]::new($cargoPath)
$startInfo.WorkingDirectory = $taskRoot
$startInfo.UseShellExecute = $false
$startInfo.CreateNoWindow = $true
$startInfo.RedirectStandardOutput = $true
$startInfo.RedirectStandardError = $true
foreach ($argument in @($Action.ToLowerInvariant(), '--jobs', $BuildJobs.ToString())) { $startInfo.ArgumentList.Add($argument) }
if ($Binary) {
    $startInfo.ArgumentList.Add('--example')
    $startInfo.ArgumentList.Add($Binary)
}
if ($Timings) { $startInfo.ArgumentList.Add('--timings') }
if ($JsonDiagnostics) {
    $startInfo.ArgumentList.Add('--message-format')
    $startInfo.ArgumentList.Add('json')
}
if ($ManifestPath) {
    $startInfo.ArgumentList.Add('--manifest-path')
    $startInfo.ArgumentList.Add($ManifestPath)
} elseif ($Package) {
    $startInfo.ArgumentList.Add('-p')
    $startInfo.ArgumentList.Add($Package)
} else {
    $startInfo.ArgumentList.Add('--workspace')
}
if ($TargetDirectory) {
    $startInfo.ArgumentList.Add('--target-dir')
    $startInfo.ArgumentList.Add($TargetDirectory)
}
if ($OriginalFixtures) {
    $startInfo.ArgumentList.Add('--features')
    $startInfo.ArgumentList.Add('original-fixtures')
}
if ($Library) { $startInfo.ArgumentList.Add('--lib') }
if ($TestTarget) {
    $startInfo.ArgumentList.Add('--test')
    $startInfo.ArgumentList.Add($TestTarget)
}
if ($AllTargets) { $startInfo.ArgumentList.Add('--all-targets') }
if ($Release) { $startInfo.ArgumentList.Add('--release') }
if ($Filter) { $startInfo.ArgumentList.Add($Filter) }
if ($Action -eq 'Test') {
    foreach ($argument in @('--', '--test-threads=1', '--nocapture')) { $startInfo.ArgumentList.Add($argument) }
    if ($Ignored) { $startInfo.ArgumentList.Add('--ignored') }
} elseif ($Action -eq 'Clippy') {
    foreach ($argument in @('--', '-D', 'warnings')) { $startInfo.ArgumentList.Add($argument) }
}
if ($CompilerArguments.Count -and $Action -ne 'Rustc') { throw 'Compiler arguments require -Action Rustc.' }
if ($CompilerPhaseDiagnostics -or $CompilerArguments.Count) {
    $startInfo.ArgumentList.Add('--')
    if ($CompilerPhaseDiagnostics) { $startInfo.ArgumentList.Add('-Ztime-passes') }
    foreach ($argument in $CompilerArguments) { $startInfo.ArgumentList.Add($argument) }
}

# The wrapper cannot start Cargo until it belongs to the invocation's job.
# This also covers rustup launching the real Cargo executable immediately.
$bootstrapCommand = @'
$ErrorActionPreference = 'Stop'
try {
    if ([Console]::ReadLine() -cne 'start') { exit 125 }
    $guardCargoPath = [Environment]::GetEnvironmentVariable('CENTURIONS_GUARD_CARGO_PATH', 'Process')
    $guardCargoArguments = @(ConvertFrom-Json -InputObject ([Environment]::GetEnvironmentVariable('CENTURIONS_GUARD_CARGO_ARGUMENTS', 'Process')))
    if (-not $guardCargoPath -or -not $guardCargoArguments.Count) { throw 'Missing guarded Cargo invocation.' }
    [Environment]::SetEnvironmentVariable('CENTURIONS_GUARD_CARGO_PATH', $null, 'Process')
    [Environment]::SetEnvironmentVariable('CENTURIONS_GUARD_CARGO_ARGUMENTS', $null, 'Process')
    & $guardCargoPath @guardCargoArguments
    exit $LASTEXITCODE
} catch {
    [Console]::Error.WriteLine($_.ToString())
    exit 1
}
'@
$bootstrapInfo = [Diagnostics.ProcessStartInfo]::new((Join-Path $PSHOME 'pwsh.exe'))
$bootstrapInfo.WorkingDirectory = $taskRoot
$bootstrapInfo.UseShellExecute = $false
$bootstrapInfo.CreateNoWindow = $true
$bootstrapInfo.RedirectStandardInput = $true
$bootstrapInfo.RedirectStandardOutput = $true
$bootstrapInfo.RedirectStandardError = $true
foreach ($argument in @('-NoLogo', '-NoProfile', '-NonInteractive', '-EncodedCommand',
    [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($bootstrapCommand)))) {
    $bootstrapInfo.ArgumentList.Add($argument)
}
$bootstrapInfo.Environment['CENTURIONS_GUARD_CARGO_PATH'] = $cargoPath
if ($CompilerPhaseDiagnostics) { $bootstrapInfo.Environment['RUSTC_BOOTSTRAP'] = '1' }
$bootstrapInfo.Environment['CENTURIONS_GUARD_CARGO_ARGUMENTS'] = ConvertTo-Json -InputObject @($startInfo.ArgumentList) -Compress

if (-not ('CenturionsBuild.CargoJob' -as [type])) {
    Add-Type -TypeDefinition @'
using System;
using System.ComponentModel;
using System.Runtime.InteropServices;
using Microsoft.Win32.SafeHandles;

namespace CenturionsBuild
{
    public sealed class CargoJob : IDisposable
    {
        [StructLayout(LayoutKind.Sequential)]
        private struct BasicLimits
        {
            public long ProcessUserTime;
            public long JobUserTime;
            public uint Flags;
            public UIntPtr MinimumWorkingSet;
            public UIntPtr MaximumWorkingSet;
            public uint ActiveProcessLimit;
            public UIntPtr Affinity;
            public uint PriorityClass;
            public uint SchedulingClass;
        }
        [StructLayout(LayoutKind.Sequential)]
        private struct IoCounters
        {
            public ulong ReadOperations, WriteOperations, OtherOperations;
            public ulong ReadBytes, WriteBytes, OtherBytes;
        }
        [StructLayout(LayoutKind.Sequential)]
        private struct ExtendedLimits
        {
            public BasicLimits Basic;
            public IoCounters Io;
            public UIntPtr ProcessMemoryLimit, JobMemoryLimit;
            public UIntPtr PeakProcessMemory, PeakJobMemory;
        }

        [DllImport("kernel32.dll", CharSet = CharSet.Unicode, ExactSpelling = true, SetLastError = true)]
        private static extern SafeFileHandle CreateJobObjectW(IntPtr attributes, string name);
        [DllImport("kernel32.dll", SetLastError = true)]
        [return: MarshalAs(UnmanagedType.Bool)]
        private static extern bool SetInformationJobObject(
            SafeFileHandle job, int informationClass, ref ExtendedLimits limits, uint size);
        [DllImport("kernel32.dll", SetLastError = true)]
        [return: MarshalAs(UnmanagedType.Bool)]
        private static extern bool AssignProcessToJobObject(SafeFileHandle job, IntPtr process);
        [DllImport("kernel32.dll", SetLastError = true)]
        [return: MarshalAs(UnmanagedType.Bool)]
        private static extern bool IsProcessInJob(
            IntPtr process, SafeFileHandle job, [MarshalAs(UnmanagedType.Bool)] out bool member);
        [DllImport("kernel32.dll", SetLastError = true)]
        [return: MarshalAs(UnmanagedType.Bool)]
        private static extern bool QueryInformationJobObject(
            SafeFileHandle job, int informationClass, IntPtr buffer, uint size, out uint returned);

        private readonly SafeFileHandle handle;
        public CargoJob()
        {
            handle = CreateJobObjectW(IntPtr.Zero, null);
            if (handle.IsInvalid) throw new Win32Exception(Marshal.GetLastWin32Error());
            try
            {
                var limits = new ExtendedLimits();
                limits.Basic.Flags = 0x2000; // JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
                if (!SetInformationJobObject(handle, 9, ref limits,
                    (uint)Marshal.SizeOf(typeof(ExtendedLimits))))
                    throw new Win32Exception(Marshal.GetLastWin32Error());
            }
            catch { handle.Dispose(); throw; }
        }
        public void Assign(IntPtr process)
        {
            if (!AssignProcessToJobObject(handle, process))
                throw new Win32Exception(Marshal.GetLastWin32Error());
        }
        public bool Contains(IntPtr process)
        {
            bool member;
            if (!IsProcessInJob(process, handle, out member))
                throw new Win32Exception(Marshal.GetLastWin32Error());
            return member;
        }
        public int[] ProcessIds()
        {
            for (int size = 4096; size <= 1048576; size *= 2)
            {
                IntPtr buffer = Marshal.AllocHGlobal(size);
                try
                {
                    uint returned;
                    if (!QueryInformationJobObject(handle, 3, buffer, (uint)size, out returned))
                    {
                        int error = Marshal.GetLastWin32Error();
                        if (error == 234 || error == 122) continue;
                        throw new Win32Exception(error);
                    }
                    int count = Marshal.ReadInt32(buffer, 4);
                    if (count < 0 || count > (size - 8) / IntPtr.Size)
                        throw new InvalidOperationException("Invalid job process list.");
                    var ids = new int[count];
                    for (int i = 0; i < count; i++)
                        ids[i] = checked((int)Marshal.ReadIntPtr(buffer, 8 + i * IntPtr.Size).ToInt64());
                    return ids;
                }
                finally { Marshal.FreeHGlobal(buffer); }
            }
            throw new InvalidOperationException("Job process list exceeds one MiB.");
        }
        public void Dispose() { handle.Dispose(); }
    }
}
'@
}

$job = $null
$guardProcess = $null
$jobAssigned = $false
$streams = @()
function Receive-CargoOutput($stream) {
    $lines = 0
    while ($stream.Task -and $stream.Task.IsCompleted -and $lines -lt 100) {
        $line = $stream.Task.GetAwaiter().GetResult()
        if ($null -eq $line) { $stream.Task = $null; break }
        Write-Output $line
        $lines++
        $stream.Task = $stream.Reader.ReadLineAsync()
    }
}
$elapsed = [Diagnostics.Stopwatch]::StartNew()
$lastProgressSeconds = 0
$testStarted = @{}
$peakResidentBytes = 0L
$peakCompilerBytes = 0L
$completed = $false
$failure = $null
try {
    $job = [CenturionsBuild.CargoJob]::new()
    $guardProcess = [Diagnostics.Process]::Start($bootstrapInfo)
    try {
        $job.Assign($guardProcess.Handle)
        $jobAssigned = $true
    } catch {
        if (-not $guardProcess.HasExited) { $guardProcess.Kill($true) }
        throw
    }
    $streams = @(
        @{ Reader = $guardProcess.StandardOutput; Task = $guardProcess.StandardOutput.ReadLineAsync() },
        @{ Reader = $guardProcess.StandardError; Task = $guardProcess.StandardError.ReadLineAsync() }
    )
    try {
        $guardProcess.PriorityClass = [Diagnostics.ProcessPriorityClass]::BelowNormal
        if (-not $AllowCompilerParallelism) { $guardProcess.ProcessorAffinity = [IntPtr]1 }
    } catch {
        if (-not $guardProcess.HasExited) { throw }
    }
    $guardProcess.StandardInput.WriteLine('start')
    $guardProcess.StandardInput.Flush()
    $guardProcess.StandardInput.Close()
    Write-Output "Starting Cargo $Action; jobs=$BuildJobs; compiler parallelism=$AllowCompilerParallelism."
    while (-not $guardProcess.HasExited) {
        if ($elapsed.Elapsed.TotalSeconds -gt $MaximumBuildSeconds) {
            $failure = "Cargo $Action exceeded the $MaximumBuildSeconds second build budget."
            break
        }
        foreach ($stream in $streams) { Receive-CargoOutput $stream }
        $activeProcesses = [Collections.Generic.HashSet[string]]::new()
        $compilerBytes = 0L
        foreach ($processId in $job.ProcessIds()) {
            $ownedProcess = Get-Process -Id $processId -ErrorAction SilentlyContinue
            if (-not $ownedProcess) { continue }
            try {
                # A live handle plus actual job membership prevents PID reuse
                # from classifying another invocation's process as ours.
                if (-not $job.Contains($ownedProcess.Handle)) { continue }
                $null = $activeProcesses.Add($ownedProcess.ProcessName)
                $ownedProcess.PriorityClass = [Diagnostics.ProcessPriorityClass]::BelowNormal
                if ($ownedProcess.ProcessName -in @('rustc', 'rustdoc', 'clippy-driver', 'rust-lld', 'link', 'lld-link', 'cl', 'clang', 'gcc')) {
                    $ownedProcess.Refresh()
                    $compilerBytes += $ownedProcess.WorkingSet64
                }
                if (-not $AllowCompilerParallelism) { $ownedProcess.ProcessorAffinity = [IntPtr]1 }
                $executable = $ownedProcess.MainModule.FileName
                $headlessRecorder = $Action -eq 'Run' -and [IO.Path]::GetFileName($executable) -ieq "$Binary.exe"
                if (!$headlessRecorder -and $executable -notmatch '[\\/]deps[\\/][^\\/]+\.exe$' -and
                    $executable -notmatch '[\\/]rustdoctest[^\\/]*[\\/].*\.exe$' -and
                    [IO.Path]::GetFileName($executable) -ine 'rust_out.exe') { continue }
                $ownedProcess.ProcessorAffinity = [IntPtr]1
                $identity = '{0}:{1}' -f $ownedProcess.Id, $ownedProcess.StartTime.ToUniversalTime().Ticks
                if (-not $testStarted.ContainsKey($identity)) {
                    $testStarted[$identity] = [Diagnostics.Stopwatch]::StartNew()
                }
                $ownedProcess.Refresh()
                $peakResidentBytes = [Math]::Max($peakResidentBytes, $ownedProcess.WorkingSet64)
                if ($ownedProcess.WorkingSet64 -gt ([long]$MaximumMemoryMiB * 1MB)) {
                    $failure = "Test $($ownedProcess.ProcessName) exceeded $MaximumMemoryMiB MiB of resident memory."
                } elseif ($testStarted[$identity].Elapsed.TotalSeconds -gt $MaximumTestSeconds) {
                    $failure = "Test $($ownedProcess.ProcessName) exceeded $MaximumTestSeconds seconds."
                }
            } catch {
                if ($ownedProcess.HasExited) { continue }
                throw
            } finally {
                $ownedProcess.Dispose()
            }
            if ($failure) { break }
        }
        if ($failure) { break }
        $peakCompilerBytes = [Math]::Max($peakCompilerBytes, $compilerBytes)
        if ($compilerBytes -gt ([long]$MaximumCompilerMemoryMiB * 1MB)) {
            $failure = "Compilers exceeded $MaximumCompilerMemoryMiB MiB of aggregate resident memory."
            break
        }
        if ($elapsed.Elapsed.TotalSeconds - $lastProgressSeconds -ge 30) {
            Write-Output ("Cargo {0}: {1:N0}s elapsed; active: {2}" -f $Action, $elapsed.Elapsed.TotalSeconds, (($activeProcesses | Sort-Object) -join ', '))
            $lastProgressSeconds = $elapsed.Elapsed.TotalSeconds
        }
        Start-Sleep -Milliseconds 200
        $guardProcess.Refresh()
    }
    if ($failure) { throw $failure }
    $guardProcess.WaitForExit()
    if ($guardProcess.ExitCode -ne 0) { $failure = "Cargo $Action failed with exit code $($guardProcess.ExitCode)."; throw $failure }
    $completed = $true
} finally {
    try {
        # Closing the invocation's job also kills surviving descendants when
        # Cargo has already exited. Do this before draining inherited pipes.
        if ($job) { $job.Dispose() }
        if ($guardProcess) {
            if (-not $jobAssigned -and -not $guardProcess.HasExited) { $guardProcess.Kill($true) }
            $null = $guardProcess.WaitForExit(5000)
        }
        foreach ($stream in $streams) {
            $drain = [Diagnostics.Stopwatch]::StartNew()
            while ($stream.Task -and $drain.Elapsed.TotalSeconds -lt 5) {
                Receive-CargoOutput $stream
                if ($stream.Task) { Start-Sleep -Milliseconds 20 }
            }
            if ($stream.Task) { Write-Warning 'Output stream did not close within five seconds after process cleanup.' }
        }
        Write-Output ("Cargo {0} duration: {1:N1}s." -f $Action, $elapsed.Elapsed.TotalSeconds)
        if ($MetricsPath) {
            $metricsFile = [IO.Path]::GetFullPath($MetricsPath)
            $null = New-Item -ItemType Directory -Path ([IO.Path]::GetDirectoryName($metricsFile)) -Force
            @{
                action = $Action; completed = $completed; failure = $failure
                elapsed_seconds = $elapsed.Elapsed.TotalSeconds
                compiler_peak_bytes = $peakCompilerBytes; test_peak_bytes = $peakResidentBytes
                build_jobs = $BuildJobs; compiler_parallelism = [bool]$AllowCompilerParallelism
                maximum_build_seconds = $MaximumBuildSeconds; maximum_compiler_memory_mib = $MaximumCompilerMemoryMiB
                arguments = @($startInfo.ArgumentList)
            } | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath $metricsFile
        }
        if ($testStarted.Count) {
            Write-Output ("Observed test peak: {0:N1} MiB; memory ceiling: {1} MiB; one logical CPU." -f ($peakResidentBytes / 1MB), $MaximumMemoryMiB)
        }
    } finally {
        if ($guardProcess) { $guardProcess.Dispose() }
    }
}
