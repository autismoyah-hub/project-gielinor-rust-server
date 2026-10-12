# For bounded, prebuilt, sequential headless runs only. Builds retain resources.ps1.
function Assert-IntegrationRunCapacity {
    if (@(Get-Process -Name cargo,rustc,kotlinc -ErrorAction SilentlyContinue).Count -gt 0) {
        throw 'Integration deferred: another compiler is active.'
    }
    if (-not ('IntegrationMemory' -as [type])) {
        Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class IntegrationMemory {
    [StructLayout(LayoutKind.Sequential)] public class Status {
        public uint length=64, load;
        public ulong totalPhysical, availablePhysical, totalPage, availablePage,
                     totalVirtual, availableVirtual, extended;
    }
    [DllImport("kernel32.dll")] public static extern bool GlobalMemoryStatusEx([In,Out] Status m);
}
'@
    }
    $status=New-Object IntegrationMemory+Status
    if (-not [IntegrationMemory]::GlobalMemoryStatusEx($status) -or $status.availablePhysical -lt 4GB) { throw 'Integration deferred: less than 4 GiB available RAM.' }
    $samples=@(Get-Counter '\Processor(_Total)\% Processor Time' -SampleInterval 1 -MaxSamples 3 | ForEach-Object {$_.CounterSamples.CookedValue})
    $cpu=($samples | Measure-Object -Average).Average
    if ($cpu -gt 30) { throw 'Integration deferred: sampled CPU exceeds 30%.' }
    if (@(Get-Process -Name cargo,rustc,kotlinc -ErrorAction SilentlyContinue).Count -gt 0) {
        throw 'Integration deferred: another compiler started during preflight.'
    }
    $own=[Diagnostics.Process]::GetCurrentProcess()
    $own.PriorityClass='BelowNormal';$own.ProcessorAffinity=[IntPtr]3
    Write-Host ('Prebuilt headless preflight: {0:N1} GiB free, CPU {1:N0}%.' -f ($status.availablePhysical/1GB),$cpu)
}
