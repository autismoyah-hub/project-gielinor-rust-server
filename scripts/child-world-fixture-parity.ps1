$ErrorActionPreference='Stop'
$serverRoot=Split-Path $PSScriptRoot -Parent
. "$PSScriptRoot/resources.ps1"
Assert-ServerBuildCapacity
$jdk='C:/Program Files/Eclipse Adoptium/jdk-21.0.12.101-hotspot/bin'
$out=Join-Path $serverRoot ('work/child-world-oracle-'+[DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss-fff'))
New-Item -ItemType Directory -Path $out -ErrorAction Stop | Out-Null
& "$jdk/javac.exe" '-J-Xmx256m' '-J-XX:ActiveProcessorCount=2' '-cp' "$serverRoot/work/rsprot/*" '-d' $out "$serverRoot/tools/child_world_fixture/ChildWorldFixtureOracle.java"
if($LASTEXITCODE -ne 0){throw 'Original child oracle compilation failed'}
& "$jdk/java.exe" '-Xmx128m' '-XX:ActiveProcessorCount=2' '-cp' "$out;$serverRoot/work/rsprot/*" ChildWorldFixtureOracle "$out/child-world.expected"
if($LASTEXITCODE -ne 0){throw 'Original child oracle failed; previous fixture preserved'}
$source=Get-Content -LiteralPath "$serverRoot/crates/server/src/child_world_fixture.rs" -Raw
$controls=@{}
foreach($match in [regex]::Matches($source,'"(?<name>active|rebuild|spawn|retain)"\s*,\s*"(?<hex>[0-9a-f]+)"')) {
    $controls[$match.Groups['name'].Value]=$match.Groups['hex'].Value
}
if($controls.Count -ne 4){throw 'Cannot identify the four synthetic Rust packet controls'}
$rows=@(Get-Content -LiteralPath "$out/child-world.expected")
if($rows.Count -ne 4){throw 'Original oracle did not emit four controls'}
foreach($row in $rows){
    $name,$hex=$row.Split(' ')
    if($controls[$name] -ne $hex){throw "Original control differs for $name; inspect preserved oracle output"}
}
& "$serverRoot/build.ps1" -Action test -CargoArgs @('-p','gielinor-server','--lib','--features','integration-fixtures','child_world_fixture::tests')
if($LASTEXITCODE -ne 0){throw 'Rust child fixture parity failed'}
