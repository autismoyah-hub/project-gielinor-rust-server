$ErrorActionPreference='Stop'
$serverRoot=Split-Path $PSScriptRoot -Parent
. "$PSScriptRoot/resources.ps1"
Assert-ServerBuildCapacity
$root="$serverRoot/work/integration"
$classpath=(Get-Content "$root/kotlin-classpath.txt" -Raw).Trim()
$cache='C:/Users/backup/.gradle/caches/modules-2/files-2.1'
$specs=@(
    @('org.jetbrains.kotlin','kotlin-compiler-embeddable','2.2.0'),
    @('org.jetbrains.kotlin','kotlin-stdlib','2.2.0'),
    @('org.jetbrains.kotlin','kotlin-script-runtime','2.2.0'),
    @('org.jetbrains.kotlin','kotlin-reflect','1.6.10'),
    @('org.jetbrains.kotlin','kotlin-daemon-embeddable','2.2.0'),
    @('org.jetbrains.kotlinx','kotlinx-coroutines-core-jvm','1.8.0'),
    @('org.jetbrains','annotations','13.0')
)
$compiler=@(foreach ($s in $specs) {
    $found=@(Get-ChildItem "$cache/$($s[0])/$($s[1])/$($s[2])" -Recurse -Filter "$($s[1])-$($s[2]).jar")
    if ($found.Count -ne 1) { throw "Expected one compiler artifact: $s" }
    $found[0].FullName
})
$jdk='C:/Program Files/Eclipse Adoptium/jdk-21.0.12.101-hotspot/bin'
$arguments=@('-Xskip-prerelease-check','-no-stdlib','-no-reflect','-jvm-target','21','-classpath',$classpath,'-d',"$root/kotlin-overlay/classes","$root/kotlin-overlay/NetworkFactory.kt","$root/kotlin-overlay/GameMapDecoder.kt","$root/kotlin-overlay/KronosReferenceVisibility.kt","$serverRoot/reference/IntegrationSnapshot.kt")
$arguments | ForEach-Object { '"'+$_.Replace('\','/')+'"' } | Set-Content "$root/kotlin-overlay/compile.args"
& "$jdk/java.exe" '-Xmx512m' '-XX:ActiveProcessorCount=2' '-cp' ($compiler -join ';') org.jetbrains.kotlin.cli.jvm.K2JVMCompiler "@$root/kotlin-overlay/compile.args"
if ($LASTEXITCODE -ne 0) { throw 'Network isolation overlay compilation failed.' }
$arguments=@('-cp',"$root/kotlin-overlay/classes;$classpath",'-d',"$root/kotlin-overlay/classes","$serverRoot/reference/IntegrationReference.java")
$arguments | ForEach-Object { '"'+$_.Replace('\','/')+'"' } | Set-Content "$root/kotlin-overlay/javac.args"
& "$jdk/javac.exe" '-J-Xmx128m' '-J-XX:ActiveProcessorCount=2' "@$root/kotlin-overlay/javac.args"
if ($LASTEXITCODE -ne 0) { throw 'Reference lifecycle wrapper compilation failed.' }
Write-Host 'Kotlin network and lifecycle isolation wrappers compiled.'
