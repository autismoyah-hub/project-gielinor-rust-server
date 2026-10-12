$ErrorActionPreference = 'Stop'
$script:agent2Root = Split-Path $PSScriptRoot -Parent
function Assert-ServerBuildCapacity {
    if (-not ('Agent2Memory' -as [type])) {
        Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class Agent2Memory {
    [StructLayout(LayoutKind.Sequential)] public class Status {
        public uint length=64, load;
        public ulong totalPhysical, availablePhysical, totalPage, availablePage,
                     totalVirtual, availableVirtual, extended;
    }
    [DllImport("kernel32.dll")] public static extern bool GlobalMemoryStatusEx([In,Out] Status m);
}
'@
    }
    $serverMemory = New-Object Agent2Memory+Status
    if (-not [Agent2Memory]::GlobalMemoryStatusEx($serverMemory)) { throw 'Cannot verify memory capacity.' }
    if ($serverMemory.availablePhysical -lt 4GB) { throw 'Build deferred: less than 4 GiB available RAM.' }
    $compilers = @(Get-Process -Name cargo,rustc,kotlinc -ErrorAction SilentlyContinue)
    if ($compilers.Count -gt 0) { throw 'Build deferred: another compiler is active.' }
    # A running client is not a compiler. Permit only the identified Agent 1 binary,
    # with stricter system headroom while it is running; never alter its process.
    $clientRuns = @(Get-Process -Name gielinor-client,gielinor_client -ErrorAction SilentlyContinue)
    foreach ($clientRun in $clientRuns) {
        if ($clientRun.Path -ne 'C:\Projects\OSRSRSPS\rust-client\target\release\gielinor-client.exe') {
            throw 'Build deferred: an unclassified client workload is active.'
        }
    }
    if ($clientRuns.Count -gt 0 -and $serverMemory.availablePhysical -lt 6GB) { throw 'Build deferred: client runtime active and less than 6 GiB available RAM.' }
    # Only exact, identified server runtimes are exempt; never alter those processes.
    # Unknown Java processes may be long-running Agent 1 Gradle daemons or tests.
    $knownLiveStart = [DateTimeOffset]::Parse('2026-10-02T18:17:04.3695206-07:00').UtcDateTime
    # Agent 1's PID file and thread dump identify its isolated GameServer, port 43617.
    # This is a running test world, not a Gradle/compiler process. Pin its start time
    # as well as PID so a later PID reuse fails closed.
    $knownClientWorldStart = [DateTimeOffset]::Parse('2026-10-12T06:10:00.7936528Z').UtcDateTime
    # Read-only jcmd identified SocialNotificationVectors: main has exited, DestroyJavaVM
    # waits for an idle ThreadPoolExecutor. Fresh thread evidence confirms no compiler
    # tasks. Bound this exemption to identity, RSS and CPU growth during preflight;
    # cumulative JVM housekeeping CPU is not evidence of an active build.
    # Evidence: work/client-reference-7072-threads-recheck.txt.
    $knownIdleReferenceStart = [DateTimeOffset]::Parse('2026-10-12T08:19:22.1213947Z').UtcDateTime
    $javaBefore = @(Get-Process -Name java -ErrorAction SilentlyContinue)
    $idleBefore = $javaBefore | Where-Object { $_.Id -eq 7072 }
    $idleCpuBefore = if ($idleBefore) { $idleBefore.CPU } else { 0 }
    $otherJava = @($javaBefore | Where-Object {
        -not (($_.Id -eq 8428 -and $_.StartTime.ToUniversalTime() -eq $knownLiveStart) -or
              ($_.Id -eq 17720 -and $_.StartTime.ToUniversalTime() -eq $knownClientWorldStart) -or
              ($_.Id -eq 7072 -and $_.StartTime.ToUniversalTime() -eq $knownIdleReferenceStart -and $_.WorkingSet64 -lt 128MB))
    })
    if ($otherJava.Count -gt 0) { throw 'Build deferred: an unclassified Java workload is active.' }
    $samples = @(Get-Counter '\Processor(_Total)\% Processor Time' -SampleInterval 1 -MaxSamples 3 | ForEach-Object { $_.CounterSamples.CookedValue })
    $cpuCeiling = if ($clientRuns.Count -gt 0) { 30 } else { 70 }
    if (($samples | Measure-Object -Average).Average -gt $cpuCeiling) { throw "Build deferred: sampled system CPU exceeds $cpuCeiling%." }
    # A compilation may have started while sampling system load.
    if (@(Get-Process -Name cargo,rustc,kotlinc -ErrorAction SilentlyContinue).Count -gt 0) { throw 'Build deferred: another compiler started during preflight.' }
    $otherJavaAfter = @(Get-Process -Name java -ErrorAction SilentlyContinue | Where-Object {
        -not (($_.Id -eq 8428 -and $_.StartTime.ToUniversalTime() -eq $knownLiveStart) -or
              ($_.Id -eq 17720 -and $_.StartTime.ToUniversalTime() -eq $knownClientWorldStart) -or
              ($idleBefore -and $_.Id -eq 7072 -and $_.StartTime.ToUniversalTime() -eq $knownIdleReferenceStart -and $_.WorkingSet64 -lt 128MB -and ($_.CPU - $idleCpuBefore) -le 0.25))
    })
    if ($otherJavaAfter.Count -gt 0) { throw 'Build deferred: Java workload changed during preflight.' }
    Write-Host ('Preflight: {0:N1} GiB available, CPU {1:N0}%.' -f ($serverMemory.availablePhysical / 1GB), ($samples | Measure-Object -Average).Average)
    # Only this invocation and its children are constrained. Never alter another process.
    $ownProcess = [System.Diagnostics.Process]::GetCurrentProcess()
    $ownProcess.PriorityClass = 'BelowNormal'
    $allowed = $ownProcess.ProcessorAffinity.ToInt64()
    $budget = [Math]::Max(1, [Math]::Min(2, [Math]::Floor([Environment]::ProcessorCount / 4)))
    [long]$mask = 0
    for ($bit = 0; $bit -lt 63 -and $budget -gt 0; $bit++) {
        [long]$candidate = [long]1 -shl $bit
        if (($allowed -band $candidate) -ne 0) { $mask = $mask -bor $candidate; $budget-- }
    }
    if ($mask -eq 0) { throw 'Cannot establish a CPU affinity budget.' }
    $ownProcess.ProcessorAffinity = [IntPtr]$mask
    $env:TEMP = Join-Path $script:agent2Root 'work/tmp'
    $env:TMP = $env:TEMP
    New-Item -ItemType Directory -Force -Path $env:TEMP | Out-Null
}
