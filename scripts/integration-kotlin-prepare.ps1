$ErrorActionPreference='Stop'
$serverRoot=Split-Path $PSScriptRoot -Parent
$root="$serverRoot/work/integration"
$reference="$serverRoot/reference/kotlin"
New-Item -ItemType Directory -Force -Path "$root/kotlin-libs","$root/kotlin-overlay" | Out-Null
foreach ($cache in @('vanilla','enriched')) {
    $target="$root/a-kotlin/.data/cache/$cache"
    if (-not (Test-Path -LiteralPath $target)) { Copy-Item -LiteralPath "C:/Projects/OSRSRSPS/Project-Gielinor/.data/cache/$cache" -Destination $target -Recurse }
}
# External runtime dependencies only; all application classes come from our
# isolated Kotlin build. The original application jars must not enter this path.
$external=Get-ChildItem 'C:/Projects/OSRSRSPS/Project-Gielinor/server/app/build/install/app/lib' -File -Filter '*.jar' | Where-Object {$_.Name -notlike '*-0.0.1.jar'}
foreach ($jar in $external) {
    $target=Join-Path "$root/kotlin-libs" $jar.Name
    if (-not (Test-Path -LiteralPath $target)) { Copy-Item -LiteralPath $jar.FullName -Destination $target }
}
$projects=Get-ChildItem "$reference/api","$reference/engine","$reference/content","$reference/server" -Recurse -Filter build.gradle.kts -File
$paths=@()
foreach ($project in $projects) {
    foreach ($suffix in @('build/classes/kotlin/main','build/classes/java/main','build/resources/main')) {
        $candidate=Join-Path $project.DirectoryName $suffix
        if (Test-Path -LiteralPath $candidate) { $paths+=$candidate }
    }
}
$paths+=@(Get-ChildItem "$root/kotlin-libs" -Filter '*.jar' | ForEach-Object FullName)
Set-Content "$root/kotlin-classpath.txt" ($paths -join ';')
$source=Get-Content "$reference/api/net/src/main/kotlin/org/rsmod/api/net/rsprot/NetworkFactory.kt" -Raw
$old='override val ports: List<Int> = listOf(43594)'
$new=@'
override val ports: List<Int> = listOf(
        requireNotNull(System.getProperty("gielinor.integration.port")) { "Dedicated integration port required" }
            .toInt().also { check(it in 1024..65535 && it != 43594) }
    )
    override val host: String? = "127.0.0.1"
'@
if (-not $source.Contains($old)) { throw 'NetworkFactory source changed; review isolation overlay.' }
Set-Content "$root/kotlin-overlay/NetworkFactory.kt" $source.Replace($old,$new)
$mapSource=Get-Content "$reference/api/cache/src/main/kotlin/org/rsmod/api/cache/map/GameMapDecoder.kt" -Raw
$mapSource=$mapSource.Replace('val archive = Js5Archives.MAPS',@'
val archive = Js5Archives.MAPS
        val integrationSquares = requireNotNull(System.getProperty("gielinor.integration.maps"))
            .split(",").map(String::toInt).toSet()
'@)
$mapSource=$mapSource.Replace('val mapSquareKey = MapSquareKey(group.id)',@'
if (group.id !in integrationSquares) return@mapNotNull null
                val mapSquareKey = MapSquareKey(group.id)
'@)
Set-Content "$root/kotlin-overlay/GameMapDecoder.kt" $mapSource
# The reference loader rejects private TypeReferences objects. Preserve every
# item binding and change only visibility in our shadow classes, not the source.
$referenceObjects=@('package org.rsmod.content.other.shops','import org.rsmod.api.type.refs.obj.ObjReferences')
foreach ($file in Get-ChildItem "$reference/content/other/kronos-shops/src/main/kotlin/org/rsmod/content/other/shops" -Filter '*.kt') {
    $text=Get-Content $file.FullName -Raw
    foreach ($match in [regex]::Matches($text,'(?ms)^private object \w+ : ObjReferences\(\) \{.*?^\}')) {
        $referenceObjects+=$match.Value.Replace('private object ','public object ')
    }
}
Set-Content "$root/kotlin-overlay/KronosReferenceVisibility.kt" ($referenceObjects -join "`n`n")
Set-Content "$root/a-kotlin/.data/server.toml" "realm = 'dev'`nworld = 1`nfirst_launch = false"
$manifest=@(Get-ChildItem "$root/kotlin-libs" -File | ForEach-Object { [ordered]@{name=$_.Name;sha256=(Get-FileHash $_.FullName).Hash} })
$manifest | ConvertTo-Json | Set-Content "$root/kotlin-dependencies.json"
Write-Host "Kotlin classpath prepared with $($paths.Count) entries; no build or server started."
