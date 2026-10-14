param([string]$Run='kotlin-login-001',[int]$Port=43728,[ValidateSet('login','render','actions','prayer','arrival','dialogue-ui','dialogue-protocol','inventory','inventory-verify','inventory-bury','floor','floor-content')][string]$Mode='login',[switch]$CaptureWorld,[switch]$CreatorResume,[string]$ClientRoot,[string]$EnvironmentPath,[string]$ScenarioExecutable)
$ErrorActionPreference='Stop'
$serverRoot=Split-Path $PSScriptRoot -Parent
. "$PSScriptRoot/integration-capacity.ps1"
Assert-IntegrationRunCapacity
if ($Port -lt 1024 -or $Port -gt 65535 -or $Port -eq 43594) { throw 'Invalid integration port' }
$portCheck=[Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback,$Port)
try { $portCheck.Start() } finally { $portCheck.Stop() }
$root="$serverRoot/work/integration"
if (-not $ClientRoot) { $ClientRoot="$root/client" }
if (-not $EnvironmentPath) { $EnvironmentPath="$root/a-kotlin" }
$scenario=if ($Mode -eq 'prayer') { 'server_prayer' } elseif ($Mode -eq 'arrival') { 'server_arrival' } elseif ($Mode -like 'dialogue-*') { 'server_dialogue' } elseif ($Mode -like 'inventory*') { 'server_inventory' } elseif ($Mode -like 'floor*') {'server_floor'} else { $null }
$scenarioBinary=if (Test-Path "$ClientRoot/bin/$scenario.exe") { "$ClientRoot/bin/$scenario.exe" } else { "$ClientRoot/target/debug/examples/$scenario.exe" }
if ($ScenarioExecutable) { $scenarioBinary=$ScenarioExecutable }
if ($scenario -and -not (Test-Path $scenarioBinary)) { throw 'Build the owned native scenario first: scripts/integration-client-build.ps1' }
$runDir="$root/runs/$Run"
if (Test-Path -LiteralPath $runDir) { throw 'Run already exists' }
if (-not (Test-Path "$root/kotlin-overlay/classes/IntegrationReference.class")) { throw 'Run integration-kotlin-build.ps1 first' }
New-Item -ItemType Directory -Path $runDir | Out-Null
$overlay=@(Get-ChildItem "$root/kotlin-overlay/classes" -Recurse -Filter '*.class' | ForEach-Object { [ordered]@{path=$_.FullName.Substring("$root/kotlin-overlay/classes".Length);sha256=(Get-FileHash $_.FullName).Hash} })
$overlay | ConvertTo-Json | Set-Content "$runDir/overlay-hashes.json"
$classpathFile=if (Test-Path "$root/kotlin-runtime-classpath.txt") { "$root/kotlin-runtime-classpath.txt" } else { "$root/kotlin-classpath.txt" }
$classpath="$root/kotlin-overlay/classes;"+(Get-Content $classpathFile -Raw).Trim()
# Lumbridge plus source squares copied by TowerFloor (dungeon and wizard room).
# Omitting these leaves the reference's constructed floor collision incomplete.
$centerX=if ($CreatorResume) {26} else {50}
$centerZ=if ($CreatorResume) {95} else {50}
# All four Tower template sources must be explicit when the login window moves.
$maps=@(@(foreach ($x in ($centerX-1)..($centerX+1)) { foreach ($z in ($centerZ-1)..($centerZ+1)) { ($x -shl 8) -bor $z } }) + @(12337,12843,12849,12851) | Sort-Object -Unique) -join ','
$args=@('-Xms128m','-Xmx768m','-XX:MaxMetaspaceSize=256m','-XX:MaxDirectMemorySize=384m','-XX:ActiveProcessorCount=2',"-Dgielinor.integration.port=$Port","-Dgielinor.integration.maps=$maps","-Dgielinor.integration.stop=$runDir/stop",'-cp',$classpath,'IntegrationReference')
New-Item -ItemType Directory -Path "$runDir/tmp" | Out-Null
# SQLite scans its native-library temp directory on startup. Keep that cleanup
# inside this run instead of encountering another agent's loaded native DLLs.
$args=@("-Djava.io.tmpdir=$runDir/tmp","-Dorg.sqlite.tmpdir=$runDir/tmp")+$args
if ($CaptureWorld) {
    if (-not (Test-Path "$root/kotlin-overlay/classes/IntegrationSnapshotKt.class")) { throw 'Rebuild the owned Kotlin integration wrapper before capturing world state' }
    $args=@("-Dgielinor.integration.snapshot=$runDir/world-state")+$args
}
$args | ForEach-Object {'"'+$_.Replace('\','/')+'"'} | Set-Content "$runDir/java.args"
$server=Start-Process -FilePath 'C:/Program Files/Eclipse Adoptium/jdk-21.0.12.101-hotspot/bin/java.exe' -ArgumentList "@$runDir/java.args" -WorkingDirectory "$EnvironmentPath" -WindowStyle Hidden -PassThru -RedirectStandardOutput "$runDir/server.log" -RedirectStandardError "$runDir/server.err"
$server.PriorityClass='BelowNormal';$server.ProcessorAffinity=[IntPtr]3
[ordered]@{pid=$server.Id;startUtc=$server.StartTime.ToUniversalTime().ToString('o');port=$Port;arguments=$args} | ConvertTo-Json -Depth 3 | Set-Content "$runDir/server.json"
try {
    $deadline=[DateTime]::UtcNow.AddSeconds(180)
    $ready=$false
    while (-not $server.HasExited -and [DateTime]::UtcNow -lt $deadline) {
        $socket=New-Object Net.Sockets.TcpClient
        try { $socket.Connect('127.0.0.1',$Port);$ready=$true;break } catch {} finally { $socket.Dispose() }
        Start-Sleep -Milliseconds 500
    }
    if (-not $ready) {
        if ($server.HasExited) { throw "Isolated reference exited before listening (code $($server.ExitCode)); inspect server logs." }
        throw 'Isolated reference did not listen within 180 seconds; inspect server logs.'
    }
    Push-Location "$EnvironmentPath"
    try {
        $observerPython='C:/Users/backup/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe'
        & $observerPython "$PSScriptRoot/integration_logout.py" capture "$EnvironmentPath/.data/saves/game.db" "$runDir/logout-before.json"
        if ($LASTEXITCODE -ne 0) {throw 'Cannot capture pre-client logout checkpoint'}
        switch ($Mode) {
            'floor' { & "$scenarioBinary" "$EnvironmentPath/.data/cache/js5" "127.0.0.1:$Port" 'integration239' "$root/test-account.password" "$root/keys/client.key" "$root/runs/kotlin-inventory-002/world-state/floor-locations.tsv" "$runDir/scene-plane-0.tsv" *> "$runDir/client.log" }
            'floor-content' { & "$scenarioBinary" "$EnvironmentPath/.data/cache/js5" "127.0.0.1:$Port" 'integration239' "$root/test-account.password" "$root/keys/client.key" "$root/runs/kotlin-inventory-002/world-state/floor-locations.tsv" "$runDir/scene-plane-0.tsv" vials *> "$runDir/client.log" }
            'login' { & "$ClientRoot/bin/live_session.exe" "$EnvironmentPath/.data/cache/js5" "$root/keys/client.key" "127.0.0.1:$Port" 'integration239' "$root/test-account.password" "$runDir/packets.bin" *> "$runDir/client.log" }
            'render' { & "$ClientRoot/bin/gielinor-client.exe" live-view "$EnvironmentPath/.data/cache/js5" "127.0.0.1:$Port" 'integration239' "$root/test-account.password" "$root/keys/client.key" --smoke *> "$runDir/client.log" }
            'actions' { & "$ClientRoot/bin/live_actions.exe" "$EnvironmentPath/.data/cache/js5" "127.0.0.1:$Port" 'integration239' "$root/test-account.password" "$root/keys/client.key" *> "$runDir/client.log" }
            'prayer' { & "$scenarioBinary" "$EnvironmentPath/.data/cache/js5" "127.0.0.1:$Port" 'integration239' "$root/test-account.password" "$root/keys/client.key" *> "$runDir/client.log" }
            'arrival' { & "$scenarioBinary" "$EnvironmentPath/.data/cache/js5" "127.0.0.1:$Port" 'integration239' "$root/test-account.password" "$root/keys/client.key" *> "$runDir/client.log" }
            'dialogue-ui' { & "$scenarioBinary" "$EnvironmentPath/.data/cache/js5" "127.0.0.1:$Port" 'integration239' "$root/test-account.password" "$root/keys/client.key" ui *> "$runDir/client.log" }
            'dialogue-protocol' { & "$scenarioBinary" "$EnvironmentPath/.data/cache/js5" "127.0.0.1:$Port" 'integration239' "$root/test-account.password" "$root/keys/client.key" protocol *> "$runDir/client.log" }
            'inventory' { & "$scenarioBinary" "$EnvironmentPath/.data/cache/js5" "127.0.0.1:$Port" 'integration239' "$root/test-account.password" "$root/keys/client.key" transact *> "$runDir/client.log" }
            'inventory-verify' { & "$scenarioBinary" "$EnvironmentPath/.data/cache/js5" "127.0.0.1:$Port" 'integration239' "$root/test-account.password" "$root/keys/client.key" verify *> "$runDir/client.log" }
            'inventory-bury' { & "$scenarioBinary" "$EnvironmentPath/.data/cache/js5" "127.0.0.1:$Port" 'integration239' "$root/test-account.password" "$root/keys/client.key" bury *> "$runDir/client.log" }
        }
        $clientExit=$LASTEXITCODE
        & $observerPython "$PSScriptRoot/integration_logout.py" wait "$EnvironmentPath/.data/saves/game.db" "$runDir/logout-observation.json" --before "$runDir/logout-before.json"
        if ($LASTEXITCODE -ne 0) {throw 'Logout observer failed; inspect run evidence'}
    } finally { Pop-Location }
    if ($Mode -eq 'render' -and (Test-Path "$EnvironmentPath/work/live-native.bmp")) { Copy-Item "$EnvironmentPath/work/live-native.bmp" "$runDir/world.bmp" }
    $binary=switch -Wildcard ($Mode) { 'login' {"$ClientRoot/bin/live_session.exe"} 'render' {"$ClientRoot/bin/gielinor-client.exe"} 'actions' {"$ClientRoot/bin/live_actions.exe"} 'prayer' {"$scenarioBinary"} 'arrival' {"$scenarioBinary"} 'dialogue-*' {"$scenarioBinary"} 'inventory*' {"$scenarioBinary"} 'floor*' {"$scenarioBinary"} }
    [ordered]@{mode=$Mode;clientExit=$clientExit;clientHash=(Get-FileHash $binary).Hash} | ConvertTo-Json | Set-Content "$runDir/result.json"
    Get-Content "$runDir/client.log" -Tail 30
    if ($clientExit -ne 0) { throw "Native client exited $clientExit" }
} finally {
    Set-Content "$runDir/stop" 'Owned integration shutdown'
    $server.WaitForExit(15000) | Out-Null
    $forced=-not $server.HasExited
    if ($forced) { $server.Kill();$server.WaitForExit() }
    [ordered]@{pid=$server.Id;exited=$server.HasExited;forced=$forced;exitCode=$server.ExitCode;finishedUtc=[DateTime]::UtcNow.ToString('o')} | ConvertTo-Json | Set-Content "$runDir/shutdown.json"
    $server.Dispose()
}


