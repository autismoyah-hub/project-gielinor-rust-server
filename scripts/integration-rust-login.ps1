param([string]$Run = 'rust-login-001',[int]$Ticks = 100,[ValidateSet('tower-diary','login','render','actions','prayer','arrival','dialogue-ui','dialogue-protocol','inventory','inventory-verify','inventory-bury','floor','floor-content','floor-altar')][string]$Mode='login',[ValidatePattern('^integration-(b|floor|prep-[a-z0-9-]+)$')][string]$StateName='integration-b',[string]$ClientRoot,[string]$ServerExecutable,[string]$CacheRoot,[string]$ScenarioExecutable,[switch]$CreatorResume)
$ErrorActionPreference='Stop'
$serverRoot=Split-Path $PSScriptRoot -Parent
. "$PSScriptRoot/integration-capacity.ps1"
Assert-IntegrationRunCapacity
$root="$serverRoot/work/integration"
if (-not $CacheRoot) { $CacheRoot="$root/b-rust" }
if (-not $ClientRoot) { $ClientRoot="$root/client" }
if (-not $ServerExecutable) { $ServerExecutable="$serverRoot/work/target/debug/gielinor-server.exe" }
$scenario=if ($Mode -eq 'tower-diary') {'server_tower'} elseif ($Mode -eq 'prayer') { 'server_prayer' } elseif ($Mode -eq 'arrival') { 'server_arrival' } elseif ($Mode -like 'dialogue-*') { 'server_dialogue' } elseif ($Mode -like 'inventory*') { 'server_inventory' } elseif ($Mode -like 'floor*') { 'server_floor' } else { $null }
$scenarioBinary=if (Test-Path "$ClientRoot/bin/$scenario.exe") { "$ClientRoot/bin/$scenario.exe" } else { "$ClientRoot/target/debug/examples/$scenario.exe" }
if ($ScenarioExecutable) { $scenarioBinary=$ScenarioExecutable }
if ($scenario -and -not (Test-Path $scenarioBinary)) { throw 'Build the owned native scenario first: scripts/integration-client-build.ps1' }
$runDir="$root/runs/$Run"
if (Test-Path -LiteralPath $runDir) { throw "Run already exists: $Run" }
New-Item -ItemType Directory -Path $runDir | Out-Null
$gracefulStop=((& "$ServerExecutable" --help | Out-String).Contains('--stop-file'))
$arguments=@('--game-cache',"$CacheRoot/cache/game",'--js5-cache',"$CacheRoot/cache/js5",'--symbols',"$CacheRoot/symbols",'--rsa-exponent',"$root/keys/exponent.bin",'--rsa-modulus',"$root/keys/modulus.bin",'--argon2',"$serverRoot/work/native/argon2.dll",'--state-name',$StateName,'--bind','127.0.0.1:0','--ticks',"$Ticks")
if ($CreatorResume) { $arguments+=@('--spawn','1686,6115,0') }
if ($gracefulStop) { $arguments+=@('--stop-file',"$runDir/stop") }
$server=Start-Process -FilePath "$ServerExecutable" -ArgumentList $arguments -WorkingDirectory "$root/b-rust" -WindowStyle Hidden -PassThru -RedirectStandardOutput "$runDir/server.log" -RedirectStandardError "$runDir/server.err"
$server.PriorityClass='BelowNormal';$server.ProcessorAffinity=[IntPtr]3
[ordered]@{pid=$server.Id;startUtc=$server.StartTime.ToUniversalTime().ToString('o');binaryHash=(Get-FileHash "$ServerExecutable").Hash;arguments=$arguments} | ConvertTo-Json -Depth 3 | Set-Content "$runDir/server.json"
$listeningUtc=$null
try {
    $deadline=[DateTime]::UtcNow.AddSeconds(30)
    $address=$null
    while (-not $server.HasExited -and [DateTime]::UtcNow -lt $deadline) {
        $log=Get-Content -LiteralPath "$runDir/server.log" -Raw
        if ($log -match 'Listening on (127\.0\.0\.1:\d+)') { $address=$Matches[1];break }
        Start-Sleep -Milliseconds 200
    }
    if (-not $address) { throw 'Rust server did not start; see server.err' }
    $listeningUtc=[DateTime]::UtcNow
    Write-Host "Running copied native client against $address"
    New-Item -ItemType Directory -Force -Path "$root/b-rust/work" | Out-Null
    Push-Location "$root/b-rust"
    try {
        $observerPython='C:/Users/backup/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe'
        & $observerPython "$PSScriptRoot/integration_logout.py" capture "$serverRoot/work/state/$StateName/accounts.sqlite" "$runDir/logout-before.json"
        if ($LASTEXITCODE -ne 0) {throw 'Cannot capture pre-client logout checkpoint'}
        $clientInvocation = [ordered]@{
            startUtc=[DateTime]::UtcNow.ToString('o'); endUtc=$null; elapsedMilliseconds=$null
            mode=$Mode; transportConnectUtc=$null
            scope='Client invocation only. Pinned client loads assets before socket connection; connect time remains UNKNOWN.'
        }
        $clientClock=[Diagnostics.Stopwatch]::StartNew()
        $clientInvocation | ConvertTo-Json | Set-Content "$runDir/client-invocation.json"
        switch ($Mode) {
            'tower-diary' { & "$scenarioBinary" "$CacheRoot/cache/js5" $address 'integration239' "$root/test-account.password" "$root/keys/client.key" "$root/runs/kotlin-inventory-002/world-state/floor-locations.tsv" "$runDir/scene-plane-0.tsv" *> "$runDir/client.log" }
            'floor' { & "$scenarioBinary" "$CacheRoot/cache/js5" $address 'integration239' "$root/test-account.password" "$root/keys/client.key" "$root/runs/kotlin-inventory-002/world-state/floor-locations.tsv" "$runDir/scene-plane-0.tsv" *> "$runDir/client.log" }
            'floor-content' { & "$scenarioBinary" "$CacheRoot/cache/js5" $address 'integration239' "$root/test-account.password" "$root/keys/client.key" "$root/runs/kotlin-inventory-002/world-state/floor-locations.tsv" "$runDir/scene-plane-0.tsv" vials *> "$runDir/client.log" }
            'floor-altar' { & "$scenarioBinary" "$CacheRoot/cache/js5" $address 'integration239' "$root/test-account.password" "$root/keys/client.key" "$root/runs/kotlin-inventory-002/world-state/floor-locations.tsv" "$runDir/scene-plane-0.tsv" altar *> "$runDir/client.log" }
            'login' { & "$ClientRoot/bin/live_session.exe" "$CacheRoot/cache/js5" "$root/keys/client.key" $address 'integration239' "$root/test-account.password" "$runDir/packets.bin" *> "$runDir/client.log" }
            'render' { & "$ClientRoot/bin/gielinor-client.exe" live-view "$CacheRoot/cache/js5" $address 'integration239' "$root/test-account.password" "$root/keys/client.key" --smoke *> "$runDir/client.log" }
            'actions' { & "$ClientRoot/bin/live_actions.exe" "$CacheRoot/cache/js5" $address 'integration239' "$root/test-account.password" "$root/keys/client.key" *> "$runDir/client.log" }
            'prayer' { & "$scenarioBinary" "$CacheRoot/cache/js5" $address 'integration239' "$root/test-account.password" "$root/keys/client.key" *> "$runDir/client.log" }
            'arrival' { & "$scenarioBinary" "$CacheRoot/cache/js5" $address 'integration239' "$root/test-account.password" "$root/keys/client.key" *> "$runDir/client.log" }
            'dialogue-ui' { & "$scenarioBinary" "$CacheRoot/cache/js5" $address 'integration239' "$root/test-account.password" "$root/keys/client.key" ui *> "$runDir/client.log" }
            'dialogue-protocol' { & "$scenarioBinary" "$CacheRoot/cache/js5" $address 'integration239' "$root/test-account.password" "$root/keys/client.key" protocol *> "$runDir/client.log" }
            'inventory' { & "$scenarioBinary" "$CacheRoot/cache/js5" $address 'integration239' "$root/test-account.password" "$root/keys/client.key" transact *> "$runDir/client.log" }
            'inventory-verify' { & "$scenarioBinary" "$CacheRoot/cache/js5" $address 'integration239' "$root/test-account.password" "$root/keys/client.key" verify *> "$runDir/client.log" }
            'inventory-bury' { & "$scenarioBinary" "$CacheRoot/cache/js5" $address 'integration239' "$root/test-account.password" "$root/keys/client.key" bury *> "$runDir/client.log" }
        }
        $clientExit=$LASTEXITCODE
        $clientClock.Stop()
        $clientInvocation.endUtc=[DateTime]::UtcNow.ToString('o')
        $clientInvocation.elapsedMilliseconds=$clientClock.ElapsedMilliseconds
        $clientInvocation | ConvertTo-Json | Set-Content "$runDir/client-invocation.json"
        & $observerPython "$PSScriptRoot/integration_logout.py" wait "$serverRoot/work/state/$StateName/accounts.sqlite" "$runDir/logout-observation.json" --before "$runDir/logout-before.json"
        if ($LASTEXITCODE -ne 0) {throw 'Logout observer failed; inspect run evidence'}
    } finally { Pop-Location }
    if ($Mode -eq 'render' -and (Test-Path "$root/b-rust/work/live-native.bmp")) { Copy-Item "$root/b-rust/work/live-native.bmp" "$runDir/world.bmp" }
    if ($gracefulStop) { Set-Content "$runDir/stop" 'Owned integration shutdown' }
    $server.WaitForExit(15000) | Out-Null
    $binary=switch -Wildcard ($Mode) { 'tower-diary' {"$scenarioBinary"} 'login' {"$ClientRoot/bin/live_session.exe"} 'render' {"$ClientRoot/bin/gielinor-client.exe"} 'actions' {"$ClientRoot/bin/live_actions.exe"} 'prayer' {"$scenarioBinary"} 'arrival' {"$scenarioBinary"} 'dialogue-*' {"$scenarioBinary"} 'inventory*' {"$scenarioBinary"} 'floor*' {"$scenarioBinary"} }
    [ordered]@{mode=$Mode;clientExit=$clientExit;serverExited=$server.HasExited;address=$address;clientHash=(Get-FileHash $binary).Hash} | ConvertTo-Json | Set-Content "$runDir/result.json"
    Get-Content "$runDir/client.log" -Tail 30
    if ($clientExit -ne 0) { throw "Native client exited $clientExit" }
} finally {
    # This exact child belongs to this run; no shared process is touched.
    # --ticks begins after cache loading/listen, not process creation. Include
    # bounded scheduling/save-drain grace for this low-priority process.
    $tickStart=if ($listeningUtc) { $listeningUtc } else { $server.StartTime.ToUniversalTime() }
    $shutdownDeadline=$tickStart.AddMilliseconds([Math]::Max(0,$Ticks)*600+30000)
    if ($gracefulStop) {
        Set-Content "$runDir/stop" 'Owned integration shutdown'
        $shutdownDeadline=[DateTime]::UtcNow.AddSeconds(30)
    }
    while (-not $server.HasExited -and [DateTime]::UtcNow -lt $shutdownDeadline) { $server.WaitForExit(1000) | Out-Null }
    $forced=-not $server.HasExited
    if ($forced) { $server.Kill();$server.WaitForExit() }
    [ordered]@{pid=$server.Id;exited=$server.HasExited;forced=$forced;exitCode=$server.ExitCode;finishedUtc=[DateTime]::UtcNow.ToString('o')} | ConvertTo-Json | Set-Content "$runDir/shutdown.json"
    $server.Dispose()
}


