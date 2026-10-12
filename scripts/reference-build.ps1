param([ValidateSet(':engine:map:compileKotlin', ':server:app:classes')][string]$Task = ':engine:map:compileKotlin')
$ErrorActionPreference = 'Stop'
$serverRoot = Split-Path $PSScriptRoot -Parent
. "$PSScriptRoot/resources.ps1"
Assert-ServerBuildCapacity
$env:JAVA_HOME = 'C:\Program Files\Eclipse Adoptium\jdk-21.0.12.101-hotspot'
$env:GRADLE_USER_HOME = "$serverRoot/work/gradle"
$env:GRADLE_RO_DEP_CACHE = 'C:\Users\backup\.gradle\caches'
$env:JAVA_TOOL_OPTIONS = '-Xmx1536m -XX:MaxMetaspaceSize=384m -XX:ActiveProcessorCount=2'
$gradle = "$serverRoot/work/gradle-dist/gradle-9.0.0/bin/gradle.bat"
if (-not (Test-Path $gradle)) {
    $installedGradle = 'C:\Users\backup\.gradle\wrapper\dists\gradle-9.0.0-bin\d6wjpkvcgsg3oed0qlfss3wgl\gradle-9.0.0'
    New-Item -ItemType Directory -Force -Path "$serverRoot/work/gradle-dist" | Out-Null
    Copy-Item -LiteralPath $installedGradle -Destination "$serverRoot/work/gradle-dist/gradle-9.0.0" -Recurse
}
Push-Location "$serverRoot/reference/kotlin"
try {
    # Do not invoke install, packCache, run or the enormous exhaustive CoordGrid tests.
    & $gradle $Task --offline --no-daemon --max-workers=1 --no-parallel --console=plain --stacktrace '-Dorg.gradle.priority=low' '-Dorg.gradle.jvmargs=-Xmx1536m -XX:MaxMetaspaceSize=384m -XX:ActiveProcessorCount=2' '-Pkotlin.compiler.execution.strategy=in-process' '-Pkotlin.daemon.jvmargs=-Xmx1024m'
    if ($LASTEXITCODE -ne 0) { throw "Isolated Gradle build failed ($LASTEXITCODE)." }
} finally { Pop-Location }
