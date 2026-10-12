# Package existing compiled files only. No compiler, Gradle, source edits or game process.
# One archive per original directory preserves classpath and getResources ordering.
$ErrorActionPreference='Stop'
$serverRoot=Split-Path $PSScriptRoot -Parent
$root="$serverRoot/work/integration"
$proc=[Diagnostics.Process]::GetCurrentProcess();$proc.PriorityClass='BelowNormal';$proc.ProcessorAffinity=[IntPtr]3
$generation=[DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss-fffffff')
$directory="$root/kotlin-classpath-archives/$generation"
New-Item -ItemType Directory -Force -Path $directory | Out-Null
$paths=(Get-Content "$root/kotlin-classpath.txt" -Raw).Trim().Split(';')
$packed=@();$manifest=@();$index=0
foreach ($path in $paths) {
    if (Test-Path -LiteralPath $path -PathType Container) {
        $id='{0:D3}' -f $index
        $archive="$directory/$id.jar"
        if (-not (Test-Path -LiteralPath $archive)) {
            [IO.Compression.ZipFile]::CreateFromDirectory($path,$archive,[IO.Compression.CompressionLevel]::Fastest,$false)
            # ClassLoader.getResources("db/migration") and other package resource
            # scans require directory entries; CreateFromDirectory omits nonempty ones.
            $zip=[IO.Compression.ZipFile]::Open($archive,[IO.Compression.ZipArchiveMode]::Update)
            try {
                foreach ($folder in Get-ChildItem -LiteralPath $path -Directory -Recurse) {
                    $entry=$folder.FullName.Substring($path.Length).TrimStart([char[]]'\/').Replace('\','/')+'/'
                    if (-not $zip.GetEntry($entry)) { $zip.CreateEntry($entry) | Out-Null }
                }
            } finally { $zip.Dispose() }
        }
        $packed+=$archive
        $manifest+=[ordered]@{source=$path;archive=$archive;sha256=(Get-FileHash $archive).Hash}
    } else { $packed+=$path }
    $index++
}
Set-Content "$root/kotlin-runtime-classpath.txt" ($packed -join ';')
$manifest | ConvertTo-Json | Set-Content "$root/kotlin-classpath-archives.json"
Write-Host "Packaged $($manifest.Count) existing class/resource directories without changing classpath order."
